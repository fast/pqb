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
fn select_function() {
    assert_snapshot!(
        Select::new()
            .expr(Expr::function("int8range", [
                Expr::value(1),
                Expr::value(10),
                Expr::value("[]"),
            ]))
            .to_sql(),
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
            .to_sql(),
        @r#"SELECT * FROM "ranges" WHERE "r1" @> "r2" AND "r1" <@ "r2" AND "r1" && "r2" AND "r1" << "r2" AND "r1" >> "r2" AND "r1" &< "r2" AND "r1" &> "r2" AND "r1" -|- "r2""#
    );
}
