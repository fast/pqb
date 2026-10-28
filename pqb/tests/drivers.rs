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

#![cfg(any(feature = "postgres", feature = "sqlx"))]

#[cfg(feature = "sqlx")]
use pqb::query::Select;
use pqb::value::ArrayType;
use pqb::value::Value;

#[test]
#[cfg(feature = "postgres")]
fn postgres_reports_overflow_and_element_type_mismatch() {
    use bytes::BytesMut;
    use postgres_types::ToSql;
    use postgres_types::Type;
    let mut output = BytesMut::new();
    assert!(
        Value::from(u64::MAX)
            .to_sql_checked(&Type::INT8, &mut output)
            .is_err()
    );
    assert!(
        Value::array([Value::from(1), Value::from("wrong")])
            .to_sql_checked(&Type::INT4_ARRAY, &mut output)
            .is_err()
    );
    assert!(
        Value::from(Option::<i32>::None)
            .to_sql_checked(&Type::TEXT, &mut output)
            .is_err()
    );
}

#[test]
#[cfg(feature = "sqlx")]
fn sqlx_reports_overflow_and_ambiguous_arrays() {
    assert!(
        Select::new()
            .expr(u64::MAX)
            .to_values()
            .into_sqlx()
            .is_err()
    );
    assert!(
        Select::new()
            .expr(Value::array(Vec::<i32>::new()))
            .to_values()
            .into_sqlx()
            .is_err()
    );
    assert!(
        Select::new()
            .expr(Value::typed_array(ArrayType::Int, [Value::from("wrong")]))
            .to_values()
            .into_sqlx()
            .is_err()
    );
    assert!(
        Select::new()
            .expr(Value::typed_array(ArrayType::Int, Vec::<i32>::new()))
            .to_values()
            .into_sqlx()
            .is_ok()
    );
}
