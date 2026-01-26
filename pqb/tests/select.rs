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

#[test]
fn select_7() {
    assert_snapshot!(
        Select::new()
            .column("aspect")
            .from("glyph")
            .and_where(Expr::column("aspect").if_null(0).gt(2))
            .to_sql(),
        @r#"SELECT "aspect" FROM "glyph" WHERE COALESCE("aspect", 0) > 2"#
    );
}

#[test]
fn select_8() {
    assert_snapshot!(
        Select::new()
            .column("character")
            .from("character")
            .left_join(
                "font",
                Expr::column(("character", "font_id")).eq(Expr::column(("font", "id"))),
            )
            .to_sql(),
        @r#"SELECT "character" FROM "character" LEFT JOIN "font" ON "character"."font_id" = "font"."id""#
    );
}

#[test]
fn select_9() {
    assert_snapshot!(
        Select::new()
            .column("character")
            .from("character")
            .left_join(
                "font",
                Expr::column(("character", "font_id")).eq(Expr::column(("font", "id"))),
            )
            .inner_join(
                "glyph",
                Expr::column(("character", "character")).eq(Expr::column(("glyph", "image"))),
            )
            .to_sql(),
        @r#"SELECT "character" FROM "character" LEFT JOIN "font" ON "character"."font_id" = "font"."id" INNER JOIN "glyph" ON "character"."character" = "glyph"."image""#
    );
}

#[test]
fn select_10() {
    assert_snapshot!(
        Select::new()
            .column("character")
            .from("character")
            .left_join(
                "font",
                Expr::column(("character", "font_id"))
                    .eq(Expr::column(("font", "id")))
                    .and(Expr::column(("character", "font_id")).eq(Expr::column(("font", "id")))),
            )
            .to_sql(),
        @r#"SELECT "character" FROM "character" LEFT JOIN "font" ON "character"."font_id" = "font"."id" AND "character"."font_id" = "font"."id""#
    );
}
