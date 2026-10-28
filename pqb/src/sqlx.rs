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

use sqlx::Arguments;
use sqlx::SqlSafeStr;
use sqlx::postgres::PgArguments;

use crate::SqlWriterValues;
use crate::value::ArrayType;
use crate::value::Value;

type Error = Box<dyn std::error::Error + Send + Sync>;

impl SqlWriterValues {
    /// Encode parameters for SQLx `query_with` or `query_as_with`.
    ///
    /// Returns an error for integer overflow, heterogeneous arrays, and untyped empty or NULL
    /// arrays. Use `Value::typed_array` or `Value::null_array` when the element type cannot be
    /// inferred.
    /// The returned SQL is ready for SQLx 0.9. This trusts custom SQL fragments supplied to the
    /// builder; values must be passed through expressions, never interpolated into those
    /// fragments.
    pub fn into_sqlx(self) -> Result<(sqlx::SqlStr, PgArguments), Error> {
        let (sql, values) = self.into_parts();
        let mut arguments = PgArguments::default();
        for value in values {
            bind(&mut arguments, value)?;
        }
        Ok((sqlx::AssertSqlSafe(sql).into_sql_str(), arguments))
    }
}

fn bind(arguments: &mut PgArguments, value: Value) -> Result<(), Error> {
    match value {
        Value::TinyInt(v) => arguments.add(v.map(i16::from)),
        Value::TinyUnsigned(v) => arguments.add(v.map(i16::from)),
        Value::SmallUnsigned(v) => arguments.add(v.map(i32::from)),
        Value::Unsigned(v) => arguments.add(v.map(i64::from)),
        Value::BigUnsigned(v) => arguments.add(v.map(i64::try_from).transpose()?),
        Value::Array(values) => {
            let ty = values
                .as_ref()
                .and_then(|v| v.first())
                .and_then(array_type)
                .ok_or(
                    "cannot infer array element type; use Value::typed_array or Value::null_array",
                )?;
            bind_array(arguments, ty, values)
        }
        Value::TypedArray(ty, values) => bind_array(arguments, ty, values),
        Value::Bool(v) => arguments.add(v),
        Value::SmallInt(v) => arguments.add(v),
        Value::Int(v) => arguments.add(v),
        Value::BigInt(v) => arguments.add(v),
        Value::Float(v) => arguments.add(v),
        Value::Double(v) => arguments.add(v),
        Value::String(v) => arguments.add(v),
        Value::Bytes(v) => arguments.add(v),
        #[cfg(feature = "with-json")]
        Value::Json(v) => arguments.add(v.map(|v| *v)),
        #[cfg(feature = "with-uuid")]
        Value::Uuid(v) => arguments.add(v),
        #[cfg(feature = "with-chrono")]
        Value::Date(v) => arguments.add(v),
        #[cfg(feature = "with-chrono")]
        Value::Time(v) => arguments.add(v),
        #[cfg(feature = "with-chrono")]
        Value::Timestamp(v) => arguments.add(v),
        #[cfg(feature = "with-chrono")]
        Value::TimestampWithTimeZone(v) => arguments.add(v),
        #[cfg(feature = "with-rust_decimal")]
        Value::Decimal(v) => arguments.add(v),
    }
}

fn array_type(value: &Value) -> Option<ArrayType> {
    Some(match value {
        Value::TinyInt(_) | Value::TinyUnsigned(_) | Value::SmallInt(_) => ArrayType::SmallInt,
        Value::SmallUnsigned(_) | Value::Int(_) => ArrayType::Int,
        Value::Unsigned(_) | Value::BigUnsigned(_) | Value::BigInt(_) => ArrayType::BigInt,
        Value::Array(_) | Value::TypedArray(..) => return None,
        Value::Bool(_) => ArrayType::Bool,
        Value::Float(_) => ArrayType::Float,
        Value::Double(_) => ArrayType::Double,
        Value::String(_) => ArrayType::Text,
        Value::Bytes(_) => ArrayType::Bytes,
        #[cfg(feature = "with-json")]
        Value::Json(_) => ArrayType::Json,
        #[cfg(feature = "with-uuid")]
        Value::Uuid(_) => ArrayType::Uuid,
        #[cfg(feature = "with-chrono")]
        Value::Date(_) => ArrayType::Date,
        #[cfg(feature = "with-chrono")]
        Value::Time(_) => ArrayType::Time,
        #[cfg(feature = "with-chrono")]
        Value::Timestamp(_) => ArrayType::Timestamp,
        #[cfg(feature = "with-chrono")]
        Value::TimestampWithTimeZone(_) => ArrayType::TimestampWithTimeZone,
        #[cfg(feature = "with-rust_decimal")]
        Value::Decimal(_) => ArrayType::Decimal,
    })
}

fn bind_array(
    arguments: &mut PgArguments,
    ty: ArrayType,
    values: Option<Vec<Value>>,
) -> Result<(), Error> {
    macro_rules! array {
        ($variant:ident, $type:ty) => {{
            let array = values
                .map(|values| {
                    values
                        .into_iter()
                        .map(|value| match value {
                            Value::$variant(value) => Ok(value),
                            _ => Err("array element does not match its declared type".into()),
                        })
                        .collect::<Result<Vec<Option<$type>>, Error>>()
                })
                .transpose()?;
            arguments.add(array)
        }};
    }
    match ty {
        ArrayType::SmallInt => {
            let array = values
                .map(|values| {
                    values
                        .into_iter()
                        .map(|v| match v {
                            Value::SmallInt(v) => Ok(v),
                            Value::TinyInt(v) => Ok(v.map(i16::from)),
                            Value::TinyUnsigned(v) => Ok(v.map(i16::from)),
                            _ => Err("expected smallint array element".into()),
                        })
                        .collect::<Result<Vec<_>, Error>>()
                })
                .transpose()?;
            arguments.add(array)
        }
        ArrayType::Int => {
            let array = values
                .map(|values| {
                    values
                        .into_iter()
                        .map(|v| match v {
                            Value::Int(v) => Ok(v),
                            Value::SmallUnsigned(v) => Ok(v.map(i32::from)),
                            _ => Err("expected integer array element".into()),
                        })
                        .collect::<Result<Vec<_>, Error>>()
                })
                .transpose()?;
            arguments.add(array)
        }
        ArrayType::BigInt => {
            let array = values
                .map(|values| {
                    values
                        .into_iter()
                        .map(|v| match v {
                            Value::BigInt(v) => Ok(v),
                            Value::Unsigned(v) => Ok(v.map(i64::from)),
                            Value::BigUnsigned(v) => Ok(v.map(i64::try_from).transpose()?),
                            _ => Err("expected bigint array element".into()),
                        })
                        .collect::<Result<Vec<_>, Error>>()
                })
                .transpose()?;
            arguments.add(array)
        }
        ArrayType::Bool => array!(Bool, bool),
        ArrayType::Float => array!(Float, f32),
        ArrayType::Double => array!(Double, f64),
        ArrayType::Text => array!(String, String),
        ArrayType::Bytes => array!(Bytes, Vec<u8>),
        #[cfg(feature = "with-uuid")]
        ArrayType::Uuid => array!(Uuid, uuid::Uuid),
        #[cfg(feature = "with-chrono")]
        ArrayType::Date => array!(Date, chrono::NaiveDate),
        #[cfg(feature = "with-chrono")]
        ArrayType::Time => array!(Time, chrono::NaiveTime),
        #[cfg(feature = "with-chrono")]
        ArrayType::Timestamp => array!(Timestamp, chrono::NaiveDateTime),
        #[cfg(feature = "with-chrono")]
        ArrayType::TimestampWithTimeZone => {
            array!(TimestampWithTimeZone, chrono::DateTime<chrono::Utc>)
        }
        #[cfg(feature = "with-rust_decimal")]
        ArrayType::Decimal => array!(Decimal, rust_decimal::Decimal),
        #[cfg(feature = "with-json")]
        ArrayType::Json => {
            let array = values
                .map(|values| {
                    values
                        .into_iter()
                        .map(|v| match v {
                            Value::Json(v) => Ok(v.map(|v| *v)),
                            _ => Err("expected json array element".into()),
                        })
                        .collect::<Result<Vec<_>, Error>>()
                })
                .transpose()?;
            arguments.add(array)
        }
    }
}
