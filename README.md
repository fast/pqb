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
[![MSRV 1.94.0][msrv-badge]](https://www.whatrustisit.com)
[![Apache 2.0 licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/pqb.svg
[crates-url]: https://crates.io/crates/pqb
[docs-badge]: https://img.shields.io/docsrs/pqb
[docs-url]: https://docs.rs/pqb
[msrv-badge]: https://img.shields.io/badge/MSRV-1.94.0-green?logo=rust
[license-badge]: https://img.shields.io/crates/l/pqb
[license-url]: https://github.com/fast/pqb/blob/main/LICENSE
[actions-badge]: https://github.com/fast/pqb/actions/workflows/ci.yml/badge.svg
[actions-url]: https://github.com/fast/pqb/actions/workflows/ci.yml

**PQB builds PostgreSQL queries in Rust.** Compose dynamic queries, bind values, and execute them through SQLx, tokio-postgres, or postgres. The core has no dependencies, and database adapters and additional value types are optional.

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

## Execute with your driver

| Feature | Integration |
| --- | --- |
| `postgres` | `postgres-types` 0.2 binding for tokio-postgres 0.7 and postgres 0.19 |
| `sqlx` | SQLx 0.9 PostgreSQL arguments and SQL text |
| `with-json` | `serde_json::Value` |
| `with-uuid` | `uuid::Uuid` |
| `with-chrono` | Dates, times, naive timestamps, UTC and fixed-offset timestamps |
| `with-rust_decimal` | `rust_decimal::Decimal` |

Features compose: for example, `cargo add pqb --features postgres,with-json,with-chrono`. Choose a runtime and TLS configuration in your driver dependency; PQB does not select them for applications.

### tokio-postgres and postgres

With the `postgres` feature, `Value` implements `postgres_types::ToSql`. `postgres_params()` borrows the ordered parameters for either driver's query methods.

```rust,no_run
# #[cfg(feature = "postgres")]
# async fn example(client: &tokio_postgres::Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
use pqb::expr::Expr;
use pqb::query::Select;

let bound = Select::new()
    .columns(["id", "name"])
    .from("users")
    .and_where(Expr::column("id").eq(42_i64))
    .to_values();
let rows = client.query(bound.sql(), &bound.postgres_params()).await?;
# let _ = rows;
# Ok(())
# }
```

For synchronous `postgres::Client`, use the same arguments without `.await`.

### SQLx

`into_sqlx()` returns owned `SqlStr` and `PgArguments`, ready for `query_with` or `query_as_with`. The adapter handles SQLx 0.9's SQL-string marker; custom SQL still has to satisfy the trust rules below.

```rust,no_run
# #[cfg(feature = "sqlx")]
# async fn example(connection: &mut sqlx::PgConnection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
use pqb::query::{Insert, OnConflict, Returning};

let (sql, args) = Insert::new()
    .into_table("users")
    .columns(["id", "name"])
    .values([42_i64.into(), "Alice".into()])
    .on_conflict(OnConflict::column("id").update_column("name"))
    .returning(Returning::all())
    .to_values()
    .into_sqlx()?;
let rows = sqlx::query_with(sql, args).fetch_all(connection).await?;
# let _ = rows;
# Ok(())
# }
```

### Values and SQL types

| Rust value | PostgreSQL binding |
| --- | --- |
| `bool` | `boolean` |
| `i8`, `u8`, `i16` | `smallint` |
| `u16`, `i32` | `integer` |
| `u32`, `u64`, `i64` | `bigint`; `u64 > i64::MAX` returns an encoding error |
| `f32`, `f64` | `real`, `double precision`, including NaN and infinities |
| `String`, `&str` | `text` and driver-supported string types |
| `Vec<u8>`, `&[u8]` | `bytea` |
| `Option<T>` | The same SQL type, with `None` representing NULL |
| Enabled JSON, UUID, Chrono, Decimal types | Corresponding PostgreSQL types |

Choose the Rust width that matches the SQL column, particularly with postgres's checked binary encoder. PostgreSQL has no unsigned integer types. Both adapters check conversions instead of truncating. Fixed-offset timestamps are normalized to UTC, preserving the instant. For standalone parameters whose type PostgreSQL cannot infer, use `Expr::value(value).cast_as("bigint")`, for example.

Use typed arrays when empty or NULL arrays are possible:

```rust
use pqb::value::{ArrayType, Value};

let ids = Value::typed_array(ArrayType::Int, [Some(1), None, Some(3)]);
let empty = Value::typed_array(ArrayType::Int, Vec::<i32>::new());
let absent = Value::null_array(ArrayType::Int);
```

Adapters support homogeneous, one-dimensional arrays with nullable elements. `Value::array` remains available: postgres infers its type from the prepared statement; SQLx infers it from the first element and rejects untyped empty or NULL arrays. Mismatched element types return an encoding error.

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
| DDL | Create/alter/drop tables, identity/generated columns, relational constraints, create/drop indexes and schemas |

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

### COPY

`Copy::from_table` imports from STDIN; `Copy::to_table` exports to STDOUT. `Copy::from_query` exports a query, including its WHERE and ordering. Set `CopyFormat::Csv` or `Binary` as needed. Common options include delimiter, NULL marker, header, quote/escape, encoding, FREEZE and CSV force-quoting/NULL handling. Files and server programs can be selected with `.file(...)` and `.program(...)`; program commands must be trusted shell syntax, and PostgreSQL enforces server-side privileges.

```rust
use pqb::query::{Copy, CopyFormat};

let command = Copy::from_table("events")
    .columns(["id", "payload"])
    .format(CopyFormat::Csv)
    .header(true)
    .to_sql();
// tokio-postgres: client.copy_in(&command).await?; then send bytes and finish the sink.
// SQLx: connection.copy_in_raw(&command).await?; then send bytes and finish the import.
```

COPY commands inline escaped literals; `to_values()` returns no parameters. Transfer rows using the driver's COPY API, not its ordinary query method. PQB does not serialize CSV or binary COPY records. CSV headers work on PostgreSQL 14; text headers require 15+. To export a partitioned table's rows, use a SELECT through `from_query`.

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

`CommonTableExpression::expr_values` accepts expression rows, including casts when a driver needs explicit parameter type information. Set operations combine complete operands, including their own limits and ordering. Calling `order_by`, `limit`, or `offset` afterwards applies to the combined result. To filter or project that result, use `Select::new().from_subquery(combined, "result")`. `combine(SetOperator::IntersectAll, other)` and `combine(SetOperator::ExceptAll, other)` expose the remaining forms.

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

Builders reject basic structural mistakes such as missing mutation targets or inconsistent insert rows. `Insert::try_values` returns an error for dynamic row input; convenience setters and rendering may panic on programmer errors. This is not a complete SQL validator: schema-dependent names, expression types, grouping rules, supported function modifiers and version-specific syntax are checked by PostgreSQL. Expressions are dynamically typed, not a compile-time schema DSL.

The test suite parses generated SQL with PostgreSQL's parser. CI also executes representative relational workflows and parameter roundtrips through both drivers on PostgreSQL 14 and 18. Version-specific features require their corresponding server: negative numeric scales and scales exceeding precision require PostgreSQL 15+, and virtual generated columns require PostgreSQL 18+.

PQB does not yet have dedicated builders for partition management, or every extension-specific SQL construct. Use explicit trusted SQL where the structured API does not cover a statement. PQB does not implement an ORM, migration runner, or connection management.

## Compatibility notes

This development version raises the minimum Rust version to 1.94.0 to support SQLx 0.9. It also corrects behavior from 0.1.4:

- DDL `to_values()` embeds literals instead of producing unusable bind parameters.
- `default_values()` inserts one default row and preserves ON CONFLICT and RETURNING. `or_default_values(n)` accepts only zero or one; use explicit rows of `Expr::default_value()` for multiple default rows.
- `ColumnSpec::value` replaces separate default/generated fields; existing `ColumnDef` builder methods remain available.
- A standalone primary-key index or combined ALTER TABLE rename is rejected instead of generating invalid SQL.

The minimum Rust version may increase in minor releases; patch releases retain their minor release's requirement.

## Development

```shell
cargo x test
cargo x lint
# Point this at a disposable PostgreSQL instance. Tests use temporary tables.
PQB_TEST_DATABASE_URL=postgresql://postgres:postgres@localhost/postgres \
  cargo test -p pqb --all-features --test postgres_live -- --ignored
```

`cargo x test` covers both the core and all optional features. The database tests are explicitly ignored during ordinary offline test runs and required in CI. Each feature can also be checked independently with `cargo check -p pqb --no-default-features --features <feature>`.

## License and origins

Licensed under [Apache License, Version 2.0](https://github.com/fast/pqb/blob/main/LICENSE).

PQB is derived from [SeaQuery](https://github.com/SeaQL/sea-query), with a simpler API focused on PostgreSQL instead of abstraction over multiple database backends.
