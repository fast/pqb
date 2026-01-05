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

//! # A PostgreSQL Query Builder
//!
//! `pqb` is a Rust library designed to facilitate the construction of SQL queries for PostgreSQL
//! databases. It provides a type-safe and ergonomic API to build complex queries programmatically.
//!
//! # Examples
//!
//! ```
//! use pqb::query;
//!
//! let _select = query::select().to_sql();
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

pub mod expr;
pub mod query;
pub mod types;
