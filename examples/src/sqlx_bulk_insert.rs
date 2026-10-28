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

//! Insert a batch of tasks from an iterator and decode the generated RETURNING rows with SQLx.
//! The connection uses DATABASE_URL and owns a temporary table; no persistent schema is needed.

use pqb::query::Insert;
use pqb::query::Returning;
use pqb::table::ColumnDef;
use pqb::table::CreateTable;
use sqlx::Connection;
use sqlx::Row;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let mut connection = sqlx::PgConnection::connect(&std::env::var("DATABASE_URL")?).await?;
    let (sql, arguments) = CreateTable::new()
        .temporary()
        .table("tasks")
        .column(ColumnDef::new("id").int().primary_key())
        .column(ColumnDef::new("title").text().not_null())
        .to_values()
        .into_sqlx()?;
    sqlx::query_with(sql, arguments)
        .execute(&mut connection)
        .await?;

    let (sql, arguments) = Insert::new()
        .into_table("tasks")
        .columns(["id", "title"])
        .rows((1..=3).map(|id| [id.into(), format!("Task {id}").into()]))
        .returning(Returning::all())
        .to_values()
        .into_sqlx()?;
    let rows = sqlx::query_with(sql, arguments)
        .fetch_all(&mut connection)
        .await?;
    for row in rows {
        println!(
            "{}: {}",
            row.get::<i32, _>("id"),
            row.get::<String, _>("title")
        );
    }
    connection.close().await?;
    Ok(())
}
