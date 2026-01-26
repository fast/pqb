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
use pqb::query::Select;

#[test]
fn select_0() {
    assert_snapshot!(Select::new().expr(Expr::value(1)).to_sql(), @"SELECT 1");
    assert_snapshot!(
        Select::new()
            .expr(Expr::column("n"))
            .from("tbl")
            .and_where(Expr::column("region").eq(Expr::value("CN")))
            .to_sql(),
        @r#"SELECT "n" FROM "tbl" WHERE "region" = 'CN'"#
    );
}

#[test]
fn select_1() {
    assert_snapshot!(
        Select::new()
            .columns(["character", "size_w", "size_h"])
            .from("character")
            .limit(10)
            .offset(100)
            .to_sql(),
        @r#"SELECT "character", "size_w", "size_h" FROM "character" LIMIT 10 OFFSET 100"#
    );
}

#[test]
fn select_2() {
    assert_snapshot!(
        Select::new()
            .columns(["character", "size_w", "size_h"])
            .from("character")
            .and_where(Expr::column("size_w").eq(3))
            .to_sql(),
        @r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" = 3"#
    );
}

#[test]
fn select_3() {
    assert_snapshot!(
        Select::new()
            .columns(["character", "size_w", "size_h"])
            .from("character")
            .and_where(Expr::column("size_w").eq(3))
            .and_where(Expr::column("size_h").eq(4))
            .to_sql(),
        @r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" = 3 AND "size_h" = 4"#
    );
}

#[test]
fn select_4() {
    assert_snapshot!(
        Select::new()
            .columns(["aspect"])
            .from_subquery(
                Select::new().columns(["image", "aspect"]).from("glyph"),
                "subglyph",
            )
            .to_sql(),
        @r#"SELECT "aspect" FROM (SELECT "image", "aspect" FROM "glyph") AS "subglyph""#
    );
}

#[test]
fn select_5() {
    assert_snapshot!(
        Select::new()
            .column(("glyph", "image"))
            .from("glyph")
            .and_where(Expr::column(("glyph", "aspect")).is_in([3, 4]))
            .to_sql(),
        @r#"SELECT "glyph"."image" FROM "glyph" WHERE "glyph"."aspect" IN (3, 4)"#
    );
}

#[test]
fn select_6() {
    assert_snapshot!(
        Select::new()
            .column("aspect")
            .expr(Expr::column("image").max())
            .from("glyph")
            .group_by_columns(["aspect"])
            .and_having(Expr::column("aspect").gt(2))
            .to_sql(),
        @r#"SELECT "aspect", MAX("image") FROM "glyph" GROUP BY "aspect" HAVING "aspect" > 2"#
    );
}
