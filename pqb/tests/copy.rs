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
use pqb::query::Copy;
use pqb::query::CopyFormat;
use pqb::query::Select;

#[test]
fn copy_inlines_query_values_and_escapes_options() {
    let copy = Copy::from_query(
        Select::new()
            .column("name")
            .from("users")
            .and_where(Expr::column("name").eq("O'Reilly\\data")),
    )
    .format(CopyFormat::Csv)
    .header(true)
    .null("N'A")
    .force_quote_all();
    let bound = copy.to_values();
    assert!(bound.values().is_empty());
    bound.sql().to_owned().validate();
    assert!(bound.sql().contains(r"E'O\'Reilly\\data'"));
    assert!(bound.sql().contains(r"NULL E'N\'A'"));
    Copy::from_table(("import", "rows\"quoted"))
        .columns(["id", "name"])
        .format(CopyFormat::Csv)
        .delimiter(';')
        .quote('\'')
        .escape('\\')
        .force_null(["name"])
        .force_not_null(["id"])
        .encoding("UTF8")
        .and_where(Expr::column("id").gt(0))
        .to_sql()
        .validate();
    Copy::to_table("rows")
        .file("/tmp/it's-a-file.csv")
        .to_sql()
        .validate();
    Copy::from_table("rows")
        .program("printf 'a\\n'")
        .to_sql()
        .validate();
}

#[test]
fn copy_rejects_incompatible_format_and_direction_options() {
    assert!(
        std::panic::catch_unwind(|| Copy::from_table("rows")
            .format(CopyFormat::Binary)
            .delimiter(',')
            .to_sql())
        .is_err()
    );
    assert!(std::panic::catch_unwind(|| Copy::from_table("rows").quote('"').to_sql()).is_err());
    assert!(
        std::panic::catch_unwind(|| Copy::to_table("rows").and_where(Expr::column("id").eq(1)))
            .is_err()
    );
    assert!(std::panic::catch_unwind(|| Copy::from_table("rows").force_quote_all()).is_err());
    assert!(
        std::panic::catch_unwind(|| Copy::from_table("rows")
            .format(CopyFormat::Csv)
            .delimiter('"')
            .to_sql())
        .is_err()
    );
}
