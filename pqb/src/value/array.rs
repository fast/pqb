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

//! Rectangular PostgreSQL arrays with explicit dimensions.

use super::ArrayType;
use super::Value;
use super::write_value;
use crate::writer::SqlWriter;

/// The length and first subscript of an array axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrayDimension {
    /// Number of elements along this axis.
    pub length: usize,
    /// First subscript, normally 1. PostgreSQL also accepts zero and negative lower bounds.
    pub lower_bound: i32,
}

impl From<usize> for ArrayDimension {
    fn from(length: usize) -> Self {
        Self {
            length,
            lower_bound: 1,
        }
    }
}

impl From<(usize, i32)> for ArrayDimension {
    fn from((length, lower_bound): (usize, i32)) -> Self {
        Self {
            length,
            lower_bound,
        }
    }
}

/// A typed rectangular array with explicit dimensions and flat, row-major elements.
///
/// The last subscript varies fastest. Convert to [`Value`] to collect it as one parameter or render
/// an SQL array literal. Any zero-length axis produces PostgreSQL's canonical empty array with no
/// dimensions; use [`Value::null_array`] for a NULL array. Elements may be typed NULL values.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayValue {
    ty: ArrayType,
    dimensions: Vec<ArrayDimension>,
    values: Vec<Value>,
}

/// An array shape or element type that PostgreSQL cannot represent.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArrayError {
    /// PostgreSQL supports at most six dimensions.
    TooManyDimensions,
    /// A dimension length, exclusive upper bound or total element count overflows int4.
    DimensionOverflow,
    /// The flat element count does not match the dimensions' product.
    ElementCount {
        /// Number of elements required by the shape.
        expected: usize,
        /// Number of elements supplied.
        actual: usize,
    },
    /// An element (including typed NULL) is incompatible with the declared array type.
    /// Unsigned integers must also fit the corresponding signed PostgreSQL type.
    ElementType {
        /// Zero-based position in the flat element list.
        index: usize,
        /// Required element type.
        expected: ArrayType,
    },
}

impl std::fmt::Display for ArrayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyDimensions => {
                f.write_str("PostgreSQL arrays support at most six dimensions")
            }
            Self::DimensionOverflow => {
                f.write_str("array dimension or element count exceeds PostgreSQL's int4 bounds")
            }
            Self::ElementCount { expected, actual } => {
                write!(f, "array shape requires {expected} elements, got {actual}")
            }
            Self::ElementType { index, expected } => write!(
                f,
                "array element {index} cannot be represented as {}",
                expected.sql_name()
            ),
        }
    }
}

impl std::error::Error for ArrayError {}

impl ArrayValue {
    /// Validate and create an array. Dimensions accept lengths (lower bound 1),
    /// `(length, lower_bound)` pairs or [`ArrayDimension`] values. Unsigned integers must fit the
    /// corresponding signed PostgreSQL type; in particular, `u64` values must fit `bigint`.
    ///
    /// ```
    /// use pqb::value::ArrayType;
    /// use pqb::value::ArrayValue;
    /// let matrix = ArrayValue::new(ArrayType::Int, [2, 2], [Some(1), None, Some(3), Some(4)])?;
    /// let shifted = ArrayValue::new(ArrayType::Int, [(2, 0), (2, -1)], [1, 2, 3, 4])?;
    /// # Ok::<(), pqb::value::ArrayError>(())
    /// ```
    pub fn new<D: Into<ArrayDimension>, V: Into<Value>>(
        ty: ArrayType,
        dimensions: impl IntoIterator<Item = D>,
        values: impl IntoIterator<Item = V>,
    ) -> Result<Self, ArrayError> {
        let mut dimensions: Vec<_> = dimensions.into_iter().map(Into::into).collect();
        if dimensions.len() > 6 {
            return Err(ArrayError::TooManyDimensions);
        }
        for dimension in &dimensions {
            let length =
                i32::try_from(dimension.length).map_err(|_| ArrayError::DimensionOverflow)?;
            // PostgreSQL checks the exclusive upper bound, not just the last valid subscript.
            dimension
                .lower_bound
                .checked_add(length)
                .ok_or(ArrayError::DimensionOverflow)?;
        }
        if dimensions.iter().any(|d| d.length == 0) {
            dimensions.clear();
        }
        let expected = if dimensions.is_empty() {
            0
        } else {
            dimensions
                .iter()
                .try_fold(1_usize, |size, d| size.checked_mul(d.length))
                .filter(|size| *size <= i32::MAX as usize)
                .ok_or(ArrayError::DimensionOverflow)?
        };
        let values: Vec<_> = values.into_iter().map(Into::into).collect();
        if values.len() != expected {
            return Err(ArrayError::ElementCount {
                expected,
                actual: values.len(),
            });
        }
        for (index, value) in values.iter().enumerate() {
            if ArrayType::of_value(value) != Some(ty) {
                return Err(ArrayError::ElementType {
                    index,
                    expected: ty,
                });
            }
        }
        Ok(Self {
            ty,
            dimensions,
            values,
        })
    }

    /// The declared SQL element type.
    pub fn element_type(&self) -> ArrayType {
        self.ty
    }

    /// Validated dimensions, or an empty slice for an empty array.
    pub fn dimensions(&self) -> &[ArrayDimension] {
        &self.dimensions
    }

    /// Flat elements in row-major order.
    pub fn values(&self) -> &[Value] {
        &self.values
    }
}

impl From<ArrayValue> for Value {
    fn from(array: ArrayValue) -> Self {
        Self::MultidimensionalArray(array)
    }
}

impl ArrayType {
    fn of_value(value: &Value) -> Option<Self> {
        Some(match value {
            Value::BigUnsigned(Some(value)) if *value > i64::MAX as u64 => return None,
            Value::TinyInt(_) | Value::TinyUnsigned(_) | Value::SmallInt(_) => Self::SmallInt,
            Value::SmallUnsigned(_) | Value::Int(_) => Self::Int,
            Value::Unsigned(_) | Value::BigUnsigned(_) | Value::BigInt(_) => Self::BigInt,
            Value::Array(_) | Value::TypedArray(..) | Value::MultidimensionalArray(_) => {
                return None;
            }
            Value::Bool(_) => Self::Bool,
            Value::Float(_) => Self::Float,
            Value::Double(_) => Self::Double,
            Value::String(_) => Self::Text,
            Value::Bytes(_) => Self::Bytes,
            #[cfg(feature = "with-json")]
            Value::Json(_) => Self::Json,
            #[cfg(feature = "with-uuid")]
            Value::Uuid(_) => Self::Uuid,
            #[cfg(feature = "with-chrono")]
            Value::Date(_) => Self::Date,
            #[cfg(feature = "with-chrono")]
            Value::Time(_) => Self::Time,
            #[cfg(feature = "with-chrono")]
            Value::Timestamp(_) => Self::Timestamp,
            #[cfg(feature = "with-chrono")]
            Value::TimestampWithTimeZone(_) => Self::TimestampWithTimeZone,
            #[cfg(feature = "with-rust_decimal")]
            Value::Decimal(_) => Self::Decimal,
        })
    }
}

pub(super) fn write_array<W: SqlWriter>(w: &mut W, array: &ArrayValue) {
    let shifted = array.dimensions.iter().any(|d| d.lower_bound != 1);
    if shifted {
        // Let PostgreSQL serialize element text (notably bytea/JSON and escaped strings) before
        // adding explicit bounds, preserving quoting for every scalar type.
        w.push_str("('");
        for d in &array.dimensions {
            w.push_fmt(format_args!(
                "[{}:{}]",
                d.lower_bound,
                i64::from(d.lower_bound) + d.length as i64 - 1
            ));
        }
        w.push_str("=' || (");
    }
    w.push_str("ARRAY");
    write_axis(w, &array.dimensions, &array.values, &mut 0);
    w.push_str("::");
    w.push_str(array.ty.sql_name());
    w.push_str("[]");
    if shifted {
        w.push_str(")::text)::");
        w.push_str(array.ty.sql_name());
        w.push_str("[]");
    }
}

fn write_axis<W: SqlWriter>(
    w: &mut W,
    dimensions: &[ArrayDimension],
    values: &[Value],
    offset: &mut usize,
) {
    w.push_char('[');
    if let Some(axis) = dimensions.first() {
        for index in 0..axis.length {
            if index > 0 {
                w.push_str(", ");
            }
            if dimensions.len() > 1 {
                write_axis(w, &dimensions[1..], values, offset);
            } else {
                write_value(w, &values[*offset]);
                *offset += 1;
            }
        }
    }
    w.push_char(']');
}
