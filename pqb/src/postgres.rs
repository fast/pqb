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
    pub fn postgres_params(&self) -> Vec<&(dyn ToSql + Sync)> {
        self.values()
            .iter()
            .map(|value| value as &(dyn ToSql + Sync))
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
                if !matches!(ty.kind(), Kind::Array(inner) if *inner == element.postgres_type()) {
                    return Err(format!("expected {}[], got {ty}", element.sql_name()).into());
                }
                values.to_sql_checked(ty, out)
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
    fn postgres_type(self) -> Type {
        match self {
            Self::Bool => Type::BOOL,
            Self::SmallInt => Type::INT2,
            Self::Int => Type::INT4,
            Self::BigInt => Type::INT8,
            Self::Float => Type::FLOAT4,
            Self::Double => Type::FLOAT8,
            Self::Text => Type::TEXT,
            Self::Bytes => Type::BYTEA,
            #[cfg(feature = "with-json")]
            Self::Json => Type::JSONB,
            #[cfg(feature = "with-uuid")]
            Self::Uuid => Type::UUID,
            #[cfg(feature = "with-chrono")]
            Self::Date => Type::DATE,
            #[cfg(feature = "with-chrono")]
            Self::Time => Type::TIME,
            #[cfg(feature = "with-chrono")]
            Self::Timestamp => Type::TIMESTAMP,
            #[cfg(feature = "with-chrono")]
            Self::TimestampWithTimeZone => Type::TIMESTAMPTZ,
            #[cfg(feature = "with-rust_decimal")]
            Self::Decimal => Type::NUMERIC,
        }
    }
}
