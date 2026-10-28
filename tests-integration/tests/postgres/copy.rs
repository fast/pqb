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
use pqb::query::Order;
use pqb::query::Select;
use pqb::table::ColumnDef;
use pqb::table::CreateTable;
use sqlx::Connection;

use crate::Error;
use crate::database_url;

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
