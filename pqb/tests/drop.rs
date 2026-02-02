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
use pqb::index::DropIndex;
use pqb::schema::DropSchema;
use pqb::table::DropTable;

#[test]
fn drop_index_sql() {
    assert_snapshot!(
        DropIndex::new()
            .index(("public", "idx_users_email"))
            .if_exists()
            .concurrently()
            .cascade()
            .to_sql(),
        @r#"DROP INDEX CONCURRENTLY IF EXISTS "public"."idx_users_email" CASCADE"#
    );
}

#[test]
fn drop_table_sql() {
    assert_snapshot!(
        DropTable::new()
            .tables([("public", "users"), ("public", "accounts")])
            .if_exists()
            .restrict()
            .to_sql(),
        @r#"DROP TABLE IF EXISTS "public"."users", "public"."accounts" RESTRICT"#
    );
}

#[test]
fn drop_schema_sql() {
    assert_snapshot!(
        DropSchema::new()
            .schemas(["public", "analytics"])
            .if_exists()
            .cascade()
            .to_sql(),
        @r#"DROP SCHEMA IF EXISTS "public", "analytics" CASCADE"#
    );
}
