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

use pqb::expr::Expr;
use pqb::func::FunctionCall;
use pqb::query::Select;
use pqb::value::ArrayType;
use pqb::value::ArrayValue;
use pqb::value::DateTime;
use pqb::value::Decimal;
use pqb::value::NaiveDate;
use pqb::value::Uuid;
use pqb::value::Value;

use crate::Error;
use crate::database_url;

pub fn value_checks() -> Select {
    let date = NaiveDate::from_ymd_opt(2026, 10, 28).unwrap();
    let timestamp = DateTime::parse_from_rfc3339("2026-10-28T12:34:56+08:00").unwrap();
    let bc = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
    let checks: Vec<(Value, &str, &str)> = vec![
        (true.into(), "boolean", "TRUE"),
        (127_i8.into(), "smallint", "127"),
        (u16::MAX.into(), "integer", "65535"),
        (u32::MAX.into(), "bigint", "4294967295"),
        (42_u64.into(), "bigint", "42"),
        ("O'Reilly\\世界".into(), "text", "E'O''Reilly\\\\世界'"),
        (
            vec![0_u8, 39, 92, 255].into(),
            "bytea",
            "decode('00275cff', 'hex')",
        ),
        (date.into(), "date", "DATE '2026-10-28'"),
        (bc.into(), "date", "DATE '0001-01-01 BC'"),
        (
            NaiveDate::from_ymd_opt(10000, 1, 1)
                .unwrap()
                .and_hms_opt(4, 34, 56)
                .unwrap()
                .into(),
            "timestamp",
            "TIMESTAMP '10000-01-01 04:34:56'",
        ),
        (
            bc.and_hms_opt(4, 34, 56).unwrap().and_utc().into(),
            "timestamptz",
            "TIMESTAMPTZ '0001-01-01 04:34:56+00 BC'",
        ),
        (
            date.and_hms_opt(4, 34, 56).unwrap().into(),
            "timestamp",
            "TIMESTAMP '2026-10-28 04:34:56'",
        ),
        (
            date.and_hms_opt(4, 34, 56).unwrap().time().into(),
            "time",
            "TIME '04:34:56'",
        ),
        (
            timestamp.into(),
            "timestamptz",
            "TIMESTAMPTZ '2026-10-28 04:34:56+00'",
        ),
        (Decimal::new(12345, 2).into(), "numeric", "123.45"),
        (
            serde_json::json!({"name": "O'Reilly", "active": true}).into(),
            "jsonb",
            "'{\"active\":true,\"name\":\"O''Reilly\"}'::jsonb",
        ),
        (
            Uuid::nil().into(),
            "uuid",
            "'00000000-0000-0000-0000-000000000000'::uuid",
        ),
        (
            Value::typed_array(ArrayType::Int, [Some(1), None, Some(3)]),
            "integer[]",
            "ARRAY[1, NULL, 3]",
        ),
        (
            Value::typed_array(ArrayType::Text, Vec::<String>::new()),
            "text[]",
            "ARRAY[]::text[]",
        ),
        (
            Value::null_array(ArrayType::Int),
            "integer[]",
            "NULL::integer[]",
        ),
        (
            ArrayValue::new(ArrayType::Int, [2, 2], [None, None, Some(3), Some(4)])
                .unwrap()
                .into(),
            "integer[]",
            "ARRAY[[NULL,NULL],[3,4]]::integer[]",
        ),
        (
            ArrayValue::new(
                ArrayType::Int,
                [(2, 0), (2, -1)],
                [Some(1), None, Some(3), Some(4)],
            )
            .unwrap()
            .into(),
            "integer[]",
            "'[0:1][-1:0]={{1,NULL},{3,4}}'::integer[]",
        ),
        (
            ArrayValue::new(
                ArrayType::Text,
                [2, 1, 2],
                [Some("a,b"), Some(r"O'Reilly\world"), Some("NULL"), None],
            )
            .unwrap()
            .into(),
            "text[]",
            r"ARRAY[[['a,b', E'O''Reilly\\world']], [['NULL', NULL]]]",
        ),
        (
            ArrayValue::new(ArrayType::Bytes, [2, 1], [vec![0_u8, 1, 39, 255], vec![]])
                .unwrap()
                .into(),
            "bytea[]",
            "ARRAY[[decode('000127ff', 'hex')], [decode('', 'hex')]]",
        ),
        (
            ArrayValue::new(
                ArrayType::Json,
                [1, 2],
                [Some(serde_json::json!({"key": "a,b"})), None],
            )
            .unwrap()
            .into(),
            "jsonb[]",
            "ARRAY[[jsonb_build_object('key', 'a,b'), NULL]]",
        ),
        (
            ArrayValue::new(ArrayType::Int, [2, 0, 3], Vec::<i32>::new())
                .unwrap()
                .into(),
            "integer[]",
            "'{}'::integer[]",
        ),
        (Option::<i32>::None.into(), "integer", "NULL::integer"),
        (
            f64::NAN.into(),
            "double precision",
            "'NaN'::double precision",
        ),
    ];
    Select::new()
        .expr(Expr::value(1).is_in(Vec::<i32>::new()).not())
        .expr(
            Expr::value(64)
                .cast_as("integer")
                .strictly_right_of(
                    Expr::value(1)
                        .cast_as("integer")
                        .binary(pqb::expr::BinaryOp::RShift, 1),
                )
                .eq(64),
        )
        .exprs(checks.into_iter().map(|(value, ty, expected)| {
            Expr::value(value)
                .cast_as(ty)
                .is_not_distinct_from(Expr::custom(expected))
        }))
}

#[tokio::test]
#[ignore = "requires PQB_TEST_DATABASE_URL"]
async fn postgres_binds_values_without_sql_type_context() -> Result<(), Error> {
    let (client, connection) =
        tokio_postgres::connect(&database_url(), tokio_postgres::NoTls).await?;
    let connection = tokio::spawn(connection);
    let matrix = ArrayValue::new(ArrayType::Int, [2, 2], [1, 2, 3, 4])?;
    let bound = Select::new()
        .expr(42)
        .expr(Option::<i32>::None)
        .expr(Value::typed_array(ArrayType::Int, Vec::<i32>::new()))
        .expr(Value::array([1, 2]))
        .expr(FunctionCall::custom("array_dims", [Value::from(matrix)]))
        .to_values();
    let rows = client
        .query_typed(bound.sql(), &bound.postgres_typed_params()?)
        .await?;
    let row = &rows[0];
    assert_eq!(row.get::<_, i32>(0), 42);
    assert_eq!(row.get::<_, Option<i32>>(1), None);
    assert!(row.get::<_, Vec<i32>>(2).is_empty());
    assert_eq!(row.get::<_, Vec<i32>>(3), [1, 2]);
    assert_eq!(row.get::<_, String>(4), "[1:2][1:2]");
    drop(client);
    connection.await??;
    Ok(())
}
