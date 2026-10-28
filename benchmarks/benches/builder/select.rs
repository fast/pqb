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

use divan::Bencher;
use divan::black_box;
use pqb::expr::Expr;
use pqb::query::Order;
use pqb::query::Select;

fn task_list(tenant_id: i32, after_id: i32) -> Select {
    Select::new()
        .columns([("tasks", "id"), ("tasks", "title"), ("owners", "name")])
        .from("tasks")
        .left_join(
            "owners",
            Expr::column(("tasks", "owner_id")).eq(Expr::column(("owners", "id"))),
        )
        .and_where(Expr::column(("tasks", "tenant_id")).eq(tenant_id))
        .and_where(Expr::column(("tasks", "id")).gt(after_id))
        .order_by([Order::column(("tasks", "id"))])
        .limit(20)
}

#[divan::bench]
fn construct() -> Select {
    task_list(black_box(42), black_box(100))
}

#[divan::bench]
fn inline(bencher: Bencher) {
    let query = task_list(42, 100);
    bencher.bench_local(|| black_box(&query).to_sql());
}

#[divan::bench]
fn bound(bencher: Bencher) {
    let query = task_list(42, 100);
    bencher.bench_local(|| black_box(&query).to_values());
}
