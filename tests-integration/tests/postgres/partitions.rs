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
use pqb::query::Insert;
use pqb::table::ColumnDef;
use pqb::table::CreateTable;

use crate::Error;
use crate::database_url;

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
