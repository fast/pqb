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

//! Insert a task, then update it through ON CONFLICT and read back each returned row.
//! The connection uses DATABASE_URL and the table disappears when the connection closes.

use pqb::query::Insert;
use pqb::query::OnConflict;
use pqb::query::Returning;
use pqb::table::ColumnDef;
use pqb::table::CreateTable;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let url = std::env::var("DATABASE_URL")?;
    let (client, connection) = tokio_postgres::connect(&url, tokio_postgres::NoTls).await?;
    let connection = tokio::spawn(connection);
    let table = CreateTable::new()
        .temporary()
        .table("tasks")
        .column(ColumnDef::new("id").int().primary_key())
        .column(ColumnDef::new("title").text().not_null());
    client.batch_execute(&table.to_sql()).await?;

    for title in ["Draft release notes", "Publish release notes"] {
        let bound = Insert::new()
            .into_table("tasks")
            .columns(["id", "title"])
            .values([1.into(), title.into()])
            .on_conflict(OnConflict::column("id").update_column("title"))
            .returning(Returning::all())
            .to_values();
        let rows = client
            .query_typed(bound.sql(), &bound.postgres_typed_params()?)
            .await?;
        for row in rows {
            println!(
                "{}: {}",
                row.get::<_, i32>("id"),
                row.get::<_, String>("title")
            );
        }
    }
    drop(client);
    connection.await??;
    Ok(())
}
