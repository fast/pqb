```text
                   __
    ____   ____ _ / /_
   / __ \ / __ `// __ \
  / /_/ // /_/ // /_/ /
 / .___/ \__, //_.___/
/_/        /_/
```

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MSRV 1.93.0][msrv-badge]](https://www.whatrustisit.com)
[![Apache 2.0 licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/pqb.svg
[crates-url]: https://crates.io/crates/pqb
[docs-badge]: https://img.shields.io/docsrs/pqb
[docs-url]: https://docs.rs/pqb
[msrv-badge]: https://img.shields.io/badge/MSRV-1.93.0-green?logo=rust
[license-badge]: https://img.shields.io/crates/l/pqb
[license-url]: https://github.com/fast/pqb/blob/main/LICENSE
[actions-badge]: https://github.com/fast/pqb/actions/workflows/ci.yml/badge.svg
[actions-url]: https://github.com/fast/pqb/actions/workflows/ci.yml

**PQB builds PostgreSQL queries in Rust.** Compose dynamic queries and collect ordered parameter values. The core has no dependencies; additional value types are optional.

PQB focuses on PostgreSQL syntax with owned, composable builders. It does not inspect your schema or infer SQL expression types; PostgreSQL validates those when the statement is prepared or executed. Connections, transactions, pooling, and result decoding belong to your driver.

## Quick start

```shell
cargo add pqb
```

```rust
use pqb::expr::Expr;
use pqb::query::{Order, Select};

let query = Select::new()
    .columns(["id", "name"])
    .from(("public", "users"))
    .and_where(Expr::column("active").eq(true))
    .and_where(Expr::column("name").ilike("alice%"))
    .order_by([Order::column("id")])
    .limit(20);

let bound = query.to_values();
assert_eq!(bound.sql(), "SELECT \"id\", \"name\" FROM \"public\".\"users\" WHERE \"active\" = $1 AND \"name\" ILIKE $2 ORDER BY \"id\" ASC LIMIT 20");
assert_eq!(bound.values(), &[true.into(), "alice%".into()]);
```

Use `Expr::column("name")` for a column and `Expr::value("Alice")` (or `"Alice".into()`) for data. Identifiers are quoted; tuples such as `("public", "users")` express qualification. A string containing a dot is one identifier, not a qualified name. Runtime `&str` names are supported, and `Iden::new_static` avoids allocating fixed identifiers.

## Values and parameters

`to_values()` returns SQL with numbered placeholders and a `Vec<Value>` in placeholder order. Use `sql()` and `values()` to borrow them, or `into_parts()` to take ownership. The application is responsible for encoding these values and executing the SQL.

| Feature | Value types |
| --- | --- |
| `with-json` | `serde_json::Value` |
| `with-uuid` | `uuid::Uuid` |
| `with-chrono` | Dates, times, naive timestamps, UTC and fixed-offset timestamps |
| `with-rust_decimal` | `rust_decimal::Decimal` |

Features compose: for example, `cargo add pqb --features with-json,with-chrono`. Binary values (`Vec<u8>` and `&[u8]`), scalar values and arrays are available without features. `Option<T>::None` represents NULL, and fixed-offset timestamps are normalized to UTC while preserving the instant.

Collected parameters retain their Rust value variants. Inline SQL uses PostgreSQL's literal type inference; use `Expr::value(value).cast_as("bigint")` when an explicit SQL type is needed. PostgreSQL has no unsigned integer types, so applications must choose a suitable SQL representation and check ranges when encoding unsigned values.

Use typed arrays when empty or NULL arrays are possible:

```rust
use pqb::value::{ArrayType, Value};

let ids = Value::typed_array(ArrayType::Int, [Some(1), None, Some(3)]);
let empty = Value::typed_array(ArrayType::Int, Vec::<i32>::new());
let absent = Value::null_array(ArrayType::Int);
```

`Value::typed_array` and `Value::null_array` preserve an explicit element type. `typed_array` records the type without validating elements; inline SQL casts the array, while the application encodes collected parameters. Use `ArrayValue::new` below when shape and element validation are needed. `Value::array` stores elements without a declared type, so empty or NULL arrays need a SQL type context when rendered inline.

For multiple dimensions, provide lengths and flat elements in row-major order (the last subscript varies fastest). Use `(length, lower_bound)` pairs when subscripts should start somewhere other than 1:

```rust
use pqb::query::Select;
use pqb::value::{ArrayType, ArrayValue};

let matrix = ArrayValue::new(ArrayType::Int, [2, 2], [Some(1), None, Some(3), Some(4)])?;
let shifted = ArrayValue::new(ArrayType::Int, [(2, 0), (2, -1)], [1, 2, 3, 4])?;
let query = Select::new().expr(matrix).expr(shifted).to_values();
// Each array occupies one parameter and retains its shape and lower bounds.
```

`ArrayValue::new` returns `ArrayError` for mismatched shape, incompatible elements, overflowing bounds or more than six dimensions. Unsigned elements must fit their corresponding signed SQL type, so `u64` values must fit `bigint`. Zero-length axes become PostgreSQL's canonical empty array; use `Value::null_array` for a NULL array. Inline SQL preserves dimensions and lower bounds too.

## Compose queries

| Area | Supported builders |
| --- | --- |
| Selection | Expressions and aliases, `DISTINCT`, `DISTINCT ON`, grouping, `HAVING`, ordering with NULL placement, limit/offset |
| Joins | Inner, left, right, full outer, cross, `USING`, subqueries, `LATERAL`, table-valued functions |
| Sets | `UNION`, `INTERSECT`, `EXCEPT`, with both distinct and `ALL` forms |
| CTEs | Multiple CTEs, `VALUES`, recursive queries, materialization hints, data-modifying statements |
| Expressions | Arithmetic, comparisons, NULL tests, range/array containment, JSON access, LIKE/ILIKE, tuples, subqueries, `EXISTS`, `CASE` |
| Aggregation | Aggregate `DISTINCT`, input ordering, `FILTER`, window partition/order/frame |
| Writes | Multi-row insert, insert from select, default values, upsert, update from, delete using, MERGE, aliased returning |
| PostgreSQL operations | COPY text/CSV/binary, row locks, `NOWAIT`, `SKIP LOCKED`, table sampling, `EXPLAIN` options |
| DDL | Create/alter/drop tables, identity/generated columns, relational constraints, create/drop indexes and schemas, RANGE/LIST/HASH partition management |

Use `Order::position(1)` to sort by an output column number; literal values passed to `Order::expr` are rejected because PostgreSQL treats them differently from bind parameters. Set operations accept output column names and positions in ORDER BY; wrap the result in `from_subquery` before ordering by an expression.

Joins and TABLESAMPLE attach to the most recently added FROM item. Add that item before calling join or sampling methods; appending another FROM item starts a separate join chain.

### MERGE

`Merge` (PostgreSQL 15+) evaluates ordered `MergeWhen` clauses against a source table or subquery. `MergeAction` supports update, delete, insert, default values and do nothing. Conditional clauses use `.and_where(...)`; an unconditional clause must come last for its match kind. RETURNING, MERGE in a CTE and `not_matched_by_source` require PostgreSQL 17+. WITH RECURSIVE is not supported for MERGE.

```rust
use pqb::expr::Expr;
use pqb::query::{Merge, MergeAction, MergeWhen};
use pqb::types::TableRef;

let query = Merge::new()
    .into_table(TableRef::from("inventory").alias("t"))
    .using(TableRef::from("incoming").alias("s"))
    .on(Expr::column(("t", "id")).eq(Expr::column(("s", "id"))))
    .when(MergeWhen::matched(MergeAction::update([
        ("qty", Expr::column(("s", "qty"))),
    ])))
    .when(MergeWhen::not_matched(MergeAction::insert(
        ["id", "qty"],
        [Expr::column(("s", "id")), Expr::column(("s", "qty"))],
    )))
    .to_values();
```

EXPLAIN execution options (`WAL`, `TIMING`, `SERIALIZE`) require an explicit `.analyze()`. PQB never enables ANALYZE implicitly, since it executes the statement. GENERIC_PLAN cannot be combined with ANALYZE.

### COPY

`Copy::from_table` imports from STDIN; `Copy::to_table` exports to STDOUT. `Copy::from_query` exports a query, including its WHERE and ordering. Set `CopyFormat::Csv` or `Binary` as needed. Common options include delimiter, NULL marker, header, quote/escape, encoding, FREEZE and CSV force-quoting/NULL handling. Files and server programs can be selected with `.file(...)` and `.program(...)`; program commands must be trusted shell syntax, and PostgreSQL enforces server-side privileges.

```rust
use pqb::query::{Copy, CopyFormat};

let command = Copy::from_table("events")
    .columns(["id", "payload"])
    .format(CopyFormat::Csv)
    .header(true)
    .to_sql();
```

COPY commands inline escaped literals; `to_values()` returns no parameters. The application must handle the PostgreSQL COPY protocol to transfer rows. PQB does not serialize CSV or binary COPY records. CSV headers work on PostgreSQL 14; text headers require 15+. To export a partitioned table's rows, use a SELECT through `from_query`.

### Partition management

Create partitioned tables with `CreateTable::partition_by`, and children with `partition_of`. Both can be combined for subpartitioning. `PartitionBound` supports RANGE endpoints (including `PartitionValue::MinValue`/`MaxValue`), LIST values (including NULL), HASH modulus/remainder and DEFAULT partitions. Keys can be columns or expressions.

```rust
use pqb::expr::Expr;
use pqb::table::{AlterTable, ColumnDef, CreateTable, PartitionBound, PartitionBy};

let parent = CreateTable::new().table("events")
    .column(ColumnDef::new("id").bigint())
    .partition_by(PartitionBy::range([Expr::column("id")])).to_sql();
let child = CreateTable::new().table("events_first")
    .partition_of("events", PartitionBound::range([0_i64], [1000_i64])).to_sql();
let detach = AlterTable::new().table("events").detach_partition("events_first").to_sql();
let attach = AlterTable::new().table("events")
    .attach_partition("events_first", PartitionBound::range([0_i64], [1000_i64])).to_sql();
```

Partition operations inline literals, like other DDL. Attach/detach must be separate ALTER TABLE commands. `detach_partition_concurrently` (PostgreSQL 14+) must run outside a transaction block and requires no default partition; `finalize_detach_partition` completes an interrupted detach. PostgreSQL validates existing rows, overlapping bounds and key types. Detaching preserves the child table and its data; use `DropTable` to remove it explicitly.

### Dynamic predicates

`Expr::all` combines conditions with AND; `Expr::any` uses OR. Empty collections yield TRUE and FALSE respectively. Empty `IN` and `NOT IN` lists also become boolean constants without allocating parameters. Conditional Rust code can build the list before passing it to PQB.

```rust
use pqb::expr::{Case, Expr};
use pqb::query::Select;

let search = Some("alice%");
let mut conditions = vec![Expr::column("active").eq(true)];
if let Some(search) = search {
    conditions.push(Expr::column("name").ilike(search));
}
let query = Select::new()
    .column("name")
    .expr_as(Case::new(Expr::column("score").gte(80), "high").otherwise("normal"), "tier")
    .from("users")
    .and_where(Expr::all(conditions));
```

### Recursive queries and set operations

```rust
use pqb::expr::Expr;
use pqb::query::{CommonTableExpression, Select, With};

let sequence = Select::new()
    .expr(Expr::value(1).cast_as("integer"))
    .union_all(Select::new()
        .expr(Expr::column("n").add(1))
        .from("sequence")
        .and_where(Expr::column("n").lt(5)));
let query = Select::new()
    .column("n")
    .from("sequence")
    .with(With::new().recursive().cte(
        CommonTableExpression::new("sequence").column("n").select(sequence),
    ));
```

`CommonTableExpression::expr_values` accepts expression rows, including casts for SQL type coercion. Set operations combine complete operands, including their own limits and ordering. Calling `order_by`, `limit`, or `offset` afterwards applies to the combined result. To filter or project that result, use `Select::new().from_subquery(combined, "result")`. `combine(SetOperator::IntersectAll, other)` and `combine(SetOperator::ExceptAll, other)` expose the remaining forms.

### Windowed aggregates

```rust
use pqb::expr::Expr;
use pqb::func::FunctionCall;
use pqb::query::{FrameBound, Order, Select, Window, WindowFrame};

let total = FunctionCall::sum(Expr::column("amount"))
    .filter(Expr::column("status").eq("paid"))
    .over(Window::new()
        .partition_by([Expr::column("account_id")])
        .order_by([Order::column("id")])
        .frame(WindowFrame::rows(FrameBound::UnboundedPreceding, FrameBound::CurrentRow)));
let query = Select::new().expr_as(total, "running_total").from("payments");
```

Modifiers must suit the selected PostgreSQL function. For example, aggregate DISTINCT is not supported inside a window function by PostgreSQL. Range frames with offsets require one numeric ordering expression; use trusted fragments for interval-valued frame offsets.

### Schema definitions

```rust
use pqb::expr::Expr;
use pqb::table::{ColumnDef, CreateTable, ForeignKey, ReferentialAction, TableConstraint};

let table = CreateTable::new()
    .table("orders")
    .column(ColumnDef::new("id").bigint().identity_by_default().primary_key())
    .column(ColumnDef::new("account_id").bigint().not_null())
    .column(ColumnDef::new("amount").numeric(12, 2).not_null())
    .constraint(TableConstraint::check(Expr::column("amount").gte(0)))
    .constraint(ForeignKey::new(["account_id"], "accounts", ["id"])
        .on_delete(ReferentialAction::Cascade));
let sql = table.to_sql();
```

DDL utility statements do not accept query parameters. Their `to_values()` methods therefore return escaped inline SQL with an empty parameter list. Column defaults, generated expressions and identity sequences are represented by mutually exclusive `ColumnValue` variants. Alter-table column renames must be emitted as separate statements.

## SQL validity and extension points

`to_values()` is the normal path for executing data queries. Parameter numbering is shared across the entire statement, including CTEs, subqueries, unions and returning expressions. `to_sql()` renders inline literals using PostgreSQL's literal type inference for inspection and statements that need them; PostgreSQL text cannot contain NUL bytes. Prepared queries are limited to 65,535 parameters, so split larger batches.

`Expr::custom`, custom function names, `cast_as` type syntax, and custom index methods accept **trusted SQL syntax**. Never interpolate user input into them. `SqlFragment` interleaves syntax and bound expressions without interpreting placeholder-like text:

```rust
use pqb::expr::{Expr, SqlFragment};
use pqb::query::Select;

let expr = SqlFragment::new("COALESCE(")
    .expr(Expr::column("name"))
    .sql(", ")
    .expr("O'Reilly")
    .sql(")");
assert_eq!(Select::new().expr(expr).to_values().sql(), "SELECT COALESCE(\"name\", $1)");
```

Builders reject structural mistakes such as missing targets, incompatible projection widths, unsupported built-in function modifiers and conflicting statement options. Checks at rendering use the final builder state, so changing setter order cannot bypass them. `Insert::try_values` returns an error for dynamic row input; convenience setters and rendering may panic on programmer errors. PostgreSQL still checks schema-dependent names, expression types, grouping rules, custom functions and version-specific syntax. Expressions are dynamically typed, not a compile-time schema DSL.

The test suite parses generated SQL with PostgreSQL's parser. CI also uses `psql` to execute generated SQL on PostgreSQL 14, 15 and 18, including prepared queries, COPY, partition operations and MERGE where supported. Version-specific features require their corresponding server: negative numeric scales and scales exceeding precision require PostgreSQL 15+, and virtual generated columns require PostgreSQL 18+.

PQB does not model every extension-specific SQL construct. Use explicit trusted SQL where the structured API does not cover a statement. PQB does not implement an ORM, migration runner, or connection management.

## Compatibility notes

This development version corrects behavior from 0.1.4:

- DDL `to_values()` embeds literals instead of producing unusable bind parameters.
- `default_values()` inserts one default row and preserves ON CONFLICT and RETURNING. `or_default_values(n)` accepts only zero or one; use explicit rows of `Expr::default_value()` for multiple default rows.
- `ColumnSpec::value` replaces separate default/generated fields; existing `ColumnDef` builder methods remain available.
- A standalone primary-key index or combined ALTER TABLE rename is rejected instead of generating invalid SQL.

The minimum Rust version may increase in minor releases; patch releases retain their minor release's requirement.

## Example

[dynamic_query](examples/src/dynamic_query.rs) demonstrates optional search filters, keyset pagination and ordered bind parameters without a database:

```shell
cargo run -p examples --example dynamic_query -- "release notes"
```

## Development

`pqb/` contains the published library and offline SQL tests. `tests-integration/` validates generated SQL through the `psql` command-line client. Runnable examples live in `examples/`, and performance workloads live in `benchmarks/`. These development packages are unpublished.

```shell
cargo x check
cargo x test
cargo x lint
# Requires psql on PATH and a disposable PostgreSQL database. Tests use temporary tables.
PQB_TEST_DATABASE_URL=postgresql://postgres:postgres@localhost/postgres \
  cargo x test --postgres
```

`cargo x test` tests the core in isolation before testing the workspace with all optional features. The database tests are explicitly ignored during ordinary offline test runs and required in CI. `cargo x check` checks each library feature independently, without workspace feature unification. See [AGENTS.md](AGENTS.md) for repository guidelines and [CHANGELOG.md](CHANGELOG.md) for user-visible changes.

### Benchmarks

```shell
cargo x bench
cargo x bench -- select::bound
cargo x bench --no-run
cargo x bench -- --test
```

Divan benchmarks measure construction of a joined SELECT, inline and parameterized rendering, and INSERT rendering at 1, 100, and 1,000 rows. Rendering cases build their query outside the measured region and include SQL allocation and parameter collection. They exclude driver encoding, network traffic, and PostgreSQL execution. Compare revisions on the same machine and toolchain; these workloads are a baseline, not a claim about overall database throughput. CI uses `--test` to smoke-test every case without timing thresholds.

## License and origins

Licensed under [Apache License, Version 2.0](https://github.com/fast/pqb/blob/main/LICENSE).

PQB is derived from [SeaQuery](https://github.com/SeaQL/sea-query), with a simpler API focused on PostgreSQL instead of abstraction over multiple database backends.
