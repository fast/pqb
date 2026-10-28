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
use pqb::query::Insert;
use pqb::query::Order;
use pqb::query::Returning;
use pqb::query::Select;
use pqb::query::With;
use pqb::table::ColumnDef;
use pqb::table::CreateTable;
use pqb::types::TableRef;

use crate::Error;
use crate::database_url;

#[tokio::test]
#[ignore = "requires PQB_TEST_DATABASE_URL"]
async fn postgres_merges_rows_and_returns_actions() -> Result<(), Error> {
    use pqb::query::Explain;
    use pqb::query::Merge;
    use pqb::query::MergeAction;
    use pqb::query::MergeWhen;
    let (client, connection) =
        tokio_postgres::connect(&database_url(), tokio_postgres::NoTls).await?;
    let connection = tokio::spawn(connection);
    let version: String = client
        .query_one("SHOW server_version_num", &[])
        .await?
        .get(0);
    let version: u32 = version.parse()?;
    if version < 150000 {
        eprintln!("MERGE requires PostgreSQL 15+; covered by the PostgreSQL 18 CI job");
        drop(client);
        connection.await??;
        return Ok(());
    }
    client
        .batch_execute(
            &CreateTable::new()
                .temporary()
                .table("inventory")
                .column(ColumnDef::new("id").int().primary_key())
                .column(ColumnDef::new("qty").int())
                .to_sql(),
        )
        .await?;
    client
        .batch_execute(
            &Insert::new()
                .into_table("inventory")
                .columns(["id", "qty"])
                .rows([
                    [1.into(), 1.into()],
                    [2.into(), 2.into()],
                    [4.into(), 4.into()],
                ])
                .to_sql(),
        )
        .await?;
    let source = CommonTableExpression::new("incoming")
        .columns(["id", "qty"])
        .expr_values(
            [[1, 5], [2, 0], [3, 7], [5, -1]]
                .into_iter()
                .map(|row| row.into_iter().map(|v| Expr::value(v).cast_as("integer"))),
        );
    let mut merge = Merge::new()
        .into_table(TableRef::from("inventory").alias("t"))
        .using(TableRef::subquery(
            Select::new().columns(["id", "qty"]).from("incoming"),
            "s",
        ))
        .with(With::new().cte(source))
        .on(Expr::column(("t", "id")).eq(Expr::column(("s", "id"))))
        .when(MergeWhen::matched(MergeAction::Delete).and_where(Expr::column(("s", "qty")).eq(0)))
        .when(MergeWhen::matched(MergeAction::update([(
            "qty",
            Expr::column(("s", "qty")),
        )])))
        .when(
            MergeWhen::not_matched(MergeAction::DoNothing)
                .and_where(Expr::column(("s", "qty")).lt(0)),
        )
        .when(MergeWhen::not_matched(MergeAction::insert(
            ["id", "qty"],
            [Expr::column(("s", "id")), Expr::column(("s", "qty"))],
        )));
    if version >= 170000 {
        merge = merge
            .when(MergeWhen::not_matched_by_source(MergeAction::Delete))
            .returning(Returning::expr_as(
                FunctionCall::custom("merge_action", Vec::<Expr>::new()),
                "action",
            ));
    }
    let explain = Explain::new().statement(merge.clone()).to_values();
    assert!(
        !client
            .query(explain.sql(), &explain.postgres_params())
            .await?
            .is_empty()
    );
    if version >= 170000 {
        let changes = Select::new()
            .column("action")
            .from("changes")
            .order_by([Order::column("action")])
            .with(With::new().cte(CommonTableExpression::new("changes").query(merge)))
            .to_values();
        let actions: Vec<String> = client
            .query(changes.sql(), &changes.postgres_params())
            .await?
            .iter()
            .map(|row| row.get(0))
            .collect();
        assert_eq!(actions, ["DELETE", "DELETE", "INSERT", "UPDATE"]);
    } else {
        let merge = merge.to_values();
        assert_eq!(
            client
                .execute(merge.sql(), &merge.postgres_params())
                .await?,
            3
        );
    }
    let rows = client
        .query("SELECT id, qty FROM inventory ORDER BY id", &[])
        .await?;
    let actual: Vec<(i32, i32)> = rows.iter().map(|row| (row.get(0), row.get(1))).collect();
    let expected = if version >= 170000 {
        vec![(1, 5), (3, 7)]
    } else {
        vec![(1, 5), (3, 7), (4, 4)]
    };
    assert_eq!(actual, expected);
    drop(client);
    connection.await??;
    Ok(())
}
