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
