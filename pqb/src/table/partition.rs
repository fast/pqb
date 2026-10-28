// Copyright 2025 FastLabs Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::expr::Expr;
use crate::expr::write_expr;
use crate::value::Value;
use crate::value::write_value;
use crate::writer::SqlWriter;

/// A table's partition strategy and key expressions.
#[derive(Debug, Clone, PartialEq)]
pub struct PartitionBy {
    strategy: Strategy,
    keys: Vec<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strategy {
    Range,
    List,
    Hash,
}

impl PartitionBy {
    /// Partition by one or more ordered keys, e.g. `[Expr::column("created_at")]`.
    pub fn range(keys: impl IntoIterator<Item = Expr>) -> Self {
        Self::new(Strategy::Range, keys)
    }

    /// Partition by one key's discrete values. PostgreSQL LIST accepts exactly one key.
    pub fn list(keys: impl IntoIterator<Item = Expr>) -> Self {
        Self::new(Strategy::List, keys)
    }

    /// Partition by the hash of one or more keys.
    pub fn hash(keys: impl IntoIterator<Item = Expr>) -> Self {
        Self::new(Strategy::Hash, keys)
    }

    fn new(strategy: Strategy, keys: impl IntoIterator<Item = Expr>) -> Self {
        let keys: Vec<_> = keys.into_iter().collect();
        assert!(!keys.is_empty(), "partitioning requires key expressions");
        assert!(
            strategy != Strategy::List || keys.len() == 1,
            "LIST partitioning requires exactly one key"
        );
        Self { strategy, keys }
    }
}

/// A RANGE endpoint: an inline value, or the unbounded MINVALUE/MAXVALUE sentinel.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum PartitionValue {
    Value(Value),
    MinValue,
    MaxValue,
}

impl<T: Into<Value>> From<T> for PartitionValue {
    fn from(value: T) -> Self {
        Self::Value(value.into())
    }
}

/// The rows assigned to a partition. Values are escaped inline, never bound parameters.
///
/// Constructors reject malformed ranges and hash bounds. PostgreSQL checks key types,
/// ordering, overlapping ranges and consistency with the parent's partition strategy.
#[derive(Debug, Clone, PartialEq)]
pub struct PartitionBound {
    kind: BoundKind,
}

#[derive(Debug, Clone, PartialEq)]
enum BoundKind {
    Range(Vec<PartitionValue>, Vec<PartitionValue>),
    List(Vec<Value>),
    Hash { modulus: u32, remainder: u32 },
    Default,
}

impl PartitionBound {
    /// Assign an inclusive lower bound and exclusive upper bound, with equal key counts.
    ///
    /// Use [`PartitionValue::MinValue`] or [`PartitionValue::MaxValue`] for unbounded keys.
    /// Every key after a sentinel must use the same sentinel. NULL range bounds are invalid.
    pub fn range<L: Into<PartitionValue>, U: Into<PartitionValue>>(
        lower: impl IntoIterator<Item = L>,
        upper: impl IntoIterator<Item = U>,
    ) -> Self {
        let lower: Vec<_> = lower.into_iter().map(Into::into).collect();
        let upper: Vec<_> = upper.into_iter().map(Into::into).collect();
        assert!(
            !lower.is_empty() && lower.len() == upper.len(),
            "partition range bounds require equal, nonempty key counts"
        );
        for bound in [&lower, &upper] {
            for (index, value) in bound.iter().enumerate() {
                match value {
                    PartitionValue::Value(value) => {
                        assert!(!value.is_null(), "RANGE bounds cannot contain NULL")
                    }
                    sentinel => assert!(
                        bound[index..].iter().all(|v| v == sentinel),
                        "keys after MINVALUE or MAXVALUE must use the same sentinel"
                    ),
                }
            }
        }
        Self {
            kind: BoundKind::Range(lower, upper),
        }
    }

    /// Assign a nonempty list of values. LIST partitions may include NULL.
    pub fn list<V: Into<Value>>(values: impl IntoIterator<Item = V>) -> Self {
        let values: Vec<_> = values.into_iter().map(Into::into).collect();
        assert!(!values.is_empty(), "LIST partition bounds require values");
        Self {
            kind: BoundKind::List(values),
        }
    }

    /// Assign a hash remainder. Modulus must be positive and fit an int4; remainder < modulus.
    pub fn hash(modulus: u32, remainder: u32) -> Self {
        assert!(
            modulus > 0 && modulus <= i32::MAX as u32 && remainder < modulus,
            "invalid HASH partition modulus or remainder"
        );
        Self {
            kind: BoundKind::Hash { modulus, remainder },
        }
    }

    /// Assign all rows not covered by other partitions (RANGE or LIST only).
    pub fn default_partition() -> Self {
        Self {
            kind: BoundKind::Default,
        }
    }
}

pub(crate) fn write_partition_by<W: SqlWriter>(w: &mut W, partition: &PartitionBy) {
    w.push_str(" PARTITION BY ");
    w.push_str(match partition.strategy {
        Strategy::Range => "RANGE",
        Strategy::List => "LIST",
        Strategy::Hash => "HASH",
    });
    w.push_str(" (");
    for (i, key) in partition.keys.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        let expression = !matches!(key, Expr::Column(_));
        if expression {
            w.push_char('(');
        }
        write_expr(w, key);
        if expression {
            w.push_char(')');
        }
    }
    w.push_char(')');
}

pub(crate) fn write_partition_bound<W: SqlWriter>(w: &mut W, bound: &PartitionBound) {
    match &bound.kind {
        BoundKind::Default => w.push_str("DEFAULT"),
        BoundKind::Hash { modulus, remainder } => w.push_fmt(format_args!(
            "FOR VALUES WITH (MODULUS {modulus}, REMAINDER {remainder})"
        )),
        BoundKind::List(values) => {
            w.push_str("FOR VALUES IN (");
            for (i, value) in values.iter().enumerate() {
                if i > 0 {
                    w.push_str(", ");
                }
                write_value(w, value);
            }
            w.push_char(')');
        }
        BoundKind::Range(lower, upper) => {
            w.push_str("FOR VALUES FROM (");
            for (index, values) in [lower, upper].iter().enumerate() {
                if index > 0 {
                    w.push_str(") TO (");
                }
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        w.push_str(", ");
                    }
                    match value {
                        PartitionValue::MinValue => w.push_str("MINVALUE"),
                        PartitionValue::MaxValue => w.push_str("MAXVALUE"),
                        PartitionValue::Value(value) => write_value(w, value),
                    }
                }
            }
            w.push_char(')');
        }
    }
}
