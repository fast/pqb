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

#[cfg(feature = "with-json")]
pub use serde_json::Value as Json;
#[cfg(feature = "with-uuid")]
pub use uuid::Uuid;

use crate::writer::SqlWriter;

/// SQL value variants.
#[derive(Debug, Clone, PartialEq)]
#[expect(missing_docs)]
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
    Array(Option<Vec<Value>>),
    #[cfg(feature = "with-json")]
    Json(Option<Box<Json>>),
    #[cfg(feature = "with-uuid")]
    Uuid(Option<Uuid>),
}

impl Value {
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
#[cfg(feature = "with-uuid")]
type_to_value!(Uuid, Uuid);

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
        | Value::Array(None) => w.push_str("NULL"),
        #[cfg(feature = "with-json")]
        Value::Json(None) => w.push_str("NULL"),
        #[cfg(feature = "with-uuid")]
        Value::Uuid(None) => w.push_str("NULL"),

        Value::Bool(Some(b)) => w.push_str(if *b { "TRUE" } else { "FALSE" }),
        Value::TinyInt(Some(i)) => w.push_fmt(format_args!("{i}")),
        Value::SmallInt(Some(i)) => w.push_fmt(format_args!("{i}")),
        Value::Int(Some(i)) => w.push_fmt(format_args!("{i}")),
        Value::BigInt(Some(i)) => w.push_fmt(format_args!("{i}")),
        Value::TinyUnsigned(Some(u)) => w.push_fmt(format_args!("{u}")),
        Value::SmallUnsigned(Some(u)) => w.push_fmt(format_args!("{u}")),
        Value::Unsigned(Some(u)) => w.push_fmt(format_args!("{u}")),
        Value::BigUnsigned(Some(u)) => w.push_fmt(format_args!("{u}")),
        Value::Float(Some(f)) => w.push_fmt(format_args!("{f}")),
        Value::Double(Some(f)) => w.push_fmt(format_args!("{f}")),
        Value::String(Some(s)) => write_string_value(w, s.as_str()),
        Value::Array(Some(a)) => write_array_value(w, a.as_slice()),
        #[cfg(feature = "with-json")]
        Value::Json(Some(v)) => {
            let value = v.to_string();
            write_string_value(w, &value);
        }
        #[cfg(feature = "with-uuid")]
        Value::Uuid(Some(u)) => w.push_fmt(format_args!("'{u}'")),
    }
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
    if should_escape(value) {
        w.push_str("E'");
    } else {
        w.push_str("'");
    }
    write_escaped_string(w, value);
    w.push_str("'");
}

fn write_escaped_string<W: SqlWriter>(w: &mut W, value: &str) {
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
}

fn should_escape(s: &str) -> bool {
    s.chars().any(|c| match c {
        '\x08' | '\x0C' | '\n' | '\r' | '\t' | '\\' | '\'' | '\0' => true,
        c if c.is_ascii_control() => true,
        _ => false,
    })
}
