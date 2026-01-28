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

use pqb::expr::Expr;
use pqb::func::FunctionCall;
use pqb::query::{Insert, OnConflict};

#[test]
fn insert_on_conflict_1() {
    let query = Insert::new()
        .into_table("glyph")
        .columns(["aspect", "image"])
        .values([
            "04108048005887010020060000204E0180400400".into(),
            3.14.into(),
        ])
        .on_conflict(OnConflict::column("id").update_column("aspect"));
    assert_eq!(
        query.to_sql(),
        [
            r#"INSERT INTO "glyph" ("aspect", "image")"#,
            r#"VALUES ('04108048005887010020060000204E0180400400', 3.14)"#,
            r#"ON CONFLICT ("id") DO UPDATE SET "aspect" = "excluded"."aspect""#,
        ]
        .join(" ")
    );
}

#[test]
fn insert_on_conflict_2() {
    assert_eq!(
        Insert::new()
            .into_table("glyph")
            .columns(["aspect", "image"])
            .values([
                "04108048005887010020060000204E0180400400".into(),
                3.14.into(),
            ])
            .on_conflict(
                OnConflict::columns(["id", "aspect"])
                    .update_columns(["aspect", "image"])
            )
            .to_sql(),
        [
            r#"INSERT INTO "glyph" ("aspect", "image")"#,
            r#"VALUES ('04108048005887010020060000204E0180400400', 3.14)"#,
            r#"ON CONFLICT ("id", "aspect") DO UPDATE SET "aspect" = "excluded"."aspect", "image" = "excluded"."image""#,
        ]
            .join(" ")
    );
}

#[test]
fn insert_on_conflict_3() {
    assert_eq!(
        Insert::new()
            .into_table("glyph")
            .columns(["aspect", "image"])
            .values([
                "04108048005887010020060000204E0180400400".into(),
                3.14.into(),
            ])
            .on_conflict(
                OnConflict::columns(["id", "aspect"])
                    .values([
                        ("aspect", "04108048005887010020060000204E0180400400".into()),
                        ("image", 3.14.into()),
                    ])
            )
            .to_sql(),
        [
            r#"INSERT INTO "glyph" ("aspect", "image")"#,
            r#"VALUES ('04108048005887010020060000204E0180400400', 3.14)"#,
            r#"ON CONFLICT ("id", "aspect") DO UPDATE SET "aspect" = '04108048005887010020060000204E0180400400', "image" = 3.14"#,
        ]
            .join(" ")
    );
}

#[test]
fn insert_on_conflict_4() {
    assert_eq!(
        Insert::new()
            .into_table("glyph")
            .columns(["aspect", "image"])
            .values([
                "04108048005887010020060000204E0180400400".into(),
                3.14.into(),
            ])
            .on_conflict(
                OnConflict::columns(["id", "aspect"]).value("image", Expr::value(1).add(2))
            )
            .to_sql(),
        [
            r#"INSERT INTO "glyph" ("aspect", "image")"#,
            r#"VALUES ('04108048005887010020060000204E0180400400', 3.14)"#,
            r#"ON CONFLICT ("id", "aspect") DO UPDATE SET "image" = 1 + 2"#,
        ]
        .join(" ")
    );
}

#[test]
fn insert_on_conflict_5() {
    assert_eq!(
        Insert::new()
            .into_table("glyph")
            .columns(["aspect", "image"])
            .values([
                "04108048005887010020060000204E0180400400".into(),
                3.14.into(),
            ])
            .on_conflict(
                OnConflict::columns(["id", "aspect"])
                    .value("aspect", Expr::value("04108048005887010020060000204E0180400400"))
                    .update_column("image")
            )
            .to_sql(),
        [
            r#"INSERT INTO "glyph" ("aspect", "image")"#,
            r#"VALUES ('04108048005887010020060000204E0180400400', 3.14)"#,
            r#"ON CONFLICT ("id", "aspect") DO UPDATE SET "aspect" = '04108048005887010020060000204E0180400400', "image" = "excluded"."image""#,
        ]
            .join(" ")
    );
}

#[test]
fn insert_on_conflict_6() {
    assert_eq!(
        Insert::new()
            .into_table("glyph")
            .columns(["aspect", "image"])
            .values([
                "04108048005887010020060000204E0180400400".into(),
                3.14.into(),
            ])
            .on_conflict(
                OnConflict::columns(["id", "aspect"])
                    .update_column("aspect")
                    .value("image", Expr::value(1).add(2))
            )
            .to_sql(),
        [
            r#"INSERT INTO "glyph" ("aspect", "image")"#,
            r#"VALUES ('04108048005887010020060000204E0180400400', 3.14)"#,
            r#"ON CONFLICT ("id", "aspect") DO UPDATE SET "aspect" = "excluded"."aspect", "image" = 1 + 2"#,
        ]
            .join(" ")
    );
}

#[test]
fn insert_on_conflict_7() {
    assert_eq!(
        Insert::new()
            .into_table("glyph")
            .columns(["aspect", "image"])
            .values([
                "04108048005887010020060000204E0180400400".into(),
                3.14.into(),
            ])
            .on_conflict(OnConflict::expr(Expr::column("id")).update_column("aspect"))
            .to_sql(),
        [
            r#"INSERT INTO "glyph" ("aspect", "image")"#,
            r#"VALUES ('04108048005887010020060000204E0180400400', 3.14)"#,
            r#"ON CONFLICT ("id") DO UPDATE SET "aspect" = "excluded"."aspect""#,
        ]
        .join(" ")
    );
}

#[test]
fn insert_on_conflict_8() {
    assert_eq!(
        Insert::new()
            .into_table("glyph")
            .columns(["aspect", "image"])
            .values([
                "04108048005887010020060000204E0180400400".into(),
                3.14.into(),
            ])
            .on_conflict(
                OnConflict::exprs([Expr::column("id"), Expr::column("aspect")])
                    .update_column("aspect")
            )
            .to_sql(),
        [
            r#"INSERT INTO "glyph" ("aspect", "image")"#,
            r#"VALUES ('04108048005887010020060000204E0180400400', 3.14)"#,
            r#"ON CONFLICT ("id", "aspect") DO UPDATE SET "aspect" = "excluded"."aspect""#,
        ]
        .join(" ")
    );
}

#[test]
fn insert_on_conflict_9() {
    assert_eq!(
        Insert::new()
            .into_table("glyph")
            .columns(["aspect", "image"])
            .values([
                "04108048005887010020060000204E0180400400".into(),
                3.14.into(),
            ])
            .on_conflict(
                OnConflict::exprs([
                    Expr::column("id"),
                    FunctionCall::lower(Expr::column("tokens")).into(),
                ])
                .update_column("aspect")
            )
            .to_sql(),
        [
            r#"INSERT INTO "glyph" ("aspect", "image")"#,
            r#"VALUES ('04108048005887010020060000204E0180400400', 3.14)"#,
            r#"ON CONFLICT ("id", LOWER("tokens")) DO UPDATE SET "aspect" = "excluded"."aspect""#,
        ]
        .join(" ")
    );
}

#[test]
fn insert_on_conflict_10() {
    assert_eq!(
        Insert::new()
            .into_table("font")
            .columns(["id", "name"])
            .values([15.into(), "CyberFont Sans Serif".into()])
            .on_conflict(OnConflict::constraint("name_unique").do_nothing())
            .to_sql(),
        [
            r#"INSERT INTO "font" ("id", "name")"#,
            r#"VALUES (15, 'CyberFont Sans Serif')"#,
            r#"ON CONFLICT ON CONSTRAINT "name_unique" DO NOTHING"#,
        ]
        .join(" ")
    );
}

#[test]
fn insert_on_conflict_11() {
    assert_eq!(
        Insert::new()
            .into_table("font")
            .columns(["id", "name"])
            .values([20.into(), "Monospaced terminal".into()])
            .on_conflict(
                OnConflict::exprs([Expr::column("name"), Expr::is_null(Expr::column("variant"))])
                    .do_nothing()
            )
            .to_sql(),
        [
            r#"INSERT INTO "font" ("id", "name")"#,
            r#"VALUES (20, 'Monospaced terminal')"#,
            r#"ON CONFLICT ("name", "variant" IS NULL) DO NOTHING"#,
        ]
        .join(" ")
    );
}

#[test]
fn insert_on_conflict_do_nothing() {
    assert_eq!(
        Insert::new()
            .into_table("glyph")
            .columns(["aspect", "image"])
            .values(["abcd".into(), 3.14.into()])
            .on_conflict(OnConflict::columns(["id", "aspect"]).do_nothing(),)
            .to_sql(),
        [
            r#"INSERT INTO "glyph" ("aspect", "image")"#,
            r#"VALUES ('abcd', 3.14)"#,
            r#"ON CONFLICT ("id", "aspect") DO NOTHING"#,
        ]
        .join(" ")
    );
}
