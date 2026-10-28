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

use bytes::BytesMut;
use postgres_types::IsNull;
use postgres_types::Kind;
use postgres_types::ToSql;
use postgres_types::Type;

use crate::SqlWriterValues;
use crate::value::ArrayType;
use crate::value::Value;

type Error = Box<dyn std::error::Error + Send + Sync>;

impl SqlWriterValues {
    /// Borrow parameters for `tokio_postgres::Client` or `postgres::Client`.
    ///
    /// Keep this bound query alive until the driver finishes using these references.
    /// The prepared statement or SQL context must determine parameter types. For queries such as
    /// `SELECT $1`, use [`Self::postgres_typed_params`] with the driver's `query_typed` method.
    pub fn postgres_params(&self) -> Vec<&(dyn ToSql + Sync)> {
        self.values()
            .iter()
            .map(|value| value as &(dyn ToSql + Sync))
            .collect()
    }

    /// Borrow parameters together with their PostgreSQL types for the driver's `query_typed`.
    ///
    /// Types come from the Rust values, including typed NULLs and arrays, so a standalone value
    /// does not need a SQL cast. Returns an error for untyped empty or NULL arrays; use
    /// [`Value::typed_array`] or [`Value::null_array`] for those. Encoding errors, including
    /// integer overflow and mismatched array elements, are reported by the driver.
    pub fn postgres_typed_params(&self) -> Result<Vec<(&(dyn ToSql + Sync), Type)>, Error> {
        self.values()
            .iter()
            .map(|value| {
                let ty = match value {
                    Value::Array(values) => values
                        .as_ref()
                        .and_then(|values| values.first())
                        .and_then(ArrayType::of_value)
                        .ok_or("cannot infer array element type; use Value::typed_array or Value::null_array")?
                        .postgres_types().1,
                    Value::TypedArray(ty, _) => ty.postgres_types().1,
                    Value::MultidimensionalArray(array) => array.ty.postgres_types().1,
                    value => ArrayType::of_value(value)
                        .ok_or("cannot infer PostgreSQL parameter type")?
                        .postgres_types().0,
                };
                Ok((value as &(dyn ToSql + Sync), ty))
            })
            .collect()
    }
}

impl ToSql for Value {
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Error> {
        match self {
            Value::TinyInt(v) => v.map(i16::from).to_sql_checked(ty, out),
            Value::TinyUnsigned(v) => v.map(i16::from).to_sql_checked(ty, out),
            Value::SmallUnsigned(v) => v.map(i32::from).to_sql_checked(ty, out),
            Value::Unsigned(v) => v.map(i64::from).to_sql_checked(ty, out),
            Value::BigUnsigned(v) => v.map(i64::try_from).transpose()?.to_sql_checked(ty, out),
            Value::Array(v) => v.to_sql_checked(ty, out),
            Value::TypedArray(element, values) => {
                if !matches!(ty.kind(), Kind::Array(inner) if *inner == element.postgres_types().0)
                {
                    return Err(format!("expected {}[], got {ty}", element.sql_name()).into());
                }
                values.to_sql_checked(ty, out)
            }
            Value::MultidimensionalArray(array) => {
                let element = array.ty.postgres_types().0;
                if !matches!(ty.kind(), Kind::Array(inner) if *inner == element) {
                    return Err(format!("expected {}[], got {ty}", array.ty.sql_name()).into());
                }
                let dimensions = array.dimensions.iter().map(|dimension| {
                    postgres_protocol::types::ArrayDimension {
                        len: dimension.length as i32,
                        lower_bound: dimension.lower_bound,
                    }
                });
                postgres_protocol::types::array_to_sql(
                    dimensions,
                    element.oid(),
                    &array.values,
                    |value, buffer| {
                        Ok(match value.to_sql_checked(&element, buffer)? {
                            IsNull::Yes => postgres_protocol::IsNull::Yes,
                            IsNull::No => postgres_protocol::IsNull::No,
                        })
                    },
                    out,
                )?;
                Ok(IsNull::No)
            }
            Value::Bool(v) => v.to_sql_checked(ty, out),
            Value::SmallInt(v) => v.to_sql_checked(ty, out),
            Value::Int(v) => v.to_sql_checked(ty, out),
            Value::BigInt(v) => v.to_sql_checked(ty, out),
            Value::Float(v) => v.to_sql_checked(ty, out),
            Value::Double(v) => v.to_sql_checked(ty, out),
            Value::String(v) => v.to_sql_checked(ty, out),
            Value::Bytes(v) => v.to_sql_checked(ty, out),
            #[cfg(feature = "with-json")]
            Value::Json(v) => v.to_sql_checked(ty, out),
            #[cfg(feature = "with-uuid")]
            Value::Uuid(v) => v.to_sql_checked(ty, out),
            #[cfg(feature = "with-chrono")]
            Value::Date(v) => v.to_sql_checked(ty, out),
            #[cfg(feature = "with-chrono")]
            Value::Time(v) => v.to_sql_checked(ty, out),
            #[cfg(feature = "with-chrono")]
            Value::Timestamp(v) => v.to_sql_checked(ty, out),
            #[cfg(feature = "with-chrono")]
            Value::TimestampWithTimeZone(v) => v.to_sql_checked(ty, out),
            #[cfg(feature = "with-rust_decimal")]
            Value::Decimal(v) => v.to_sql_checked(ty, out),
        }
    }

    fn accepts(_: &Type) -> bool {
        // The type depends on the variant; to_sql delegates to each concrete type's checked
        // encoder.
        true
    }

    postgres_types::to_sql_checked!();
}

impl ArrayType {
    fn postgres_types(self) -> (Type, Type) {
        match self {
            Self::Bool => (Type::BOOL, Type::BOOL_ARRAY),
            Self::SmallInt => (Type::INT2, Type::INT2_ARRAY),
            Self::Int => (Type::INT4, Type::INT4_ARRAY),
            Self::BigInt => (Type::INT8, Type::INT8_ARRAY),
            Self::Float => (Type::FLOAT4, Type::FLOAT4_ARRAY),
            Self::Double => (Type::FLOAT8, Type::FLOAT8_ARRAY),
            Self::Text => (Type::TEXT, Type::TEXT_ARRAY),
            Self::Bytes => (Type::BYTEA, Type::BYTEA_ARRAY),
            #[cfg(feature = "with-json")]
            Self::Json => (Type::JSONB, Type::JSONB_ARRAY),
            #[cfg(feature = "with-uuid")]
            Self::Uuid => (Type::UUID, Type::UUID_ARRAY),
            #[cfg(feature = "with-chrono")]
            Self::Date => (Type::DATE, Type::DATE_ARRAY),
            #[cfg(feature = "with-chrono")]
            Self::Time => (Type::TIME, Type::TIME_ARRAY),
            #[cfg(feature = "with-chrono")]
            Self::Timestamp => (Type::TIMESTAMP, Type::TIMESTAMP_ARRAY),
            #[cfg(feature = "with-chrono")]
            Self::TimestampWithTimeZone => (Type::TIMESTAMPTZ, Type::TIMESTAMPTZ_ARRAY),
            #[cfg(feature = "with-rust_decimal")]
            Self::Decimal => (Type::NUMERIC, Type::NUMERIC_ARRAY),
        }
    }
}
