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
use divan::counter::ItemsCount;
use pqb::query::Insert;

fn tasks(rows: usize) -> Insert {
    Insert::new()
        .into_table("tasks")
        .columns(["id", "title", "done"])
        .rows((0..rows as i32).map(|id| {
            [
                id.into(),
                format!("Task {id}: O'Reilly").into(),
                false.into(),
            ]
        }))
}

#[divan::bench(args = [1, 100, 1000])]
fn inline(bencher: Bencher, rows: usize) {
    let query = tasks(rows);
    bencher
        .counter(ItemsCount::new(rows))
        .bench_local(|| black_box(&query).to_sql());
}

#[divan::bench(args = [1, 100, 1000])]
fn bound(bencher: Bencher, rows: usize) {
    let query = tasks(rows);
    bencher
        .counter(ItemsCount::new(rows))
        .bench_local(|| black_box(&query).to_values());
}
