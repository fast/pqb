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
use pqb::query::CommonTableExpression;
use pqb::query::Delete;
use pqb::query::FrameBound;
use pqb::query::Insert;
use pqb::query::OnConflict;
use pqb::query::Order;
use pqb::query::Returning;
use pqb::query::Select;
use pqb::query::Update;
use pqb::query::Window;
use pqb::query::WindowFrame;
use pqb::query::With;
use pqb::table::ColumnDef;
use pqb::table::CreateTable;
use pqb::table::ForeignKey;
use pqb::table::ReferentialAction;
use pqb::table::TableConstraint;
use pqb::types::TableRef;
use sqlx::Connection;
use sqlx::Row;

use crate::Error;
use crate::database_url;
use crate::values::value_checks;

#[tokio::test]
#[ignore = "requires PQB_TEST_DATABASE_URL"]
async fn postgres_executes_bound_queries_and_relational_workflows() -> Result<(), Error> {
    let (client, connection) =
        tokio_postgres::connect(&database_url(), tokio_postgres::NoTls).await?;
    let connection = tokio::spawn(connection);
    let checks = value_checks();
    let bound = checks.to_values();
    let row = client
        .query_one(bound.sql(), &bound.postgres_params())
        .await?;
    for i in 0..row.len() {
        assert!(row.get::<_, bool>(i), "bound value {i}");
    }
    let row = client.query_one(&checks.to_sql(), &[]).await?;
    for i in 0..row.len() {
        assert!(row.get::<_, bool>(i), "inline value {i}");
    }

    let accounts = CreateTable::new()
        .temporary()
        .table("accounts")
        .column(
            ColumnDef::new("id")
                .int()
                .identity_by_default()
                .primary_key(),
        )
        .column(ColumnDef::new("name").text().default("new".into()));
    client.batch_execute(&accounts.to_sql()).await?;
    let payments = CreateTable::new()
        .temporary()
        .table("payments")
        .column(
            ColumnDef::new("id")
                .int()
                .identity_by_default()
                .primary_key(),
        )
        .column(ColumnDef::new("account_id").int().not_null())
        .column(ColumnDef::new("amount").int().not_null())
        .column(ColumnDef::new("status").text())
        .constraint(TableConstraint::check(Expr::column("amount").gte(0)))
        .constraint(
            ForeignKey::new(["account_id"], "accounts", ["id"])
                .name("payments_account")
                .on_delete(ReferentialAction::Cascade),
        );
    client.batch_execute(&payments.to_sql()).await?;
    let insert = Insert::new()
        .into_table("accounts")
        .default_values()
        .returning(Returning::all())
        .to_values();
    let row = client
        .query_one(insert.sql(), &insert.postgres_params())
        .await?;
    assert_eq!(row.get::<_, i32>("id"), 1);
    assert_eq!(row.get::<_, String>("name"), "new");
    let insert = Insert::new()
        .into_table("payments")
        .columns(["account_id", "amount", "status"])
        .rows([
            vec![1.into(), 10.into(), "paid".into()],
            vec![1.into(), 5.into(), "pending".into()],
            vec![1.into(), 7.into(), "paid".into()],
        ])
        .to_values();
    assert_eq!(
        client
            .execute(insert.sql(), &insert.postgres_params())
            .await?,
        3
    );

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
        .expr(total)
        .from("payments")
        .order_by([Order::column("id")])
        .to_values();
    let rows = client.query(query.sql(), &query.postgres_params()).await?;
    assert_eq!(
        rows.iter()
            .map(|row| row.get::<_, i64>(0))
            .collect::<Vec<_>>(),
        [10, 10, 17]
    );

    let value_source = CommonTableExpression::new("input")
        .column("n")
        .expr_values(vec![
            vec![Expr::value(2).cast_as("integer")],
            vec![Expr::value(3).cast_as("integer")],
        ]);
    let query = Select::new()
        .expr(Expr::column("n").sum())
        .from("input")
        .with(With::new().cte(value_source))
        .to_values();
    assert_eq!(
        client
            .query_one(query.sql(), &query.postgres_params())
            .await?
            .get::<_, i64>(0),
        5
    );
    let method = String::from("btree");
    let index = pqb::index::CreateIndex::new()
        .name("payments_amount_idx")
        .table("payments")
        .using(method.as_str())
        .expr(
            pqb::expr::Case::new(Expr::column("status").eq("paid"), Expr::column("amount"))
                .otherwise(0),
        );
    client.batch_execute(&index.to_sql()).await?;
    client
        .batch_execute(
            &pqb::index::DropIndex::new()
                .index("payments_amount_idx")
                .concurrently()
                .to_sql(),
        )
        .await?;

    let latest = Select::new()
        .column("amount")
        .from("payments")
        .and_where(Expr::column("account_id").eq(Expr::column(("accounts", "id"))))
        .order_by([Order::column("id").desc()])
        .limit(1);
    let query = Select::new()
        .column(("latest", "amount"))
        .from("accounts")
        .left_join(TableRef::lateral_subquery(latest, "latest"), true)
        .to_values();
    assert_eq!(
        client
            .query_one(query.sql(), &query.postgres_params())
            .await?
            .get::<_, i32>(0),
        7
    );

    let update = Update::new()
        .table("accounts")
        .value("name", "settled")
        .from("payments")
        .and_where(Expr::column(("accounts", "id")).eq(Expr::column(("payments", "account_id"))))
        .returning(Returning::column(("accounts", "name")))
        .to_values();
    assert_eq!(
        client
            .query_one(update.sql(), &update.postgres_params())
            .await?
            .get::<_, String>(0),
        "settled"
    );

    let sequence = Select::new()
        .expr(Expr::value(1).cast_as("integer"))
        .union_all(
            Select::new()
                .expr(Expr::column("n").add(1))
                .from("sequence")
                .and_where(Expr::column("n").lt(5)),
        );
    let recursive = Select::new()
        .expr(Expr::column("n").sum())
        .from("sequence")
        .with(
            With::new().recursive().cte(
                CommonTableExpression::new("sequence")
                    .column("n")
                    .select(sequence),
            ),
        )
        .to_values();
    assert_eq!(
        client
            .query_one(recursive.sql(), &recursive.postgres_params())
            .await?
            .get::<_, i64>(0),
        15
    );

    let deleted = Delete::new()
        .from_table("payments")
        .using("accounts")
        .and_where(Expr::column(("payments", "account_id")).eq(Expr::column(("accounts", "id"))))
        .and_where(Expr::column("status").eq("pending"))
        .returning(Returning::column(("payments", "id")));
    let query = Select::new()
        .expr(FunctionCall::count_all())
        .from("deleted")
        .with(With::new().cte(CommonTableExpression::new("deleted").query(deleted)))
        .to_values();
    assert_eq!(
        client
            .query_one(query.sql(), &query.postgres_params())
            .await?
            .get::<_, i64>(0),
        1
    );
    drop(client);
    connection.await??;
    Ok(())
}

#[tokio::test]
#[ignore = "requires PQB_TEST_DATABASE_URL"]
async fn sqlx_executes_bound_values_and_upserts() -> Result<(), Error> {
    let mut connection = sqlx::PgConnection::connect(&database_url()).await?;
    let (sql, args) = value_checks().to_values().into_sqlx()?;
    let row = sqlx::query_with(sql, args)
        .fetch_one(&mut connection)
        .await?;
    for i in 0..row.len() {
        assert!(row.get::<bool, _>(i), "bound value {i}");
    }
    let table = CreateTable::new()
        .temporary()
        .table("entries")
        .column(ColumnDef::new("id").int().primary_key())
        .column(ColumnDef::new("name").text());
    let (sql, args) = table.to_values().into_sqlx()?;
    sqlx::query_with(sql, args).execute(&mut connection).await?;
    for name in ["first", "O'Reilly"] {
        let query = Insert::new()
            .into_table("entries")
            .columns(["id", "name"])
            .values([1.into(), name.into()])
            .on_conflict(OnConflict::column("id").update_column("name"))
            .returning(Returning::expr_as(Expr::column("name"), "saved_name"));
        let (sql, args) = query.to_values().into_sqlx()?;
        let row = sqlx::query_with(sql, args)
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(row.get::<String, _>("saved_name"), name);
    }
    connection.close().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires PQB_TEST_DATABASE_URL"]
async fn postgres_preserves_from_and_join_scope() -> Result<(), Error> {
    use pqb::query::TableSample;
    let (client, connection) =
        tokio_postgres::connect(&database_url(), tokio_postgres::NoTls).await?;
    let connection = tokio::spawn(connection);
    client
        .batch_execute(
            &CreateTable::new()
                .temporary()
                .table("join_scope")
                .column(ColumnDef::new("id").int())
                .to_sql(),
        )
        .await?;
    client
        .batch_execute(
            &Insert::new()
                .into_table("join_scope")
                .values([1.into()])
                .to_sql(),
        )
        .await?;
    let source = |id| Select::new().expr_as(Expr::value(id).cast_as("integer"), "id");
    let query = Select::new()
        .columns([("a", "id"), ("b", "id"), ("c", "id"), ("d", "id")])
        .from_as("join_scope", "a")
        .table_sample(TableSample::system())
        .inner_join(
            TableRef::subquery(source(1), "b"),
            Expr::column(("a", "id")).eq(Expr::column(("b", "id"))),
        )
        .from_subquery(source(2), "c")
        .left_join(
            TableRef::subquery(source(3), "d"),
            Expr::column(("c", "id")).lt(Expr::column(("d", "id"))),
        );
    let bound = query.to_values();
    let row = client
        .query_one(bound.sql(), &bound.postgres_params())
        .await?;
    assert_eq!(
        (
            row.get::<_, i32>(0),
            row.get::<_, i32>(1),
            row.get::<_, i32>(2),
            row.get::<_, i32>(3)
        ),
        (1, 1, 2, 3)
    );
    client.query_one(&query.to_sql(), &[]).await?;
    let ordered = source(2)
        .union_all(source(1))
        .order_by([Order::position(1)]);
    let bound = ordered.to_values();
    let ids: Vec<i32> = client
        .query(bound.sql(), &bound.postgres_params())
        .await?
        .iter()
        .map(|row| row.get(0))
        .collect();
    assert_eq!(ids, [1, 2]);
    let explain = pqb::query::Explain::new()
        .analyze()
        .wal(true)
        .timing(false)
        .statement(ordered)
        .to_values();
    assert!(
        !client
            .query(explain.sql(), &explain.postgres_params())
            .await?
            .is_empty()
    );
    let insert = Insert::new()
        .into_table("join_scope")
        .select_from(source(4))
        .columns(["id"])
        .to_values();
    assert_eq!(
        client
            .execute(insert.sql(), &insert.postgres_params())
            .await?,
        1
    );

    drop(client);
    connection.await??;
    Ok(())
}
