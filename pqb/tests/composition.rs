// Copyright 2026 FastLabs Developers
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
use pqb::query::Delete;
use pqb::query::Order;
use pqb::query::Returning;
use pqb::query::Select;
use pqb::query::With;
use pqb::types::JoinType;
use pqb::types::TableRef;

#[test]
fn recursive_cte_preserves_parameter_order() {
    let sequence = Select::new().expr(1).union_all(
        Select::new()
            .expr(Expr::column("n").add(1))
            .from("sequence")
            .and_where(Expr::column("n").lt(5)),
    );
    let query = Select::new().column("n").from("sequence").with(
        With::new().recursive().cte(
            CommonTableExpression::new("sequence")
                .column("n")
                .select(sequence),
        ),
    );
    query.to_sql().validate();
    let (sql, values) = query.to_values().into_parts();
    assert_eq!(
        sql.validate(),
        "WITH RECURSIVE \"sequence\" (\"n\") AS ((SELECT $1) UNION ALL (SELECT \"n\" + $2 FROM \"sequence\" WHERE \"n\" < $3)) SELECT \"n\" FROM \"sequence\""
    );
    assert_eq!(values, vec![1.into(), 1.into(), 5.into()]);
}

#[test]
fn set_operations_keep_operand_and_result_limits_separate() {
    let query = Select::new()
        .expr(1)
        .limit(1)
        .union(Select::new().expr(2))
        .intersect(Select::new().expr(2).union_all(Select::new().expr(3)))
        .limit(2);
    assert_eq!(
        query.to_sql().validate(),
        "((SELECT 1 LIMIT 1) UNION (SELECT 2)) INTERSECT ((SELECT 2) UNION ALL (SELECT 3)) LIMIT 2"
    );
    let (sql, values) = query.to_values().into_parts();
    sql.validate();
    assert_eq!(values, vec![1.into(), 2.into(), 2.into(), 3.into()]);
}

#[test]
fn latest_row_with_lateral_and_using_join() {
    let latest = Select::new()
        .column("body")
        .from("messages")
        .and_where(Expr::column("user_id").eq(Expr::column(("users", "id"))))
        .limit(1);
    let query = Select::new()
        .distinct_on([Expr::column(("users", "id"))])
        .column(("users", "id"))
        .column(("latest", "body"))
        .from("users")
        .left_join(TableRef::lateral_subquery(latest, "latest"), true)
        .join_using(JoinType::FullOuterJoin, "profiles", ["id"])
        .order_by([Order::column(("users", "id"))]);
    query.to_sql().validate();
    let (sql, values) = query.to_values().into_parts();
    assert!(sql.clone().validate().contains("LEFT JOIN LATERAL (SELECT"));
    assert!(sql.contains("FULL OUTER JOIN \"profiles\" USING (\"id\")"));
    assert_eq!(values, vec![true.into()]);
}

#[test]
fn data_modifying_cte_returns_deleted_rows() {
    let deleted = Delete::new()
        .from_table("events")
        .and_where(Expr::column("expired").eq(true))
        .returning(Returning::all());
    let query = Select::new()
        .column("id")
        .from("deleted")
        .with(With::new().cte(CommonTableExpression::new("deleted").query(deleted)));
    query.to_sql().validate();
    let (sql, values) = query.to_values().into_parts();
    assert_eq!(
        sql.validate(),
        "WITH \"deleted\" AS (DELETE FROM \"events\" WHERE \"expired\" = $1 RETURNING *) SELECT \"id\" FROM \"deleted\""
    );
    assert_eq!(values, vec![true.into()]);
}
