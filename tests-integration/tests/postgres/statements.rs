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
use pqb::query::Copy;
use pqb::query::CopyFormat;
use pqb::query::Insert;
use pqb::query::Merge;
use pqb::query::MergeAction;
use pqb::query::MergeWhen;
use pqb::query::Order;
use pqb::query::Returning;
use pqb::query::Select;
use pqb::query::With;
use pqb::table::AlterTable;
use pqb::table::ColumnDef;
use pqb::table::CreateTable;
use pqb::table::PartitionBound;
use pqb::table::PartitionBy;
use pqb::types::TableRef;

use crate::psql;

#[test]
#[ignore = "requires psql and PQB_TEST_DATABASE_URL"]
fn queries_copy_and_partitions_execute() {
    let table = CreateTable::new()
        .temporary()
        .table("tasks")
        .column(ColumnDef::new("id").int().primary_key())
        .column(ColumnDef::new("title").text());
    let insert = Insert::new()
        .into_table("tasks")
        .columns(["id", "title"])
        .rows([
            [1.into(), "first".into()],
            [2.into(), "O'Reilly".into()],
            [3.into(), "hidden".into()],
        ]);
    let query = Select::new()
        .columns(["id", "title"])
        .from("tasks")
        .and_where(Expr::column("id").gt(1))
        .and_where(Expr::column("title").ne("hidden"))
        .to_values();
    let joins = Select::new()
        .column(("a", "id"))
        .column(("c", "id"))
        .from(TableRef::from("tasks").alias("a"))
        .inner_join(
            TableRef::subquery(Select::new().expr_as(1, "id"), "b"),
            Expr::column(("a", "id")).eq(Expr::column(("b", "id"))),
        )
        .from_subquery(Select::new().expr_as(2, "id"), "c");
    let import = Copy::from_table("tasks")
        .columns(["id", "title"])
        .format(CopyFormat::Csv);
    let export = Copy::from_query(
        Select::new()
            .columns(["id", "title"])
            .from("tasks")
            .and_where(Expr::column("id").eq(4)),
    )
    .format(CopyFormat::Csv)
    .header(true);
    let parent = CreateTable::new()
        .temporary()
        .table("events")
        .column(ColumnDef::new("id").int())
        .partition_by(PartitionBy::range([Expr::column("id")]));
    let child = CreateTable::new()
        .temporary()
        .table("events_low")
        .partition_of("events", PartitionBound::range([0], [10]));
    let event = Insert::new()
        .into_table("events")
        .columns(["id"])
        .values([5.into()]);
    let route = Select::new()
        .expr(Expr::column("tableoid").cast_as("regclass").cast_as("text"))
        .from("events");
    let count = Select::new().expr(FunctionCall::count_all()).from("events");
    let detach = AlterTable::new()
        .table("events")
        .detach_partition("events_low");
    let attach = AlterTable::new()
        .table("events")
        .attach_partition("events_low", PartitionBound::range([0], [10]));
    let script = format!(
        "BEGIN;\n{};\n{};\nPREPARE tasks_query(integer, text) AS {};\nEXECUTE tasks_query(1, 'hidden');\n{};\n{};\n4,copied\n\\.\n{};\n{};\n{};\n{};\n{};\n{};\n{};\n{};\n{};\nROLLBACK;\n",
        table.to_sql(),
        insert.to_sql(),
        query.sql(),
        joins.to_sql(),
        import.to_sql(),
        export.to_sql(),
        parent.to_sql(),
        child.to_sql(),
        event.to_sql(),
        route.to_sql(),
        detach.to_sql(),
        count.to_sql(),
        attach.to_sql(),
        count.to_sql(),
    );
    assert_eq!(
        psql(&script),
        "2|O'Reilly\n1|2\nid,title\n4,copied\nevents_low\n0\n1"
    );
}

#[test]
#[ignore = "requires psql and PQB_TEST_DATABASE_URL"]
fn merge_executes_on_supported_servers() {
    let version: u32 = psql("SHOW server_version_num;\n").parse().unwrap();
    if version < 150000 {
        return;
    }
    let table = CreateTable::new()
        .temporary()
        .table("inventory")
        .column(ColumnDef::new("id").int().primary_key())
        .column(ColumnDef::new("qty").int());
    let insert = Insert::new()
        .into_table("inventory")
        .columns(["id", "qty"])
        .rows([
            [1.into(), 1.into()],
            [2.into(), 2.into()],
            [4.into(), 4.into()],
        ]);
    let source = CommonTableExpression::new("incoming")
        .columns(["id", "qty"])
        .values([
            [1.into(), 5.into()],
            [2.into(), 0.into()],
            [3.into(), 7.into()],
        ]);
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
        .when(MergeWhen::not_matched(MergeAction::insert(
            ["id", "qty"],
            [Expr::column(("s", "id")), Expr::column(("s", "qty"))],
        )));
    let sql = if version >= 170000 {
        merge = merge
            .when(MergeWhen::not_matched_by_source(MergeAction::Delete))
            .returning(Returning::expr_as(
                FunctionCall::custom("merge_action", Vec::<Expr>::new()),
                "action",
            ));
        Select::new()
            .column("action")
            .from("changes")
            .order_by([Order::column("action")])
            .with(With::new().cte(CommonTableExpression::new("changes").query(merge)))
            .to_sql()
    } else {
        merge.to_sql()
    };
    let result = Select::new()
        .columns(["id", "qty"])
        .from("inventory")
        .order_by([Order::column("id")]);
    let script = format!(
        "BEGIN;\n{};\n{};\n{};\n{};\nROLLBACK;\n",
        table.to_sql(),
        insert.to_sql(),
        sql,
        result.to_sql()
    );
    let expected = if version >= 170000 {
        "DELETE\nDELETE\nINSERT\nUPDATE\n1|5\n3|7"
    } else {
        "1|5\n3|7\n4|4"
    };
    assert_eq!(psql(&script), expected);
}
