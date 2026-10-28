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

//! Build a paginated task search with optional filters, retaining user input as parameters.

use pqb::expr::Expr;
use pqb::query::Order;
use pqb::query::Select;

fn task_search(search: Option<&str>, after_id: Option<i32>) -> Select {
    let mut query = Select::new()
        .columns(["id", "title"])
        .from("tasks")
        .and_where(Expr::column("done").eq(false))
        .order_by([Order::column("id")])
        .limit(20);
    if let Some(search) = search {
        query = query.and_where(Expr::column("title").ilike(format!("%{search}%")));
    }
    if let Some(after_id) = after_id {
        query = query.and_where(Expr::column("id").gt(after_id));
    }
    query
}

fn main() {
    let search = std::env::args().nth(1);
    let bound = task_search(search.as_deref(), Some(100)).to_values();
    println!("{}", bound.sql());
    println!("{:?}", bound.values());
}
