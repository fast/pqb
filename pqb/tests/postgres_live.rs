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

#![cfg(all(
    feature = "postgres",
    feature = "sqlx",
    feature = "with-json",
    feature = "with-uuid",
    feature = "with-chrono",
    feature = "with-rust_decimal"
))]

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
use pqb::value::ArrayType;
use pqb::value::ArrayValue;
use pqb::value::DateTime;
use pqb::value::Decimal;
use pqb::value::NaiveDate;
use pqb::value::Uuid;
use pqb::value::Value;
use sqlx::Connection;
use sqlx::Row;

type Error = Box<dyn std::error::Error + Send + Sync>;

fn database_url() -> String {
    std::env::var("PQB_TEST_DATABASE_URL")
        .expect("set PQB_TEST_DATABASE_URL to a disposable PostgreSQL database")
}

fn value_checks() -> Select {
    let date = NaiveDate::from_ymd_opt(2026, 10, 28).unwrap();
    let timestamp = DateTime::parse_from_rfc3339("2026-10-28T12:34:56+08:00").unwrap();
    let checks: Vec<(Value, &str, &str)> = vec![
        (true.into(), "boolean", "TRUE"),
        (127_i8.into(), "smallint", "127"),
        (u16::MAX.into(), "integer", "65535"),
        (u32::MAX.into(), "bigint", "4294967295"),
        (42_u64.into(), "bigint", "42"),
        ("O'Reilly\\世界".into(), "text", "E'O''Reilly\\\\世界'"),
        (
            vec![0_u8, 39, 92, 255].into(),
            "bytea",
            "decode('00275cff', 'hex')",
        ),
        (date.into(), "date", "DATE '2026-10-28'"),
        (
            date.and_hms_opt(4, 34, 56).unwrap().into(),
            "timestamp",
            "TIMESTAMP '2026-10-28 04:34:56'",
        ),
        (
            date.and_hms_opt(4, 34, 56).unwrap().time().into(),
            "time",
            "TIME '04:34:56'",
        ),
        (
            timestamp.into(),
            "timestamptz",
            "TIMESTAMPTZ '2026-10-28 04:34:56+00'",
        ),
        (Decimal::new(12345, 2).into(), "numeric", "123.45"),
        (
            serde_json::json!({"name": "O'Reilly", "active": true}).into(),
            "jsonb",
            "'{\"active\":true,\"name\":\"O''Reilly\"}'::jsonb",
        ),
        (
            Uuid::nil().into(),
            "uuid",
            "'00000000-0000-0000-0000-000000000000'::uuid",
        ),
        (
            Value::typed_array(ArrayType::Int, [Some(1), None, Some(3)]),
            "integer[]",
            "ARRAY[1, NULL, 3]",
        ),
        (
            Value::typed_array(ArrayType::Text, Vec::<String>::new()),
            "text[]",
            "ARRAY[]::text[]",
        ),
        (
            Value::null_array(ArrayType::Int),
            "integer[]",
            "NULL::integer[]",
        ),
        (
            ArrayValue::new(ArrayType::Int, [2, 2], [None, None, Some(3), Some(4)])
                .unwrap()
                .into(),
            "integer[]",
            "ARRAY[[NULL,NULL],[3,4]]::integer[]",
        ),
        (
            ArrayValue::new(
                ArrayType::Int,
                [(2, 0), (2, -1)],
                [Some(1), None, Some(3), Some(4)],
            )
            .unwrap()
            .into(),
            "integer[]",
            "'[0:1][-1:0]={{1,NULL},{3,4}}'::integer[]",
        ),
        (
            ArrayValue::new(
                ArrayType::Text,
                [2, 1, 2],
                [Some("a,b"), Some(r"O'Reilly\world"), Some("NULL"), None],
            )
            .unwrap()
            .into(),
            "text[]",
            r"ARRAY[[['a,b', E'O''Reilly\\world']], [['NULL', NULL]]]",
        ),
        (
            ArrayValue::new(ArrayType::Bytes, [2, 1], [vec![0_u8, 1, 39, 255], vec![]])
                .unwrap()
                .into(),
            "bytea[]",
            "ARRAY[[decode('000127ff', 'hex')], [decode('', 'hex')]]",
        ),
        (
            ArrayValue::new(
                ArrayType::Json,
                [1, 2],
                [Some(serde_json::json!({"key": "a,b"})), None],
            )
            .unwrap()
            .into(),
            "jsonb[]",
            "ARRAY[[jsonb_build_object('key', 'a,b'), NULL]]",
        ),
        (
            ArrayValue::new(ArrayType::Int, [2, 0, 3], Vec::<i32>::new())
                .unwrap()
                .into(),
            "integer[]",
            "'{}'::integer[]",
        ),
        (Option::<i32>::None.into(), "integer", "NULL::integer"),
        (
            f64::NAN.into(),
            "double precision",
            "'NaN'::double precision",
        ),
    ];
    Select::new()
        .expr(Expr::value(1).is_in(Vec::<i32>::new()).not())
        .exprs(checks.into_iter().map(|(value, ty, expected)| {
            Expr::value(value)
                .cast_as(ty)
                .is_not_distinct_from(Expr::custom(expected))
        }))
}

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
    let index = pqb::index::CreateIndex::new().table("payments").expr(
        pqb::expr::Case::new(Expr::column("status").eq("paid"), Expr::column("amount"))
            .otherwise(0),
    );
    client.batch_execute(&index.to_sql()).await?;

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
                .map(|row| {
                    row.into_iter()
                        .map(|v| Expr::value(v).cast_as("integer"))
                        .collect()
                })
                .collect(),
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

#[tokio::test]
#[ignore = "requires PQB_TEST_DATABASE_URL"]
async fn postgres_and_sqlx_copy_csv_and_binary_streams() -> Result<(), Error> {
    use futures_util::SinkExt;
    use futures_util::TryStreamExt;
    use pqb::query::Copy;
    use pqb::query::CopyFormat;
    let (client, connection) =
        tokio_postgres::connect(&database_url(), tokio_postgres::NoTls).await?;
    let connection = tokio::spawn(connection);
    let table = CreateTable::new()
        .temporary()
        .table("copy_rows")
        .column(ColumnDef::new("id").int())
        .column(ColumnDef::new("name").text());
    client.batch_execute(&table.to_sql()).await?;
    let copy_in = Copy::from_table("copy_rows")
        .columns(["id", "name"])
        .format(CopyFormat::Csv)
        .header(true)
        .and_where(Expr::column("id").lt(3))
        .to_sql();
    let sink = client.copy_in(&copy_in).await?;
    tokio::pin!(sink);
    sink.send(bytes::Bytes::from_static(
        b"id,name\n1,\"O'Reilly, \"\"quoted\"\"\"\n2,\n3,ignored\n",
    ))
    .await?;
    assert_eq!(sink.finish().await?, 2);
    let query = Select::new()
        .columns(["id", "name"])
        .from("copy_rows")
        .and_where(Expr::column("id").gt(0))
        .order_by([Order::column("id")]);
    let csv = Copy::from_query(query.clone())
        .format(CopyFormat::Csv)
        .header(true)
        .force_quote_all()
        .to_sql();
    let chunks: Vec<bytes::Bytes> = client.copy_out(&csv).await?.try_collect().await?;
    assert_eq!(
        chunks.concat(),
        b"id,name\n\"1\",\"O'Reilly, \"\"quoted\"\"\"\n\"2\",\n"
    );
    let binary = Copy::from_query(query).format(CopyFormat::Binary).to_sql();
    let chunks: Vec<bytes::Bytes> = client.copy_out(&binary).await?.try_collect().await?;
    drop(client);
    connection.await??;

    let mut sqlx = sqlx::PgConnection::connect(&database_url()).await?;
    let (sql, args) = table.to_values().into_sqlx()?;
    sqlx::query_with(sql, args).execute(&mut sqlx).await?;
    let copy_in = Copy::from_table("copy_rows")
        .format(CopyFormat::Binary)
        .to_sql();
    let mut import = sqlx.copy_in_raw(&copy_in).await?;
    for chunk in chunks {
        import.send(chunk).await?;
    }
    assert_eq!(import.finish().await?, 2);
    let chunks: Vec<bytes::Bytes> = sqlx.copy_out_raw(&csv).await?.try_collect().await?;
    assert_eq!(
        chunks.concat(),
        b"id,name\n\"1\",\"O'Reilly, \"\"quoted\"\"\"\n\"2\",\n"
    );
    sqlx.close().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires PQB_TEST_DATABASE_URL"]
async fn postgres_routes_and_manages_partitions() -> Result<(), Error> {
    use pqb::table::AlterTable;
    use pqb::table::PartitionBound;
    use pqb::table::PartitionBy;
    let (client, connection) =
        tokio_postgres::connect(&database_url(), tokio_postgres::NoTls).await?;
    let connection = tokio::spawn(connection);
    let columns = CreateTable::new()
        .temporary()
        .column(ColumnDef::new("id").int())
        .column(ColumnDef::new("region").text());
    client
        .batch_execute(
            &columns
                .clone()
                .table("measurements")
                .partition_by(PartitionBy::range([Expr::column("id")]))
                .to_sql(),
        )
        .await?;
    client
        .batch_execute(
            &CreateTable::new()
                .temporary()
                .table("low")
                .partition_of("measurements", PartitionBound::range([0], [10]))
                .partition_by(PartitionBy::list([Expr::column("region")]))
                .to_sql(),
        )
        .await?;
    for (name, parent, bound) in [
        ("low_west", "low", PartitionBound::list(["west"])),
        ("low_rest", "low", PartitionBound::default_partition()),
        (
            "outside",
            "measurements",
            PartitionBound::default_partition(),
        ),
    ] {
        client
            .batch_execute(
                &CreateTable::new()
                    .temporary()
                    .table(name)
                    .partition_of(parent, bound)
                    .to_sql(),
            )
            .await?;
    }
    client
        .batch_execute(&columns.table("manual_partition").to_sql())
        .await?;
    client
        .batch_execute(
            &AlterTable::new()
                .table("measurements")
                .attach_partition("manual_partition", PartitionBound::range([20], [30]))
                .to_sql(),
        )
        .await?;
    let insert = Insert::new()
        .into_table("measurements")
        .columns(["id", "region"])
        .rows([
            [1.into(), "west".into()],
            [2.into(), "east".into()],
            [11.into(), "east".into()],
            [21.into(), "west".into()],
        ])
        .to_values();
    client
        .execute(insert.sql(), &insert.postgres_params())
        .await?;
    let physical: Vec<String> = client
        .query(
            "SELECT tableoid::regclass::text FROM measurements ORDER BY id",
            &[],
        )
        .await?
        .iter()
        .map(|row| row.get(0))
        .collect();
    assert_eq!(
        physical,
        ["low_west", "low_rest", "outside", "manual_partition"]
    );
    client
        .batch_execute(
            &AlterTable::new()
                .table("measurements")
                .detach_partition("outside")
                .to_sql(),
        )
        .await?;
    client
        .batch_execute(
            &AlterTable::new()
                .table("measurements")
                .detach_partition_concurrently("manual_partition")
                .to_sql(),
        )
        .await?;
    assert_eq!(
        client
            .query_one("SELECT count(*) FROM measurements", &[])
            .await?
            .get::<_, i64>(0),
        2
    );
    assert_eq!(
        client
            .query_one("SELECT count(*) FROM manual_partition", &[])
            .await?
            .get::<_, i64>(0),
        1
    );
    client
        .batch_execute(
            &AlterTable::new()
                .table("measurements")
                .attach_partition("manual_partition", PartitionBound::range([20], [30]))
                .to_sql(),
        )
        .await?;
    assert_eq!(
        client
            .query_one("SELECT count(*) FROM measurements", &[])
            .await?
            .get::<_, i64>(0),
        3
    );

    client
        .batch_execute(
            &CreateTable::new()
                .temporary()
                .table("hashed")
                .column(ColumnDef::new("id").int())
                .partition_by(PartitionBy::hash([Expr::column("id")]))
                .to_sql(),
        )
        .await?;
    for remainder in 0..2 {
        client
            .batch_execute(
                &CreateTable::new()
                    .temporary()
                    .table(format!("hashed_{remainder}"))
                    .partition_of("hashed", PartitionBound::hash(2, remainder))
                    .to_sql(),
            )
            .await?;
    }
    client
        .batch_execute(
            &Insert::new()
                .into_table("hashed")
                .columns(["id"])
                .rows((0..20).map(|id| [Expr::value(id)]))
                .to_sql(),
        )
        .await?;
    let row = client
        .query_one("SELECT count(*), count(DISTINCT tableoid) FROM hashed", &[])
        .await?;
    assert_eq!(row.get::<_, i64>(0), 20);
    assert_eq!(row.get::<_, i64>(1), 2);
    drop(client);
    connection.await??;
    Ok(())
}
