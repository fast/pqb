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

#[test]
fn dynamic_filters_case_and_json_keep_values_bound() {
    use pqb::expr::Case;
    use pqb::expr::SqlFragment;
    let label = Case::new(Expr::column("score").gte(80), "high").otherwise("low");
    let query = Select::new()
        .expr_as(label, "label")
        .expr(
            SqlFragment::new("COALESCE(")
                .expr(Expr::column("name"))
                .sql(", ")
                .expr("O'Reilly $1")
                .sql(")"),
        )
        .from("users")
        .and_where(Expr::all([
            Expr::column("profile").json_get_text("city").eq("上海"),
            Expr::any([
                Expr::column("name").ilike("alice%"),
                Expr::column("name").is_null(),
            ]),
        ]));
    query.to_sql().validate();
    let (sql, values) = query.to_values().into_parts();
    assert_eq!(
        sql.validate(),
        "SELECT CASE WHEN \"score\" >= $1 THEN $2 ELSE $3 END AS \"label\", COALESCE(\"name\", $4) FROM \"users\" WHERE (\"profile\" ->> $5) = $6 AND (\"name\" ILIKE $7 OR \"name\" IS NULL)"
    );
    assert_eq!(
        values,
        vec![
            80.into(),
            "high".into(),
            "low".into(),
            "O'Reilly $1".into(),
            "city".into(),
            "上海".into(),
            "alice%".into()
        ]
    );

    let (sql, values) = Select::new()
        .expr(Expr::all(Vec::<Expr>::new()))
        .expr(Expr::any(Vec::<Expr>::new()))
        .to_values()
        .into_parts();
    assert_eq!(sql.validate(), "SELECT TRUE, FALSE");
    assert!(values.is_empty());
}

#[test]
fn aggregates_and_window_frames() {
    use pqb::func::FunctionCall;
    use pqb::query::FrameBound;
    use pqb::query::Order;
    use pqb::query::Window;
    use pqb::query::WindowFrame;
    let total = FunctionCall::sum(Expr::column("amount"))
        .filter(Expr::column("status").eq("paid"))
        .over(
            Window::new()
                .partition_by([Expr::column("account_id")])
                .order_by([Order::column("id")])
                .frame(WindowFrame::rows(
                    FrameBound::UnboundedPreceding,
                    FrameBound::CurrentRow,
                )),
        );
    let query = Select::new()
        .expr_as(total, "running_total")
        .from("payments");
    query.to_sql().validate();
    let (sql, values) = query.to_values().into_parts();
    assert_eq!(
        sql.validate(),
        "SELECT SUM(\"amount\") FILTER (WHERE \"status\" = $1) OVER (PARTITION BY \"account_id\" ORDER BY \"id\" ASC ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS \"running_total\" FROM \"payments\""
    );
    assert_eq!(values, vec!["paid".into()]);

    Select::new()
        .expr(FunctionCall::count(Expr::column("account_id")).distinct())
        .expr(
            FunctionCall::custom("array_agg", [Expr::column("amount")])
                .order_by([Order::column("id").desc()]),
        )
        .from("payments")
        .to_sql()
        .validate();
}

#[test]
fn between_groups_compound_bounds() {
    let query = Select::new().expr(Expr::value(true).between(Expr::value(false).or(true), true));
    assert_eq!(
        query.to_sql().validate(),
        "SELECT TRUE BETWEEN (FALSE OR TRUE) AND TRUE"
    );
}
