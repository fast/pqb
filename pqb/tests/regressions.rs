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

mod common;

use common::ValidateSql;
use pqb::expr::Expr;
use pqb::index::CreateIndex;
use pqb::query::CommonTableExpression;
use pqb::query::Insert;
use pqb::query::OnConflict;
use pqb::query::Returning;
use pqb::query::Select;
use pqb::query::With;
use pqb::table::ColumnDef;
use pqb::table::CreateTable;

#[test]
fn values_cte_binds_before_outer_query() {
    let query = Select::new()
        .with(
            With::new().cte(
                CommonTableExpression::new("input")
                    .column("name")
                    .values(vec![vec!["O'Reilly".into()], vec!["世界".into()]]),
            ),
        )
        .column("name")
        .from("input")
        .and_where(Expr::column("name").ne("ignored"));
    query.to_sql().validate();
    let (sql, values) = query.to_values().into_parts();
    assert_eq!(
        sql.validate(),
        "WITH \"input\" (\"name\") AS (VALUES ($1), ($2)) SELECT \"name\" FROM \"input\" WHERE \"name\" <> $3"
    );
    assert_eq!(
        values,
        vec!["O'Reilly".into(), "世界".into(), "ignored".into()]
    );
}

#[test]
fn default_insert_preserves_conflict_and_returning() {
    let query = Insert::new()
        .into_table("events")
        .default_values()
        .on_conflict(OnConflict::constraint("odd\"key").do_nothing())
        .returning(Returning::all());
    assert_eq!(
        query.to_sql().validate(),
        "INSERT INTO \"events\" DEFAULT VALUES ON CONFLICT ON CONSTRAINT \"odd\"\"key\" DO NOTHING RETURNING *"
    );
    let (sql, values) = query.to_values().into_parts();
    assert_eq!(sql.validate(), query.to_sql());
    assert!(values.is_empty());
}

#[test]
fn ddl_literals_are_not_bind_parameters() {
    let table = CreateTable::new()
        .table("events")
        .column(ColumnDef::new("name").text().default("O'Reilly".into()));
    let (sql, values) = table.to_values().into_parts();
    assert_eq!(sql.validate(), table.to_sql());
    assert!(values.is_empty());

    let index = CreateIndex::new()
        .name("active_names")
        .table("events")
        .unique()
        .column("name")
        .index_where(Expr::column("name").ne("deleted"));
    let (sql, values) = index.to_values().into_parts();
    assert!(sql.clone().validate().starts_with("CREATE UNIQUE INDEX "));
    assert_eq!(sql, index.to_sql());
    assert!(values.is_empty());
}

#[test]
fn truth_tests_do_not_bind_sql_keywords() {
    let nullable = Select::new()
        .expr(Expr::column("value").is(Option::<i32>::None))
        .to_values();
    assert_eq!(nullable.sql(), "SELECT \"value\" IS NULL");
    assert!(nullable.values().is_empty());
    let query = Select::new()
        .column("active")
        .from("users")
        .and_where(Expr::column("active").is(true))
        .and_where(Expr::column("id").eq(42));
    let (sql, values) = query.to_values().into_parts();
    assert_eq!(
        sql.validate(),
        "SELECT \"active\" FROM \"users\" WHERE \"active\" IS TRUE AND \"id\" = $1"
    );
    assert_eq!(values, vec![42.into()]);
}

#[test]
fn non_finite_floats_are_typed_literals() {
    let query = Select::new()
        .expr(f64::NAN)
        .expr(f64::INFINITY)
        .expr(f32::NEG_INFINITY);
    assert_eq!(
        query.to_sql().validate(),
        "SELECT 'NaN'::double precision, 'Infinity'::double precision, '-Infinity'::real"
    );
}

#[test]
fn binary_and_typed_array_literals() {
    use pqb::value::ArrayType;
    use pqb::value::Value;
    let query = Select::new()
        .expr(vec![0_u8, 39, 92, 255])
        .expr(Value::typed_array(ArrayType::Int, Vec::<i32>::new()))
        .expr(Value::null_array(ArrayType::Text));
    assert_eq!(
        query.to_sql().validate(),
        "SELECT E'\\\\x00275cff'::bytea, '{}'::integer[], NULL::text[]"
    );
    let (sql, values) = query.to_values().into_parts();
    assert_eq!(sql.validate(), "SELECT $1, $2, $3");
    assert_eq!(values.len(), 3);
}

#[test]
#[cfg(all(feature = "with-chrono", feature = "with-rust_decimal"))]
fn temporal_and_decimal_literals() {
    use pqb::value::DateTime;
    use pqb::value::Decimal;
    use pqb::value::NaiveDate;
    let date = NaiveDate::from_ymd_opt(2026, 10, 28).unwrap();
    let timestamp = DateTime::parse_from_rfc3339("2026-10-28T12:34:56+08:00").unwrap();
    let query = Select::new()
        .expr(date)
        .expr(timestamp)
        .expr(Decimal::new(12345, 2));
    assert_eq!(
        query.to_sql().validate(),
        "SELECT '2026-10-28'::date, '2026-10-28T04:34:56+00:00'::timestamptz, 123.45"
    );
}

#[test]
fn ddl_rejects_incomplete_actions_and_inapplicable_options() {
    use std::panic::catch_unwind;

    use pqb::index::DropIndex;
    use pqb::table::AlterTable;

    assert!(
        catch_unwind(|| AlterTable::new()
            .table("events")
            .add_column(ColumnDef::new("id"))
            .to_sql())
        .is_err()
    );
    assert!(
        catch_unwind(|| AlterTable::new()
            .table("events")
            .modify_column(ColumnDef::new("id"))
            .to_sql())
        .is_err()
    );
    assert!(
        catch_unwind(|| DropIndex::new()
            .index("idx")
            .concurrently()
            .cascade()
            .to_sql())
        .is_err()
    );
    assert!(
        catch_unwind(|| DropIndex::new()
            .indexes(["one", "two"])
            .concurrently()
            .to_sql())
        .is_err()
    );
    assert!(
        catch_unwind(|| CreateTable::new()
            .table("events")
            .primary_key(CreateIndex::new().expr(Expr::column("id").add(1)))
            .to_sql())
        .is_err()
    );
    assert!(
        catch_unwind(|| CreateTable::new()
            .table("events")
            .primary_key(CreateIndex::new().column("id").concurrently())
            .to_sql())
        .is_err()
    );
}
