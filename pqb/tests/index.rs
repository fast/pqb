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

#[test]
fn create_index_gist_with_options() {
    assert_snapshot!(
        CreateIndex::new()
            .table("spatial")
            .column("geom")
            .gist()
            .with_option("fillfactor", 80)
            .with_option("buffering", "auto")
            .to_sql(),
        @r#"CREATE INDEX ON "spatial" USING gist ("geom") WITH ("fillfactor" = 80, "buffering" = 'auto')"#
    );
}

#[test]
fn create_index_brin_with_options() {
    assert_snapshot!(
        CreateIndex::new()
            .table("events")
            .column("created_at")
            .brin()
            .with_options([("pages_per_range", Expr::value(32)), ("autosummarize", Expr::value(true))])
            .to_sql(),
        @r#"CREATE INDEX ON "events" USING brin ("created_at") WITH ("pages_per_range" = 32, "autosummarize" = TRUE)"#
    );
}

#[test]
fn create_index_hash() {
    assert_snapshot!(
        CreateIndex::new()
            .table("tokens")
            .column("value")
            .hash()
            .to_sql(),
        @r#"CREATE INDEX ON "tokens" USING hash ("value")"#
    );
}

#[test]
fn create_index_named() {
    assert_snapshot!(
        CreateIndex::new()
            .name("idx_tokens_value")
            .table("tokens")
            .column("value")
            .to_sql(),
        @r#"CREATE INDEX "idx_tokens_value" ON "tokens" ("value")"#
    );
}

#[test]
fn create_index_if_not_exists_named() {
    assert_snapshot!(
        CreateIndex::new()
            .name("idx_events_created_at_brin")
            .table("events")
            .column("created_at")
            .brin()
            .if_not_exists()
            .to_sql(),
        @r#"CREATE INDEX IF NOT EXISTS "idx_events_created_at_brin" ON "events" USING brin ("created_at")"#
    );
}

#[test]
fn create_index_named_with_options() {
    assert_snapshot!(
        CreateIndex::new()
            .name("idx_spatial_geom_gist")
            .table("spatial")
            .column("geom")
            .gist()
            .with_option("fillfactor", 90)
            .with_option("buffering", "auto")
            .to_sql(),
        @r#"CREATE INDEX "idx_spatial_geom_gist" ON "spatial" USING gist ("geom") WITH ("fillfactor" = 90, "buffering" = 'auto')"#
    );
}

#[test]
fn create_index_if_not_exists_custom_method() {
    assert_snapshot!(
        CreateIndex::new()
            .name("idx_tokens_value_hnsw")
            .table("tokens")
            .column("value")
            .if_not_exists()
            .using("hnsw")
            .to_sql(),
        @r#"CREATE INDEX IF NOT EXISTS "idx_tokens_value_hnsw" ON "tokens" USING hnsw ("value")"#
    );
}

#[test]
fn create_index_include_columns() {
    assert_snapshot!(
        CreateIndex::new()
            .name("idx_orders_customer")
            .table("orders")
            .column("customer_id")
            .include_columns(["id", "created_at"])
            .to_sql(),
        @r#"CREATE INDEX "idx_orders_customer" ON "orders" ("customer_id") INCLUDE ("id", "created_at")"#
    );
}

#[test]
fn create_index_partial() {
    assert_snapshot!(
        CreateIndex::new()
            .name("idx_sessions_active")
            .table("sessions")
            .column("user_id")
            .where_(Expr::column("expires_at").gt(Expr::current_timestamp()))
            .to_sql(),
        @r#"CREATE INDEX "idx_sessions_active" ON "sessions" ("user_id") WHERE "expires_at" > CURRENT_TIMESTAMP"#
    );
}

#[test]
fn create_index_concurrently() {
    assert_snapshot!(
        CreateIndex::new()
            .name("idx_orders_customer")
            .table("orders")
            .column("customer_id")
            .concurrently()
            .if_not_exists()
            .to_sql(),
        @r#"CREATE INDEX CONCURRENTLY IF NOT EXISTS "idx_orders_customer" ON "orders" ("customer_id")"#
    );
}
