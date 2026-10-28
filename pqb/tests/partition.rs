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
use pqb::table::AlterTable;
use pqb::table::ColumnDef;
use pqb::table::CreateTable;
use pqb::table::PartitionBound;
use pqb::table::PartitionBy;
use pqb::table::PartitionValue;

#[test]
fn partition_bounds_and_expression_keys_render_as_utility_sql() {
    let parent = CreateTable::new()
        .table(("app", "events"))
        .column(ColumnDef::new("id").int())
        .partition_by(PartitionBy::range([Expr::column("id").add(1)]));
    parent.to_sql().validate();
    let partition = CreateTable::new()
        .table(("app", "events_low"))
        .partition_of(
            ("app", "events"),
            PartitionBound::range([PartitionValue::MinValue], [10]),
        )
        .column(ColumnDef::new("id").default(2.into()))
        .partition_by(PartitionBy::hash([Expr::column("id")]));
    let query = partition.to_values();
    assert!(query.values().is_empty());
    query.sql().to_owned().validate();
    for bound in [
        PartitionBound::range([10], [PartitionValue::MaxValue]),
        PartitionBound::list([Some("O'Reilly"), None]),
        PartitionBound::hash(4, 3),
        PartitionBound::default_partition(),
    ] {
        AlterTable::new()
            .table(("app", "events"))
            .attach_partition("child", bound)
            .to_sql()
            .validate();
    }
    AlterTable::new()
        .table("events")
        .detach_partition_concurrently("child")
        .to_sql()
        .validate();
    AlterTable::new()
        .table("events")
        .finalize_detach_partition("child")
        .to_sql()
        .validate();
}

#[test]
fn partition_builders_reject_malformed_bounds_and_combined_ddl() {
    assert!(std::panic::catch_unwind(|| PartitionBound::range([1, 2], [3])).is_err());
    assert!(std::panic::catch_unwind(|| PartitionBound::range([None::<i32>], [1])).is_err());
    assert!(
        std::panic::catch_unwind(|| PartitionBound::range(
            [PartitionValue::MinValue, 1.into()],
            [2, 3]
        ))
        .is_err()
    );
    assert!(std::panic::catch_unwind(|| PartitionBound::hash(2, 2)).is_err());
    assert!(
        std::panic::catch_unwind(|| AlterTable::new()
            .table("events")
            .detach_partition("child")
            .add_column(ColumnDef::new("note").text())
            .to_sql())
        .is_err()
    );
}
