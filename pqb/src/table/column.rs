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

    /// Set default value for the column.
    ///
    /// ## Panics
    /// This method will panic if the column is a generated column.
    pub fn default(mut self, expr: Expr) -> Self {
        if self.spec.generated.is_some() {
            panic!("A generated column cannot have a default value.");
        }
        self.spec.default = Some(expr);
        self
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

    /// Set column type as char with length.
    ///
    /// ## Panics
    /// This method will panic if the `size` is zero.
    pub fn char(mut self, size: u32) -> Self {
        if size == 0 {
            panic!("Character type size must be greater than zero.");
        }

        self.ty = Some(ColumnType::Char(size));
        self
    }

    /// Set column type as varchar with length.
    ///
    /// ## Panics
    /// This method will panic if the `size` is zero.
    pub fn varchar(mut self, size: u32) -> Self {
        if size == 0 {
            panic!("Character type size must be greater than zero.");
        }

        self.ty = Some(ColumnType::Varchar(size));
        self
    }

    /// Set column type as text
    pub fn text(mut self) -> Self {
        self.ty = Some(ColumnType::Text);
        self
    }

    /// Set column type as bytea
    pub fn bytea(mut self) -> Self {
        self.ty = Some(ColumnType::Bytea);
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

    /// Set column type as numeric with precision and scale.
    pub fn numeric(mut self, precision: i32, scale: i32) -> Self {
        if scale > precision {
            panic!("Numeric scale cannot be greater than precision.");
        }
        if precision > 1000 {
            panic!("Numeric precision cannot be greater than 1000.");
        }

        self.ty = Some(ColumnType::Numeric(Some((precision, scale))));
        self
    }

    /// Set column type as unbounded numeric.
    pub fn numeric_unbounded(mut self) -> Self {
        self.ty = Some(ColumnType::Numeric(None));
        self
    }

    /// Set column type as smallserial
    pub fn smallserial(mut self) -> Self {
        self.ty = Some(ColumnType::SmallSerial);
        self
    }

    /// Set column type as serial
    pub fn serial(mut self) -> Self {
        self.ty = Some(ColumnType::Serial);
        self
    }

    /// Set column type as bigserial
    pub fn bigserial(mut self) -> Self {
        self.ty = Some(ColumnType::BigSerial);
        self
    }

    /// Set column type as int4range
    pub fn int4_range(mut self) -> Self {
        self.ty = Some(ColumnType::Int4Range);
        self
    }

    /// Set column type as int8range
    pub fn int8_range(mut self) -> Self {
        self.ty = Some(ColumnType::Int8Range);
        self
    }

    /// Set column type as numrange
    pub fn num_range(mut self) -> Self {
        self.ty = Some(ColumnType::NumRange);
        self
    }

    /// Set column type as tsrange
    pub fn ts_range(mut self) -> Self {
        self.ty = Some(ColumnType::TsRange);
        self
    }

    /// Set column type as tstzrange
    pub fn ts_tz_range(mut self) -> Self {
        self.ty = Some(ColumnType::TsTzRange);
        self
    }

    /// Set column type as daterange
    pub fn date_range(mut self) -> Self {
        self.ty = Some(ColumnType::DateRange);
        self
    }

    /// Set column type as timestamp without time zone.
    pub fn date_time(mut self) -> Self {
        self.ty = Some(ColumnType::DateTime);
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

    /// Set column type as boolean
    pub fn boolean(mut self) -> Self {
        self.ty = Some(ColumnType::Boolean);
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

    /// Set column type as array of the given element type.
    pub fn array_of(mut self, ty: ColumnType) -> Self {
        self.ty = Some(ColumnType::Array(Arc::new(ty)));
        self
    }

    /// Set column as generated with expression and stored storage.
    ///
    /// ## Panics
    /// This method will panic if the column has a default value set.
    pub fn generated_as_stored<E>(mut self, expr: E) -> Self
    where
        E: Into<Expr>,
    {
        if self.spec.default.is_some() {
            panic!("A generated column cannot have a default value.");
        }
        self.spec.generated = Some(GeneratedColumn {
            expr: expr.into(),
            kind: GeneratedColumnKind::Stored,
        });
        self
    }

    /// Set column as generated with expression and virtual storage.
    ///
    /// ## Notice
    /// Before PostgreSQL 18, STORED is the only supported kind and must be specified.
    ///
    /// ## Panics
    /// This method will panic if the column has a default value set.
    pub fn generated_as_virtual<E>(mut self, expr: E) -> Self
    where
        E: Into<Expr>,
    {
        if self.spec.default.is_some() {
            panic!("A generated column cannot have a default value.");
        }
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
    // Character types
    Char(u32),
    Varchar(u32),
    Text,

    // Binary types
    Bytea,

    // Numeric types
    SmallInt,
    Int,
    BigInt,
    Float,
    Double,
    /// NUMERIC(prec, scale)/NUMERIC
    Numeric(Option<(i32, i32)>),

    // Range types
    Int4Range,
    Int8Range,
    NumRange,
    TsRange,
    TsTzRange,
    DateRange,

    // Serial types
    SmallSerial,
    Serial,
    BigSerial,

    // Date/Time types
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
        ColumnType::Char(size) => {
            w.push_str("char(");
            w.push_str(&size.to_string());
            w.push_str(")");
        }
        ColumnType::Varchar(size) => {
            w.push_str("varchar(");
            w.push_str(&size.to_string());
            w.push_str(")");
        }
        ColumnType::Text => w.push_str("text"),

        ColumnType::Bytea => w.push_str("bytea"),

        ColumnType::SmallInt => w.push_str("smallint"),
        ColumnType::Int => w.push_str("integer"),
        ColumnType::BigInt => w.push_str("bigint"),
        ColumnType::Float => w.push_str("real"),
        ColumnType::Double => w.push_str("double precision"),
        ColumnType::Numeric(Some((p, s))) => {
            w.push_str("numeric(");
            w.push_str(&p.to_string());
            w.push_str(", ");
            w.push_str(&s.to_string());
            w.push_str(")");
        }
        ColumnType::Numeric(None) => {
            w.push_str("numeric");
        }

        ColumnType::SmallSerial => w.push_str("smallserial"),
        ColumnType::Serial => w.push_str("serial"),
        ColumnType::BigSerial => w.push_str("bigserial"),

        ColumnType::Int4Range => w.push_str("int4range"),
        ColumnType::Int8Range => w.push_str("int8range"),
        ColumnType::NumRange => w.push_str("numrange"),
        ColumnType::TsRange => w.push_str("tsrange"),
        ColumnType::TsTzRange => w.push_str("tstzrange"),
        ColumnType::DateRange => w.push_str("daterange"),

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
