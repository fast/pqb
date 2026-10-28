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

//! Container for all SQL value types.

#[cfg(feature = "with-chrono")]
pub use chrono::DateTime;
#[cfg(feature = "with-chrono")]
pub use chrono::FixedOffset;
#[cfg(feature = "with-chrono")]
pub use chrono::NaiveDate;
#[cfg(feature = "with-chrono")]
pub use chrono::NaiveDateTime;
#[cfg(feature = "with-chrono")]
pub use chrono::NaiveTime;
#[cfg(feature = "with-chrono")]
pub use chrono::Utc;
#[cfg(feature = "with-rust_decimal")]
pub use rust_decimal::Decimal;
#[cfg(feature = "with-json")]
pub use serde_json::Value as Json;
#[cfg(feature = "with-uuid")]
pub use uuid::Uuid;

use crate::writer::SqlWriter;

mod array;
pub use self::array::ArrayDimension;
pub use self::array::ArrayError;
pub use self::array::ArrayValue;

/// SQL value variants.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum Value {
    Bool(Option<bool>),
    TinyInt(Option<i8>),
    SmallInt(Option<i16>),
    Int(Option<i32>),
    BigInt(Option<i64>),
    TinyUnsigned(Option<u8>),
    SmallUnsigned(Option<u16>),
    Unsigned(Option<u32>),
    BigUnsigned(Option<u64>),
    Float(Option<f32>),
    Double(Option<f64>),
    String(Option<String>),
    Bytes(Option<Vec<u8>>),
    Array(Option<Vec<Value>>),
    TypedArray(ArrayType, Option<Vec<Value>>),
    MultidimensionalArray(ArrayValue),
    #[cfg(feature = "with-json")]
    Json(Option<Box<Json>>),
    #[cfg(feature = "with-uuid")]
    Uuid(Option<Uuid>),
    #[cfg(feature = "with-chrono")]
    Date(Option<NaiveDate>),
    #[cfg(feature = "with-chrono")]
    Time(Option<NaiveTime>),
    #[cfg(feature = "with-chrono")]
    Timestamp(Option<NaiveDateTime>),
    #[cfg(feature = "with-chrono")]
    TimestampWithTimeZone(Option<DateTime<Utc>>),
    #[cfg(feature = "with-rust_decimal")]
    Decimal(Option<Decimal>),
}

impl Value {
    pub(crate) fn is_null(&self) -> bool {
        match self {
            Self::Bool(value) => value.is_none(),
            Self::TinyInt(value) => value.is_none(),
            Self::SmallInt(value) => value.is_none(),
            Self::Int(value) => value.is_none(),
            Self::BigInt(value) => value.is_none(),
            Self::TinyUnsigned(value) => value.is_none(),
            Self::SmallUnsigned(value) => value.is_none(),
            Self::Unsigned(value) => value.is_none(),
            Self::BigUnsigned(value) => value.is_none(),
            Self::Float(value) => value.is_none(),
            Self::Double(value) => value.is_none(),
            Self::String(value) => value.is_none(),
            Self::Bytes(value) => value.is_none(),
            Self::Array(value) => value.is_none(),
            #[cfg(feature = "with-json")]
            Self::Json(value) => value.is_none(),
            #[cfg(feature = "with-uuid")]
            Self::Uuid(value) => value.is_none(),
            #[cfg(feature = "with-chrono")]
            Self::Date(value) => value.is_none(),
            #[cfg(feature = "with-chrono")]
            Self::Time(value) => value.is_none(),
            #[cfg(feature = "with-chrono")]
            Self::Timestamp(value) => value.is_none(),
            #[cfg(feature = "with-chrono")]
            Self::TimestampWithTimeZone(value) => value.is_none(),
            #[cfg(feature = "with-rust_decimal")]
            Self::Decimal(value) => value.is_none(),
            Self::TypedArray(_, value) => value.is_none(),
            Self::MultidimensionalArray(_) => false,
        }
    }

    /// Create a typed PostgreSQL array, including empty arrays and arrays containing nulls.
    ///
    /// Drivers reject elements whose types do not match the declared element type.
    pub fn typed_array<I, V>(ty: ArrayType, values: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<Value>,
    {
        Self::TypedArray(ty, Some(values.into_iter().map(Into::into).collect()))
    }

    /// Create a NULL array with an explicit element type.
    pub fn null_array(ty: ArrayType) -> Self {
        Self::TypedArray(ty, None)
    }

    /// Create a new array value.
    pub fn array<T, I>(values: I) -> Value
    where
        T: Into<Value>,
        I: IntoIterator<Item = T>,
    {
        Value::Array(Some(values.into_iter().map(|e| e.into()).collect()))
    }
}

macro_rules! type_to_value {
    ( $type: ty, $name: ident ) => {
        impl From<$type> for Value {
            fn from(x: $type) -> Value {
                Value::$name(Some(x))
            }
        }

        impl Nullable for $type {
            fn null() -> Value {
                Value::$name(None)
            }
        }
    };
}

type_to_value!(bool, Bool);
type_to_value!(i8, TinyInt);
type_to_value!(i16, SmallInt);
type_to_value!(i32, Int);
type_to_value!(i64, BigInt);
type_to_value!(u8, TinyUnsigned);
type_to_value!(u16, SmallUnsigned);
type_to_value!(u32, Unsigned);
type_to_value!(u64, BigUnsigned);
type_to_value!(f32, Float);
type_to_value!(f64, Double);
type_to_value!(String, String);
type_to_value!(Vec<u8>, Bytes);
#[cfg(feature = "with-uuid")]
type_to_value!(Uuid, Uuid);
#[cfg(feature = "with-chrono")]
type_to_value!(NaiveDate, Date);
#[cfg(feature = "with-chrono")]
type_to_value!(NaiveTime, Time);
#[cfg(feature = "with-chrono")]
type_to_value!(NaiveDateTime, Timestamp);
#[cfg(feature = "with-chrono")]
type_to_value!(DateTime<Utc>, TimestampWithTimeZone);
#[cfg(feature = "with-rust_decimal")]
type_to_value!(Decimal, Decimal);

#[cfg(feature = "with-chrono")]
impl From<DateTime<FixedOffset>> for Value {
    fn from(value: DateTime<FixedOffset>) -> Self {
        Self::TimestampWithTimeZone(Some(value.with_timezone(&Utc)))
    }
}

#[cfg(feature = "with-chrono")]
impl Nullable for DateTime<FixedOffset> {
    fn null() -> Value {
        Value::TimestampWithTimeZone(None)
    }
}

impl From<&[u8]> for Value {
    fn from(value: &[u8]) -> Self {
        Self::Bytes(Some(value.to_vec()))
    }
}

impl Nullable for &[u8] {
    fn null() -> Value {
        Value::Bytes(None)
    }
}

#[allow(unused_macros)]
macro_rules! type_to_box_value {
    ( $type: ty, $name: ident, $col_type: expr ) => {
        impl From<$type> for Value {
            fn from(x: $type) -> Value {
                Value::$name(Some(Box::new(x)))
            }
        }

        impl Nullable for $type {
            fn null() -> Value {
                Value::$name(None)
            }
        }
    };
}

#[cfg(feature = "with-json")]
type_to_box_value!(Json, Json, Json);

impl From<&str> for Value {
    fn from(x: &str) -> Value {
        Value::String(Some(x.to_owned()))
    }
}

impl From<&String> for Value {
    fn from(x: &String) -> Value {
        Value::String(Some(x.clone()))
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value> + Nullable,
{
    fn from(x: Option<T>) -> Value {
        match x {
            Some(v) => v.into(),
            None => T::null(),
        }
    }
}

trait Nullable {
    fn null() -> Value;
}

impl Nullable for &str {
    fn null() -> Value {
        Value::String(None)
    }
}

pub(crate) fn write_value<W: SqlWriter>(w: &mut W, value: &Value) {
    match value {
        Value::Bool(None)
        | Value::TinyInt(None)
        | Value::SmallInt(None)
        | Value::Int(None)
        | Value::BigInt(None)
        | Value::TinyUnsigned(None)
        | Value::SmallUnsigned(None)
        | Value::Unsigned(None)
        | Value::BigUnsigned(None)
        | Value::Float(None)
        | Value::Double(None)
        | Value::String(None)
        | Value::Bytes(None)
        | Value::Array(None) => w.push_str("NULL"),
        #[cfg(feature = "with-json")]
        Value::Json(None) => w.push_str("NULL"),
        #[cfg(feature = "with-uuid")]
        Value::Uuid(None) => w.push_str("NULL"),
        #[cfg(feature = "with-chrono")]
        Value::Date(None)
        | Value::Time(None)
        | Value::Timestamp(None)
        | Value::TimestampWithTimeZone(None) => w.push_str("NULL"),
        #[cfg(feature = "with-rust_decimal")]
        Value::Decimal(None) => w.push_str("NULL"),

        Value::Bool(Some(b)) => w.push_str(if *b { "TRUE" } else { "FALSE" }),
        Value::TinyInt(Some(i)) => w.push_fmt(format_args!("{i}")),
        Value::SmallInt(Some(i)) => w.push_fmt(format_args!("{i}")),
        Value::Int(Some(i)) => w.push_fmt(format_args!("{i}")),
        Value::BigInt(Some(i)) => w.push_fmt(format_args!("{i}")),
        Value::TinyUnsigned(Some(u)) => w.push_fmt(format_args!("{u}")),
        Value::SmallUnsigned(Some(u)) => w.push_fmt(format_args!("{u}")),
        Value::Unsigned(Some(u)) => w.push_fmt(format_args!("{u}")),
        Value::BigUnsigned(Some(u)) => w.push_fmt(format_args!("{u}")),
        Value::Float(Some(f)) if !f.is_finite() => write_special_float(w, f64::from(*f), "real"),
        Value::Double(Some(f)) if !f.is_finite() => write_special_float(w, *f, "double precision"),
        Value::Float(Some(f)) => w.push_fmt(format_args!("{f}")),
        Value::Double(Some(f)) => w.push_fmt(format_args!("{f}")),
        Value::String(Some(s)) => write_string_value(w, s.as_str()),
        Value::Array(Some(a)) => write_array_value(w, a.as_slice()),
        Value::MultidimensionalArray(array) => array::write_array(w, array),
        Value::TypedArray(ty, values) => {
            if let Some(values) = values {
                write_array_value(w, values);
            } else {
                w.push_str("NULL");
            }
            w.push_str("::");
            w.push_str(ty.sql_name());
            w.push_str("[]");
        }
        Value::Bytes(Some(bytes)) => {
            w.push_str("E'\\\\x");
            for byte in bytes {
                w.push_fmt(format_args!("{byte:02x}"));
            }
            w.push_str("'::bytea");
        }
        #[cfg(feature = "with-json")]
        Value::Json(Some(v)) => {
            let value = v.to_string();
            write_string_value(w, &value);
        }
        #[cfg(feature = "with-uuid")]
        Value::Uuid(Some(u)) => w.push_fmt(format_args!("'{u}'")),
        #[cfg(feature = "with-chrono")]
        Value::Date(Some(v)) => write_calendar_value(w, *v, None, false),
        #[cfg(feature = "with-chrono")]
        Value::Time(Some(v)) => w.push_fmt(format_args!("'{}'::time", v.format("%H:%M:%S%.f"))),
        #[cfg(feature = "with-chrono")]
        Value::Timestamp(Some(v)) => write_calendar_value(w, v.date(), Some(v.time()), false),
        #[cfg(feature = "with-chrono")]
        Value::TimestampWithTimeZone(Some(v)) => {
            write_calendar_value(w, v.date_naive(), Some(v.time()), true)
        }
        #[cfg(feature = "with-rust_decimal")]
        Value::Decimal(Some(v)) => w.push_fmt(format_args!("{v}")),
    }
}

#[cfg(feature = "with-chrono")]
fn write_calendar_value<W: SqlWriter>(
    w: &mut W,
    date: NaiveDate,
    time: Option<NaiveTime>,
    timezone: bool,
) {
    use chrono::Datelike;

    // PostgreSQL has no year zero and rejects chrono's leading '+' for years beyond 9999.
    let (ce, year) = date.year_ce();
    w.push_fmt(format_args!(
        "'{year:04}-{:02}-{:02}",
        date.month(),
        date.day()
    ));
    if let Some(time) = time {
        w.push_char(if timezone { 'T' } else { ' ' });
        w.push_fmt(format_args!("{}", time.format("%H:%M:%S%.f")));
    }
    if timezone {
        w.push_str("+00:00");
    }
    if !ce {
        w.push_str(" BC");
    }
    w.push_str(if timezone {
        "'::timestamptz"
    } else if time.is_some() {
        "'::timestamp"
    } else {
        "'::date"
    });
}

fn write_array_value<W: SqlWriter>(w: &mut W, values: &[Value]) {
    if values.is_empty() {
        w.push_str("'{}'");
    } else {
        w.push_str("ARRAY [");

        let mut it = values.iter();

        if let Some(element) = it.next() {
            write_value(w, element);
        }

        for element in it {
            w.push_str(",");
            write_value(w, element);
        }
        w.push_str("]");
    }
}

fn write_string_value<W: SqlWriter>(w: &mut W, value: &str) {
    assert!(
        !value.contains('\0'),
        "PostgreSQL text cannot contain NUL bytes"
    );
    if should_escape(value) {
        write_string_escaped(w, value)
    } else {
        w.push_str("'");
        w.push_str(value);
        w.push_str("'");
    }
}

fn write_special_float<W: SqlWriter>(w: &mut W, value: f64, ty: &str) {
    let literal = if value.is_nan() {
        "NaN"
    } else if value.is_sign_positive() {
        "Infinity"
    } else {
        "-Infinity"
    };
    w.push_fmt(format_args!("'{literal}'::{ty}"));
}

fn write_string_escaped<W: SqlWriter>(w: &mut W, value: &str) {
    w.push_str("E'");
    for c in value.chars() {
        match c {
            '\x08' => w.push_str(r"\b"),
            '\x0C' => w.push_str(r"\f"),
            '\n' => w.push_str(r"\n"),
            '\r' => w.push_str(r"\r"),
            '\t' => w.push_str(r"\t"),
            '\\' => w.push_str(r"\\"),
            '\'' => w.push_str(r"\'"),
            '\0' => w.push_str(r"\0"),
            c if c.is_ascii_control() => {
                let escaped_control_char = format!(r"\{:03o}", c as u32);
                w.push_str(&escaped_control_char);
            }
            c => w.push_char(c),
        }
    }
    w.push_str("'");
}

fn should_escape(s: &str) -> bool {
    s.chars().any(|c| match c {
        '\x08' | '\x0C' | '\n' | '\r' | '\t' | '\\' | '\'' | '\0' => true,
        c if c.is_ascii_control() => true,
        _ => false,
    })
}

/// PostgreSQL element types for typed arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum ArrayType {
    Bool,
    SmallInt,
    Int,
    BigInt,
    Float,
    Double,
    Text,
    Bytes,
    #[cfg(feature = "with-json")]
    Json,
    #[cfg(feature = "with-uuid")]
    Uuid,
    #[cfg(feature = "with-chrono")]
    Date,
    #[cfg(feature = "with-chrono")]
    Time,
    #[cfg(feature = "with-chrono")]
    Timestamp,
    #[cfg(feature = "with-chrono")]
    TimestampWithTimeZone,
    #[cfg(feature = "with-rust_decimal")]
    Decimal,
}

impl ArrayType {
    pub(crate) fn sql_name(self) -> &'static str {
        match self {
            Self::Bool => "boolean",
            Self::SmallInt => "smallint",
            Self::Int => "integer",
            Self::BigInt => "bigint",
            Self::Float => "real",
            Self::Double => "double precision",
            Self::Text => "text",
            Self::Bytes => "bytea",
            #[cfg(feature = "with-json")]
            Self::Json => "jsonb",
            #[cfg(feature = "with-uuid")]
            Self::Uuid => "uuid",
            #[cfg(feature = "with-chrono")]
            Self::Date => "date",
            #[cfg(feature = "with-chrono")]
            Self::Time => "time",
            #[cfg(feature = "with-chrono")]
            Self::Timestamp => "timestamp",
            #[cfg(feature = "with-chrono")]
            Self::TimestampWithTimeZone => "timestamptz",
            #[cfg(feature = "with-rust_decimal")]
            Self::Decimal => "numeric",
        }
    }
}
