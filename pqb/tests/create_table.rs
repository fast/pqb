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

use insta::assert_snapshot;
use pqb::expr::Expr;
use pqb::index::CreateIndex;
use pqb::table::ColumnDef;
use pqb::table::ColumnType;
use pqb::table::CreateTable;

#[test]
fn create_table_basic() {
    assert_snapshot!(
        CreateTable::new()
            .table("users")
            .column(ColumnDef::new("id").bigint().not_null())
            .column(ColumnDef::new("email").text().not_null())
            .column(ColumnDef::new("nickname").text().null())
            .column(ColumnDef::new("created_at").timestamp_with_time_zone())
            .to_sql(),
        @r#"CREATE TABLE "users" ( "id" bigint NOT NULL, "email" text NOT NULL, "nickname" text NULL, "created_at" timestamp with time zone )"#
    );
}

#[test]
fn create_table_if_not_exists_temporary() {
    assert_snapshot!(
        CreateTable::new()
            .temporary()
            .if_not_exists()
            .table("cache")
            .column(ColumnDef::new("key").text().not_null())
            .column(ColumnDef::new("value").json_binary())
            .to_sql(),
        @r#"CREATE TEMPORARY TABLE IF NOT EXISTS "cache" ( "key" text NOT NULL, "value" jsonb )"#
    );
}

#[test]
fn create_table_primary_key_index() {
    assert_snapshot!(
        CreateTable::new()
            .table("widgets")
            .column(ColumnDef::new("id").int())
            .column(ColumnDef::new("name").text())
            .primary_key(CreateIndex::new().column("id"))
            .to_sql(),
        @r#"CREATE TABLE "widgets" ( "id" integer, "name" text, PRIMARY KEY ("id") )"#
    );
}

#[test]
fn create_table_generated_column() {
    assert_snapshot!(
        CreateTable::new()
            .table("calc")
            .column(ColumnDef::new("a").int())
            .column(ColumnDef::new("b").int())
            .column(
                ColumnDef::new("sum")
                    .int()
                    .generated_as_stored(Expr::column("a").add(Expr::column("b"))),
            )
            .column(
                ColumnDef::new("avg")
                    .int()
                    .generated_as_virtual(Expr::column("sum").div(Expr::value(2))),
            )
            .to_sql(),
        @r#"CREATE TABLE "calc" ( "a" integer, "b" integer, "sum" integer GENERATED ALWAYS AS ("a" + "b") STORED, "avg" integer GENERATED ALWAYS AS ("sum" / 2) VIRTUAL )"#
    );
}

#[test]
fn create_table_all_column_types() {
    assert_snapshot!(
        CreateTable::new()
            .table("all_types")
            .column(ColumnDef::new("col_char").char(4))
            .column(ColumnDef::new("col_varchar").varchar(10))
            .column(ColumnDef::new("col_text").text())
            .column(ColumnDef::new("col_bytea").bytea())
            .column(ColumnDef::new("col_smallint").smallint())
            .column(ColumnDef::new("col_int").int())
            .column(ColumnDef::new("col_bigint").bigint())
            .column(ColumnDef::new("col_float").float())
            .column(ColumnDef::new("col_double").double())
            .column(ColumnDef::new("col_numeric").numeric(10, 2))
            .column(ColumnDef::new("col_smallserial").smallserial())
            .column(ColumnDef::new("col_serial").serial())
            .column(ColumnDef::new("col_bigserial").bigserial())
            .column(ColumnDef::new("col_int4range").int4_range())
            .column(ColumnDef::new("col_int8range").int8_range())
            .column(ColumnDef::new("col_numrange").num_range())
            .column(ColumnDef::new("col_tsrange").ts_range())
            .column(ColumnDef::new("col_tstzrange").ts_tz_range())
            .column(ColumnDef::new("col_daterange").date_range())
            .column(ColumnDef::new("col_datetime").date_time())
            .column(ColumnDef::new("col_timestamp").timestamp())
            .column(ColumnDef::new("col_timestamptz").timestamp_with_time_zone())
            .column(ColumnDef::new("col_time").time())
            .column(ColumnDef::new("col_date").date())
            .column(ColumnDef::new("col_bool").boolean())
            .column(ColumnDef::new("col_json").json())
            .column(ColumnDef::new("col_jsonb").json_binary())
            .column(ColumnDef::new("col_uuid").uuid())
            .column(ColumnDef::new("col_int_array").array_of(ColumnType::Int))
            .to_sql(),
        @r#"CREATE TABLE "all_types" ( "col_char" char(4), "col_varchar" varchar(10), "col_text" text, "col_bytea" bytea, "col_smallint" smallint, "col_int" integer, "col_bigint" bigint, "col_float" real, "col_double" double precision, "col_numeric" numeric(10, 2), "col_smallserial" smallserial, "col_serial" serial, "col_bigserial" bigserial, "col_int4range" int4range, "col_int8range" int8range, "col_numrange" numrange, "col_tsrange" tsrange, "col_tstzrange" tstzrange, "col_daterange" daterange, "col_datetime" timestamp without time zone, "col_timestamp" timestamp, "col_timestamptz" timestamp with time zone, "col_time" time, "col_date" date, "col_bool" bool, "col_json" json, "col_jsonb" jsonb, "col_uuid" uuid, "col_int_array" integer[] )"#
    );
}

#[test]
#[should_panic(expected = "A generated column cannot have a default value.")]
fn create_table_generated_column_with_default_should_panic() {
    let _ = CreateTable::new()
        .table("bad_table")
        .column(
            ColumnDef::new("bad_column")
                .int()
                .default(Expr::value(10))
                .generated_as_stored(Expr::column("bad_column").add(Expr::value(5))),
        )
        .to_sql();
}

#[test]
#[should_panic(expected = "A generated column cannot have a default value.")]
fn create_table_default_with_generated_column_should_panic() {
    let _ = CreateTable::new()
        .table("bad_table")
        .column(
            ColumnDef::new("bad_column")
                .int()
                .generated_as_stored(Expr::column("bad_column").add(Expr::value(5)))
                .default(Expr::value(10)),
        )
        .to_sql();
}
