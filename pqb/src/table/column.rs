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

use std::sync::Arc;

use crate::expr::Expr;
use crate::expr::write_expr;
use crate::types::Iden;
use crate::writer::SqlWriter;

/// Specification of a table column.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ColumnDef {
    pub(crate) name: Iden,
    pub(crate) ty: Option<ColumnType>,
    pub(crate) spec: ColumnSpec,
}

impl ColumnDef {
    /// Create a new column definition with the given name.
    pub fn new<N>(name: N) -> Self
    where
        N: Into<Iden>,
    {
        Self {
            name: name.into(),
            ty: None,
            spec: ColumnSpec::default(),
        }
    }

    /// Set column not null
    pub fn not_null(mut self) -> Self {
        self.spec.nullable = Some(false);
        self
    }

    /// Set column null
    pub fn null(mut self) -> Self {
        self.spec.nullable = Some(true);
        self
    }

    /// Set column type as text
    pub fn text(mut self) -> Self {
        self.ty = Some(ColumnType::Text);
        self
    }

    /// Set column type as smallint
    pub fn smallint(mut self) -> Self {
        self.ty = Some(ColumnType::SmallInt);
        self
    }

    /// Set column type as int
    pub fn int(mut self) -> Self {
        self.ty = Some(ColumnType::Int);
        self
    }

    /// Set column type as bigint
    pub fn bigint(mut self) -> Self {
        self.ty = Some(ColumnType::BigInt);
        self
    }

    /// Set column type as float
    pub fn float(mut self) -> Self {
        self.ty = Some(ColumnType::Float);
        self
    }

    /// Set column type as double
    pub fn double(mut self) -> Self {
        self.ty = Some(ColumnType::Double);
        self
    }

    /// Set column type as timestamp
    pub fn timestamp(mut self) -> Self {
        self.ty = Some(ColumnType::Timestamp);
        self
    }

    /// Set column type as timestamp with time zone.
    pub fn timestamp_with_time_zone(mut self) -> Self {
        self.ty = Some(ColumnType::TimestampWithTimeZone);
        self
    }

    /// Set column type as time
    pub fn time(mut self) -> Self {
        self.ty = Some(ColumnType::Time);
        self
    }

    /// Set column type as date
    pub fn date(mut self) -> Self {
        self.ty = Some(ColumnType::Date);
        self
    }

    /// Set column type as JSON.
    pub fn json(mut self) -> Self {
        self.ty = Some(ColumnType::Json);
        self
    }

    /// Set column type as JSON binary.
    pub fn json_binary(mut self) -> Self {
        self.ty = Some(ColumnType::JsonBinary);
        self
    }

    /// Set column type as uuid
    pub fn uuid(mut self) -> Self {
        self.ty = Some(ColumnType::Uuid);
        self
    }

    /// Set column as generated with expression and stored storage.
    pub fn generated_as_stored<E>(mut self, expr: E) -> Self
    where
        E: Into<Expr>,
    {
        self.spec.generated = Some(GeneratedColumn {
            expr: expr.into(),
            kind: GeneratedColumnKind::Stored,
        });
        self
    }

    /// Set column as generated with expression and virtual storage.
    pub fn generated_as_virtual<E>(mut self, expr: E) -> Self
    where
        E: Into<Expr>,
    {
        self.spec.generated = Some(GeneratedColumn {
            expr: expr.into(),
            kind: GeneratedColumnKind::Virtual,
        });
        self
    }
}

/// Column data types.
#[derive(Debug, Clone)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum ColumnType {
    Text,
    SmallInt,
    Int,
    BigInt,
    Float,
    Double,
    DateTime,
    Timestamp,
    TimestampWithTimeZone,
    Time,
    Date,
    Boolean,
    Json,
    JsonBinary,
    Uuid,
    Array(Arc<ColumnType>),
}

/// Specification of column attributes.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
#[expect(missing_docs)]
pub struct ColumnSpec {
    pub nullable: Option<bool>,
    pub default: Option<Expr>,
    pub generated: Option<GeneratedColumn>,
    pub unique: bool,
    pub primary_key: bool,
}

/// Generated column specification.
#[derive(Debug, Clone)]
#[non_exhaustive]
#[expect(missing_docs)]
pub struct GeneratedColumn {
    pub expr: Expr,
    /// Before PostgreSQL 18, STORED is the only supported kind and must be specified.
    pub kind: GeneratedColumnKind,
}

/// Generated column storage kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum GeneratedColumnKind {
    Stored,
    Virtual,
}

pub(crate) fn write_column_type<W: SqlWriter>(w: &mut W, column_type: &ColumnType) {
    match column_type {
        ColumnType::Text => w.push_str("text"),
        ColumnType::SmallInt => w.push_str("smallint"),
        ColumnType::Int => w.push_str("integer"),
        ColumnType::BigInt => w.push_str("bigint"),
        ColumnType::Float => w.push_str("real"),
        ColumnType::Double => w.push_str("double precision"),
        ColumnType::DateTime => w.push_str("timestamp without time zone"),
        ColumnType::Timestamp => w.push_str("timestamp"),
        ColumnType::TimestampWithTimeZone => w.push_str("timestamp with time zone"),
        ColumnType::Time => w.push_str("time"),
        ColumnType::Date => w.push_str("date"),
        ColumnType::Boolean => w.push_str("bool"),
        ColumnType::Json => w.push_str("json"),
        ColumnType::JsonBinary => w.push_str("jsonb"),
        ColumnType::Uuid => w.push_str("uuid"),
        ColumnType::Array(ty) => {
            write_column_type(w, ty);
            w.push_str("[]");
        }
    }
}

pub(crate) fn write_column_spec<W: SqlWriter>(w: &mut W, column_spec: &ColumnSpec) {
    let ColumnSpec {
        nullable,
        default,
        generated,
        unique,
        primary_key,
    } = column_spec;

    if let Some(nullable) = nullable {
        w.push_str(if *nullable { " NULL" } else { " NOT NULL" });
    }

    if let Some(default) = default {
        w.push_str(" DEFAULT ");
        match default {
            Expr::Value(_) | Expr::Keyword(_) => write_expr(w, default),
            _ => {
                w.push_str("(");
                write_expr(w, default);
                w.push_str(")");
            }
        }
    }

    if let Some(generated) = generated {
        w.push_str(" GENERATED ALWAYS AS (");
        write_expr(w, &generated.expr);
        w.push_str(")");
        w.push_str(match generated.kind {
            GeneratedColumnKind::Stored => " STORED",
            GeneratedColumnKind::Virtual => " VIRTUAL",
        });
    }

    if *primary_key {
        w.push_str(" PRIMARY KEY");
    }

    if *unique {
        w.push_str(" UNIQUE");
    }
}
