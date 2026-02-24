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
use pqb::expr::Expr;
use pqb::query::Select;

use crate::common::ValidateSql;

#[test]
fn select_function() {
    assert_snapshot!(
        Select::new()
            .expr(Expr::function("int8range", [
                Expr::value(1),
                Expr::value(10),
                Expr::value("[]"),
            ]))
            .to_sql()
            .validate(),
        @"SELECT int8range(1, 10, '[]')"
    );
}

#[test]
fn select_range_ops() {
    let left = Expr::column("r1");
    let right = Expr::column("r2");

    assert_snapshot!(
        Select::new()
            .expr(Expr::asterisk())
            .from("ranges")
            .and_where(left.clone().contains(right.clone()))
            .and_where(left.clone().contained_by(right.clone()))
            .and_where(left.clone().overlaps(right.clone()))
            .and_where(left.clone().strictly_left_of(right.clone()))
            .and_where(left.clone().strictly_right_of(right.clone()))
            .and_where(left.clone().does_not_extend_right_of(right.clone()))
            .and_where(left.clone().does_not_extend_left_of(right.clone()))
            .and_where(left.adjacent_to(right))
            .to_sql()
            .validate(),
        @r#"SELECT * FROM "ranges" WHERE "r1" @> "r2" AND "r1" <@ "r2" AND "r1" && "r2" AND "r1" << "r2" AND "r1" >> "r2" AND "r1" &< "r2" AND "r1" &> "r2" AND "r1" -|- "r2""#
    );
}

#[test]
fn select_is_distinct_from() {
    let left = Expr::column("c1");
    let right = Expr::column("c2");

    assert_snapshot!(
        Select::new()
            .expr(Expr::asterisk())
            .from("t")
            .and_where(left.clone().is_distinct_from(right.clone()))
            .and_where(left.clone().is_not_distinct_from(right.clone()))
            .to_sql()
            .validate(),
        @r#"SELECT * FROM "t" WHERE "c1" IS DISTINCT FROM "c2" AND "c1" IS NOT DISTINCT FROM "c2""#
    );

    assert_snapshot!(
        Select::new()
            .expr(Expr::asterisk())
            .from("t")
            .and_where(left.clone().add(Expr::value(1)).is_distinct_from(right.clone().add(Expr::value(2))))
            .to_sql()
            .validate(),
        @r#"SELECT * FROM "t" WHERE "c1" + 1 IS DISTINCT FROM "c2" + 2"#
    );
}

#[test]
fn select_is_null() {
    let c1 = Expr::column("c1");

    assert_snapshot!(
        Select::new()
            .expr(Expr::asterisk())
            .from("t")
            .and_where(c1.clone().add(Expr::value(1)).is_null())
            .to_sql()
            .validate(),
        @r#"SELECT * FROM "t" WHERE "c1" + 1 IS NULL"#
    );

    assert_snapshot!(
        Select::new()
            .expr(Expr::asterisk())
            .from("t")
            .and_where(c1.clone().add(Expr::value(1)).is_not_null())
            .to_sql()
            .validate(),
        @r#"SELECT * FROM "t" WHERE "c1" + 1 IS NOT NULL"#
    );
}
