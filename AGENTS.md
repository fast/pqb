# PQB Project Agent Guide

This document provides context and guidelines for AI agents working on the `pqb` project.

## Project Overview

**pqb** is a PostgreSQL Query Builder library in Rust, ported from [sea-query](https://github.com/SeaQL/sea-query)'s PostgreSQL dialect. It provides a type-safe, ergonomic API for constructing SQL queries programmatically.

### Key Design Decisions

- **Focus on PostgreSQL only** (unlike sea-query which supports MySQL, PostgreSQL, and SQLite)
- **Simplified architecture** compared to sea-query - no backend abstraction, direct PG dialect implementation
- **snapshot testing** using `insta` crate for SQL output verification

## Architecture

### Module Structure

```
pqb/src/
├── lib.rs      # Public exports
├── expr.rs     # SQL expressions (Expr enum, operators, function calls)
├── func.rs     # SQL built-in functions (MAX, MIN, SUM, COUNT, etc.)
├── query/
│   ├── mod.rs  # Query module exports
│   └── select.rs  # SELECT statement builder
├── types/      # Type system
│   ├── mod.rs  # Iden, TableRef, ColumnRef, etc.
│   └── qualification.rs  # Database/Schema/Table/Column name qualifications
├── value.rs    # SQL value types and serialization
└── writer.rs   # SqlWriter trait for SQL generation
```

### Core Types

| Type | Purpose |
|------|---------|
| `Expr` | SQL expression enum (Column, Value, Unary, Binary, FunctionCall, etc.) |
| `Iden` | SQL identifier (with automatic escaping for special chars) |
| `TableRef` | Table reference (table name or subquery) |
| `ColumnRef` | Column reference (with optional table qualification) |
| `Value` | SQL values (integers, strings, etc.) with NULL handling |
| `Select` | SELECT statement builder |

### SQL Generation Pattern

SQL is generated through `write_*` functions that take a `&mut impl SqlWriter`:

```rust
// Example: writing an expression
pub(crate) fn write_expr<W: SqlWriter>(w: &mut W, expr: &Expr) {
    match expr {
        Expr::Column(col) => write_column_ref(w, col),
        Expr::Value(v) => write_value(w, v.clone()),
        // ...
    }
}
```

## Porting from Sea-Query

### Reference Location

Sea-query source is at `../sea-query/` relative to this project root.

Key files for reference:
- `../sea-query/src/backend/postgres/query.rs` - PG query generation
- `../sea-query/src/backend/postgres/mod.rs` - PG-specific escaping/quoting
- `../sea-query/src/query/select.rs` - SelectStatement structure
- `../sea-query/src/func.rs` - SQL functions
- `../sea-query/tests/postgres/query.rs` - PG test cases

### Porting Checklist

#### Phase 1: Core Query Features ✅
- [x] Basic SELECT (columns, FROM, WHERE)
- [x] Aggregate functions (MAX, MIN, SUM, AVG, COUNT)
- [x] GROUP BY / HAVING
- [ ] JOIN (LEFT, RIGHT, INNER, CROSS)
- [ ] ORDER BY (with NULLS FIRST/LAST)
- [ ] LIMIT / OFFSET
- [ ] DISTINCT / DISTINCT ON

#### Phase 2: DML Statements
- [ ] INSERT
- [ ] INSERT with RETURNING
- [ ] UPDATE with FROM
- [ ] UPDATE with RETURNING
- [ ] DELETE
- [ ] DELETE with RETURNING
- [ ] ON CONFLICT

#### Phase 3: PG-Specific Operators
- [ ] ILIKE (case-insensitive LIKE)
- [ ] JSON operators (->, ->>, #>, #>>)
- [ ] Array operators (@>, <@, &&)
- [ ] Full-text search (@@)
- [ ] Range operators
- [ ] TABLESAMPLE

#### Phase 4: DDL Statements
- [ ] CREATE TABLE
- [ ] ALTER TABLE
- [ ] DROP TABLE
- [ ] CREATE INDEX
- [ ] DROP INDEX
- [ ] PG custom types (CREATE TYPE)

#### Phase 5: Advanced Query Features
- [ ] CTE (WITH / WITH RECURSIVE)
- [ ] UNION / INTERSECT / EXCEPT
- [ ] Window functions (OVER, PARTITION BY)
- [ ] Lock clauses (FOR UPDATE, FOR SHARE)
- [ ] Subqueries

### How to Port a Test Case

1. **Find the test** in `../sea-query/tests/postgres/query.rs`
   ```bash
   grep -n "fn select_N" ../sea-query/tests/postgres/query.rs
   ```

2. **Understand the test** - identify what features it uses

3. **Check existing implementation** - see what's already done

4. **Implement missing features**:
   - Add new `Expr` variants if needed
   - Add new methods to `Select` or builders
   - Update `write_*` functions for SQL generation

5. **Add the test** to `pqb/tests/select.rs`

6. **Run tests** to verify:
   ```bash
   cd pqb && cargo test select_N
   ```

## Coding Guidelines

### Style

- Follow existing code style in the project
- Use `#[non_exhaustive]` for public enums (allows future additions)
- Use `#[expect(missing_docs)]` for trivial/self-documenting items
- Prefer builder pattern for complex types

### Error Handling

- This crate uses infallible SQL generation (no `Result`)
- Invalid states should be prevented by the type system at compile time

### Testing

- Use `insta::assert_snapshot!` for SQL output verification
- Test names follow sea-query convention: `select_N`, `insert_N`, etc.
- Tests are in `pqb/tests/` directory

### Example Test Pattern

```rust
use insta::assert_snapshot;
use pqb::expr::Expr;
use pqb::query::Select;

#[test]
fn select_N() {
    assert_snapshot!(
        Select::new()
            .columns(["col1"])
            .from("table")
            .to_sql(),
        @r#"SELECT "col1" FROM "table""#
    );
}
```

## Common Patterns

### Adding a New Binary Operator

1. Add to `BinaryOp` enum in `expr.rs`
2. Add case in `write_binary_op()`
3. Optionally add convenience method on `Expr`

### Adding a New SQL Function

1. Add to `Func` enum in `func.rs`
2. Add constructor method on `FunctionCall`
3. Handle in `write_function_call()`

### Adding GROUP BY / HAVING

See the existing implementation in `select.rs`:
- Store `groups: Vec<Expr>` and `having: Vec<Expr>` in `Select`
- Add builder methods: `group_by_columns()`, `and_having()`
- Write in `write_select()`: iterate and join with `, `

## Important Notes

- **Identifier Quoting**: PostgreSQL uses double quotes `"` for identifiers
- **String Escaping**: PG supports `E'...'` syntax for escape sequences (backslash escapes)
- **NULL Safety**: Use `Option<T>` in `Value` variants for nullable values
- **Tuple Comparison**: PG supports `(a, b) = (c, d)` syntax

## Related Projects

- **sea-query** (`../sea-query/`): The upstream query builder with multi-database support
