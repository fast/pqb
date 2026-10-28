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

use common::ValidateSql;
use pqb::expr::Expr;
use pqb::query::CommonTableExpression;
use pqb::query::Explain;
use pqb::query::Merge;
use pqb::query::MergeAction;
use pqb::query::MergeWhen;
use pqb::query::Returning;
use pqb::query::With;
use pqb::types::TableRef;

#[test]
fn merge_composes_ctes_conditions_and_returning_parameters() {
    let query = Merge::new()
        .into_table(TableRef::from("items").alias("t"))
        .using(TableRef::from("source").alias("s"))
        .on(Expr::column(("t", "id")).eq(Expr::column(("s", "id"))))
        .with(
            With::new().cte(
                CommonTableExpression::new("source")
                    .columns(["id", "qty"])
                    .expr_values(vec![vec![
                        Expr::value(1).cast_as("integer"),
                        Expr::value(2).cast_as("integer"),
                    ]]),
            ),
        )
        .when(MergeWhen::matched(MergeAction::Delete).and_where(Expr::column(("s", "qty")).eq(0)))
        .when(MergeWhen::matched(MergeAction::update([(
            "qty",
            Expr::column(("s", "qty")).add(3),
        )])))
        .when(MergeWhen::not_matched(MergeAction::insert(
            ["id", "qty"],
            [Expr::column(("s", "id")), 4.into()],
        )))
        .returning(Returning::expr_as(
            Expr::column(("t", "qty")).add(5),
            "new_qty",
        ));
    query.to_sql().validate();
    let bound = query.to_values();
    bound.sql().to_owned().validate();
    assert_eq!(
        bound.values(),
        &[1.into(), 2.into(), 0.into(), 3.into(), 4.into(), 5.into()]
    );
    let plan = Explain::new().statement(query).to_values();
    plan.sql().to_owned().validate();
    assert_eq!(plan.values(), bound.values());
}

#[test]
fn merge_rejects_invalid_actions_and_unreachable_clauses() {
    assert!(std::panic::catch_unwind(|| MergeWhen::not_matched(MergeAction::Delete)).is_err());
    assert!(
        std::panic::catch_unwind(|| MergeWhen::matched(MergeAction::insert(["id"], [1.into()])))
            .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| MergeWhen::not_matched(MergeAction::insert(
            ["id"],
            [1.into(), 2.into()]
        )))
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| Merge::new()
            .when(MergeWhen::matched(MergeAction::DoNothing))
            .when(MergeWhen::matched(MergeAction::Delete).and_where(Expr::column("qty").eq(0))))
        .is_err()
    );
}
