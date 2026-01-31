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
use pqb::query::Select;
use pqb::types::Asterisk;
use pqb::types::Iden;

#[test]
fn iden_escape_detection() {
    assert!(Iden::new("alpha_1").is_escaped());
    assert!(Iden::new("_alpha").is_escaped());
    assert!(!Iden::new("1alpha").is_escaped());
    assert!(!Iden::new("has space").is_escaped());
    assert!(!Iden::new("has-dash").is_escaped());
}

#[test]
fn iden_rendering() {
    assert_snapshot!(
        Select::new().column(Iden::new("simple")).to_sql(),
        @r#"SELECT "simple""#
    );
    assert_snapshot!(
        Select::new().column(Iden::new("has space")).to_sql(),
        @r#"SELECT "has space""#
    );
    assert_snapshot!(
        Select::new()
            .column(Iden::new(r#"has"quote"#))
            .to_sql(),
        @r#"SELECT "has""quote""#
    );
}

#[test]
fn qualified_names_rendering() {
    assert_snapshot!(
        Select::new().column("id").from(("audit", "events")).to_sql(),
        @r#"SELECT "id" FROM "audit"."events""#
    );
    assert_snapshot!(
        Select::new()
            .column("id")
            .from(("analytics", "audit", "events"))
            .to_sql(),
        @r#"SELECT "id" FROM "analytics"."audit"."events""#
    );
    assert_snapshot!(
        Select::new()
            .column(("audit", "events", "id"))
            .from(("audit", "events"))
            .to_sql(),
        @r#"SELECT "audit"."events"."id" FROM "audit"."events""#
    );
    assert_snapshot!(
        Select::new()
            .column(("analytics", "audit", "events", Asterisk))
            .from(("analytics", "audit", "events"))
            .to_sql(),
        @r#"SELECT "analytics"."audit"."events".* FROM "analytics"."audit"."events""#
    );
}
