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

use insta::{assert_compact_debug_snapshot, assert_snapshot};
use pqb::expr::Expr;
use pqb::query::Select;
use pqb::types::Order;

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

#[test]
fn select_11() {
    assert_snapshot!(
        Select::new()
            .column("aspect")
            .from("glyph")
            .and_where(Expr::column("aspect").if_null(0).gt(2))
            .order_by("image", Order::Desc)
            .order_by(("glyph", "aspect"), Order::Asc)
            .to_sql(),
        @r#"SELECT "aspect" FROM "glyph" WHERE COALESCE("aspect", 0) > 2 ORDER BY "image" DESC, "glyph"."aspect" ASC"#
    );
}

#[test]
fn select_12() {
    assert_snapshot!(
        Select::new()
            .column("aspect")
            .from("glyph")
            .and_where(Expr::column("aspect").if_null(0).gt(2))
            .order_by_columns([("id", Order::Asc), ("aspect", Order::Desc)])
            .to_sql(),
        @r#"SELECT "aspect" FROM "glyph" WHERE COALESCE("aspect", 0) > 2 ORDER BY "id" ASC, "aspect" DESC"#
    );
}

#[test]
fn select_13() {
    assert_snapshot!(
        Select::new()
            .column("aspect")
            .from("glyph")
            .and_where(Expr::column("aspect").if_null(0).gt(2))
            .order_by_columns([
                (("glyph", "id"), Order::Asc),
                (("glyph", "aspect"), Order::Desc),
            ])
            .to_sql(),
        @r#"SELECT "aspect" FROM "glyph" WHERE COALESCE("aspect", 0) > 2 ORDER BY "glyph"."id" ASC, "glyph"."aspect" DESC"#
    );
}

#[test]
fn select_14() {
    assert_snapshot!(
        Select::new()
            .columns(["id", "aspect"])
            .expr(Expr::column("image").max())
            .from("glyph")
            .group_by_columns([("glyph", "id"), ("glyph", "aspect")])
            .and_having(Expr::column("aspect").gt(2))
            .to_sql(),
        @r#"SELECT "id", "aspect", MAX("image") FROM "glyph" GROUP BY "glyph"."id", "glyph"."aspect" HAVING "aspect" > 2"#
    );
}

#[test]
fn select_15() {
    assert_snapshot!(
        Select::new()
            .column("character")
            .from("character")
            .and_where(Expr::column("font_id").is_null())
            .to_sql(),
        @r#"SELECT "character" FROM "character" WHERE "font_id" IS NULL"#
    );
}

#[test]
fn select_16() {
    assert_snapshot!(
        Select::new()
            .column("character")
            .from("character")
            .and_where(Expr::column("font_id").is_null())
            .and_where(Expr::column("character").is_not_null())
            .to_sql(),
        @r#"SELECT "character" FROM "character" WHERE "font_id" IS NULL AND "character" IS NOT NULL"#
    );
}

#[test]
fn select_17() {
    assert_snapshot!(
        Select::new()
            .column(("glyph", "image"))
            .from("glyph")
            .and_where(Expr::column(("glyph", "aspect")).between(3, 5))
            .to_sql(),
        @r#"SELECT "glyph"."image" FROM "glyph" WHERE "glyph"."aspect" BETWEEN 3 AND 5"#
    );
}

#[test]
fn select_18() {
    assert_snapshot!(
        Select::new()
            .column("aspect")
            .from("glyph")
            .and_where(Expr::column("aspect").between(3, 5))
            .and_where(Expr::column("aspect").not_between(8, 10))
            .to_sql(),
        @r#"SELECT "aspect" FROM "glyph" WHERE ("aspect" BETWEEN 3 AND 5) AND ("aspect" NOT BETWEEN 8 AND 10)"#
    );
}

#[test]
fn select_19() {
    assert_snapshot!(
        Select::new()
            .column("character")
            .from("character")
            .and_where(Expr::column("character").eq("A"))
            .to_sql(),
        @r#"SELECT "character" FROM "character" WHERE "character" = 'A'"#
    );
}

#[test]
fn select_20() {
    assert_snapshot!(
        Select::new()
            .column("character")
            .from("character")
            .and_where(Expr::column("character").like("A"))
            .to_sql(),
        @r#"SELECT "character" FROM "character" WHERE "character" LIKE 'A'"#
    );
}

#[test]
fn select_21() {
    assert_snapshot!(
        Select::new()
            .column("character")
            .from("character")
            .and_where(
                Expr::column("character")
                    .like("A%")
                    .or(Expr::column("character").like("%B"))
                    .or(Expr::column("character").like("%C%")),
            )
            .to_sql(),
        @r#"SELECT "character" FROM "character" WHERE "character" LIKE 'A%' OR "character" LIKE '%B' OR "character" LIKE '%C%'"#
    );
}

#[test]
fn select_22() {
    assert_snapshot!(
        Select::new()
            .column("character")
            .from("character")
            .and_where(
                Expr::column("character")
                    .like("C")
                    .or(Expr::column("character").like("D").and(Expr::column("character").like("E"))),
            )
            .and_where(
                Expr::column("character")
                    .like("F")
                    .or(Expr::column("character").like("G")),
            )
            .to_sql(),
        @r#"SELECT "character" FROM "character" WHERE ("character" LIKE 'C' OR ("character" LIKE 'D' AND "character" LIKE 'E')) AND ("character" LIKE 'F' OR "character" LIKE 'G')"#
    );
}

#[test]
fn select_23() {
    // pqb style: no condition, no WHERE clause
    assert_snapshot!(
        Select::new()
            .column("character")
            .from("character")
            .to_sql(),
        @r#"SELECT "character" FROM "character""#
    );
}

#[test]
fn select_24() {
    // pqb style: use Rust if/else for conditional building
    // or just call and_where when condition is true
    assert_snapshot!(
        Select::new()
            .column("character")
            .from("character")
            .and_where(Expr::column("font_id").eq(5))
            .to_sql(),
        @r#"SELECT "character" FROM "character" WHERE "font_id" = 5"#
    );
}

#[test]
fn select_25() {
    assert_snapshot!(
        Select::new()
            .column("character")
            .from("character")
            .and_where(Expr::column("size_w").mul(2).eq(Expr::column("size_h").div(2)))
            .to_sql(),
        @r#"SELECT "character" FROM "character" WHERE "size_w" * 2 = "size_h" / 2"#
    );
}

#[test]
fn select_26() {
    assert_snapshot!(
        Select::new()
            .column("character")
            .from("character")
            .and_where(Expr::column("size_w").add(1).mul(2).eq(Expr::column("size_h").div(2).sub(1)))
            .to_sql(),
        @r#"SELECT "character" FROM "character" WHERE ("size_w" + 1) * 2 = ("size_h" / 2) - 1"#
    );
}

#[test]
fn select_27() {
    assert_snapshot!(
        Select::new()
            .columns(["character", "size_w", "size_h"])
            .from("character")
            .and_where(Expr::column("size_w").eq(3))
            .and_where(Expr::column("size_h").eq(4))
            .and_where(Expr::column("size_h").eq(5))
            .to_sql(),
        @r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" = 3 AND "size_h" = 4 AND "size_h" = 5"#
    );
}

#[test]
fn select_28() {
    // pqb style: use .or() chaining instead of any! macro
    assert_snapshot!(
        Select::new()
            .columns(["character", "size_w", "size_h"])
            .from("character")
            .and_where(
                Expr::column("size_w")
                    .eq(3)
                    .or(Expr::column("size_h").eq(4))
                    .or(Expr::column("size_h").eq(5)),
            )
            .to_sql(),
        @r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" = 3 OR "size_h" = 4 OR "size_h" = 5"#
    );
}

#[test]
fn select_30() {
    assert_snapshot!(
        Select::new()
            .columns(["character", "size_w", "size_h"])
            .from("character")
            .and_where(
                Expr::column("size_w").mul(2).add(Expr::column("size_h").div(3)).eq(4)
            )
            .to_sql(),
        @r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE ("size_w" * 2) + ("size_h" / 3) = 4"#
    );
}

#[test]
fn select_31() {
    assert_snapshot!(
        Select::new()
            .expr((1..10_i32).fold(Expr::value(0), |expr, i| expr.add(i)))
            .to_sql(),
        @r#"SELECT 0 + 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9"#
    );
}

#[test]
fn select_32() {
    assert_snapshot!(
        Select::new()
            .expr_as(Expr::column("character"), "C")
            .from("character")
            .to_sql(),
        @r#"SELECT "character" AS "C" FROM "character""#
    );
}

#[test]
fn select_33a() {
    assert_snapshot!(
        Select::new()
            .column("image")
            .from("glyph")
            .and_where(Expr::column("aspect").in_subquery(
                Select::new().expr(Expr::custom("3 + 2 * 2")),
            ))
            .to_sql(),
        @r#"SELECT "image" FROM "glyph" WHERE "aspect" IN (SELECT 3 + 2 * 2)"#
    );
}

#[test]
fn select_33b() {
    assert_snapshot!(
        Select::new()
            .column("image")
            .from("glyph")
            .and_where(Expr::column("aspect").in_subquery(
                Select::new().expr(Expr::column("ignore")),
            ))
            .to_sql(),
        @r#"SELECT "image" FROM "glyph" WHERE "aspect" IN (SELECT "ignore")"#
    );
}

#[test]
fn select_34() {
    assert_snapshot!(
        Select::new()
            .column("aspect")
            .expr(Expr::column("image").max())
            .from("glyph")
            .group_by_columns(["aspect"])
            .and_having(
                Expr::column("aspect")
                    .gt(2)
                    .or(Expr::column("aspect").lt(8))
                    .or(Expr::column("aspect").gt(12).and(Expr::column("aspect").lt(18)))
                    .or(Expr::column("aspect").gt(32)),
            )
            .to_sql(),
        @r#"SELECT "aspect", MAX("image") FROM "glyph" GROUP BY "aspect" HAVING "aspect" > 2 OR "aspect" < 8 OR ("aspect" > 12 AND "aspect" < 18) OR "aspect" > 32"#
    );
}

#[test]
fn select_35() {
    assert_snapshot!(
        Select::new()
            .column("id")
            .from("glyph")
            .and_where(Expr::column("aspect").is_null())
            .to_sql(),
        @r#"SELECT "id" FROM "glyph" WHERE "aspect" IS NULL"#
    );
}

#[test]
fn select_36() {
    assert_snapshot!(
        Select::new()
            .column("id")
            .from("glyph")
            .and_where(Expr::column("aspect").is_null())
            .to_sql(),
        @r#"SELECT "id" FROM "glyph" WHERE "aspect" IS NULL"#
    );
}

#[test]
fn select_37() {
    let (statement, values) = Select::new()
        .column("id")
        .from("glyph")
        .and_where(Expr::value(true).or(Expr::value(false)))
        .to_values()
        .into_parts();
    assert_snapshot!(
        statement,
        @r#"SELECT "id" FROM "glyph" WHERE $1 OR $2"#
    );
    assert_compact_debug_snapshot!(values, @"[Bool(Some(true)), Bool(Some(false))]");
}

#[test]
fn select_37a() {
    let (statement, values) = Select::new()
        .column("id")
        .from("glyph")
        .and_where(Expr::value(true).not().and(Expr::value(false).not()).not())
        .to_values()
        .into_parts();
    assert_snapshot!(
        statement,
        @r#"SELECT "id" FROM "glyph" WHERE NOT ((NOT $1) AND (NOT $2))"#
    );
    assert_compact_debug_snapshot!(values, @"[Bool(Some(true)), Bool(Some(false))]");
}

#[test]
fn select_38() {
    let (statement, values) = Select::new()
        .column("id")
        .from("glyph")
        .and_where(Expr::column("aspect").is_null().or(Expr::column("aspect").is_not_null()))
        .to_values()
        .into_parts();
    assert_snapshot!(
        statement,
        @r#"SELECT "id" FROM "glyph" WHERE "aspect" IS NULL OR "aspect" IS NOT NULL"#
    );
    assert!(values.is_empty());
}
