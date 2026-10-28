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

//! A PostgreSQL query builder with ordered parameters.
//!
//! PQB constructs SQL and ordered parameters. Expressions are dynamically typed; PostgreSQL checks
//! schema names and SQL types. The default build has no dependencies. Connections, transactions and
//! result decoding belong to your driver.
//!
//! # Parameterized queries
//!
//! ```
//! use pqb::expr::Expr;
//! use pqb::query::{Order, Select};
//!
//! let query = Select::new()
//!     .columns(["id", "name"])
//!     .from("users")
//!     .and_where(Expr::column("name").ilike("alice%"))
//!     .order_by([Order::column("id")])
//!     .limit(20)
//!     .to_values();
//! assert_eq!(query.sql(), "SELECT \"id\", \"name\" FROM \"users\" WHERE \"name\" ILIKE $1 ORDER BY \"id\" ASC LIMIT 20");
//! assert_eq!(query.values(), &["alice%".into()]);
//! ```
//!
//! [`query`] contains SELECT, mutations (including MERGE), COPY, CTEs, set operations, windows
//! and EXPLAIN. [`expr`]
//! provides predicates, CASE and trusted SQL fragments; [`func`] provides function calls and
//! aggregate modifiers. [`table`] includes partition management. DDL and COPY use inline literals
//! because PostgreSQL utility statements do not accept query parameters.
//!
//! # Value features
//!
//! Enable `with-json`, `with-uuid`, `with-chrono` or `with-rust_decimal` for additional
//! value types. [`SqlWriterValues::into_parts`] returns SQL and ordered values for
//! application-owned parameter encoding and execution.
//!
//! # Contracts
//!
//! Pass data through expressions and values. [`expr::Expr::custom`], custom function names and cast
//! type strings accept trusted SQL syntax; they must never contain interpolated user input.
//! Builders guard common structural mistakes and may panic on programmer errors.
//! [`query::Insert::try_values`] provides a fallible API for dynamic row input. A builder is not a
//! complete SQL validator: grouping rules, schema-dependent types and version-specific features are
//! checked by PostgreSQL. Prefer bound queries for execution; inline literals use PostgreSQL's
//! literal type inference.
//!
//! [`value::Value::typed_array`] and [`value::Value::null_array`] preserve the element type of
//! empty and NULL arrays. [`value::ArrayValue`] adds validated multidimensional arrays and explicit
//! lower bounds. See the [README](https://github.com/fast/pqb#readme) for
//! composition examples, value features and compatibility notes.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

pub mod expr;
pub mod func;
pub mod index;
pub mod query;
pub mod schema;
pub mod table;
pub mod types;
pub mod value;

mod writer;
pub use self::writer::SqlWriterValues;
