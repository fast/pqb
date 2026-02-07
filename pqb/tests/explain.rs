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

use insta::assert_snapshot;
use pqb::query::Explain;
use pqb::query::Select;

use crate::common::ValidateSQL;

#[test]
fn explain_postgres_select_with_options() {
    assert_snapshot!(
        Explain::new()
            .analyze()
            .verbose(false)
            .costs(true)
            .settings(false)
            .generic_plan(true)
            .buffers(true)
            .serialize_text()
            .wal(true)
            .timing(false)
            .summary(true)
            .memory(true)
            .format_json()
            .statement(Select::new().column("character").from("character"))
            .to_sql()
            .validate(),
        @r#"EXPLAIN (ANALYZE, VERBOSE 0, COSTS, SETTINGS 0, GENERIC_PLAN, BUFFERS, SERIALIZE TEXT, WAL, TIMING 0, SUMMARY, MEMORY, FORMAT JSON) SELECT "character" FROM "character""#
    );
}

#[test]
fn explain_postgres_serialize_text() {
    assert_snapshot!(
        Explain::new()
            .serialize_text()
            .statement(Select::new().column("character").from("character"))
            .to_sql()
            .validate(),
        @r#"EXPLAIN (SERIALIZE TEXT) SELECT "character" FROM "character""#
    );
}

#[test]
fn explain_postgres_serialize_binary() {
    assert_snapshot!(
        Explain::new()
            .serialize_binary()
            .statement(Select::new().column("character").from("character"))
            .to_sql()
            .validate(),
        @r#"EXPLAIN (SERIALIZE BINARY) SELECT "character" FROM "character""#
    );
}

#[test]
fn explain_postgres_serialize_none() {
    assert_snapshot!(
        Explain::new()
            .serialize_none()
            .statement(Select::new().column("character").from("character"))
            .to_sql()
            .validate(),
        @r#"EXPLAIN (SERIALIZE NONE) SELECT "character" FROM "character""#
    );
}
