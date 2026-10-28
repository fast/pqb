# Changelog

## Unreleased

### Breaking changes

- Raise the minimum supported Rust version from 1.93.0 to 1.94.0 for SQLx 0.9 support.
- Replace `ColumnSpec::default` and `ColumnSpec::generated` with `ColumnSpec::value`, using `ColumnValue::Default`, `Generated`, or `Identity`. Existing `ColumnDef` builder methods remain available.
- Make DDL `to_values()` return inline SQL and an empty parameter list, matching PostgreSQL utility-statement requirements.
- Restrict `Insert::or_default_values(n)` to zero or one row. Use `default_values()` for one default row, or explicit columns and rows containing `Expr::default_value()` for multiple rows.
- Reject structurally invalid statement and option combinations instead of returning unusable SQL. Use table constraints for primary keys, separate ALTER TABLE statements for renames and partition actions, and `Order::position` for output-column ordinals.

### New features

- Compose UNION, INTERSECT, and EXCEPT queries, including ALL variants, operand-local ordering and limits, complete joins and LATERAL subqueries, recursive CTEs, and data-modifying CTEs.
- Build CASE and JSON expressions, empty-list predicates, aggregate DISTINCT, ORDER BY and FILTER clauses, and window functions with partitions, ordering and frames. Embed expressions in trusted SQL fragments while preserving parameter order.
- Insert iterator-based batches with `Insert::rows`, validate dynamic rows with `try_values`, update through FROM, delete through USING, and return aliased expressions.
- Build MERGE with ordered conditional update, delete and insert actions on PostgreSQL 15+. RETURNING, use as a CTE, and NOT MATCHED BY SOURCE require PostgreSQL 17+.
- Build COPY commands for text, CSV and binary streams, query exports and server files or programs. COPY transport and row encoding remain the driver's responsibility.
- Create schemas, identity columns, named relational constraints, and RANGE, LIST and HASH partitions; attach, detach, and subpartition tables, including PostgreSQL 14+ concurrent detach.
- Bind values through optional `postgres` and `sqlx` adapters. `postgres_typed_params()` supports native-driver queries without SQL parameter type context, and `into_sqlx()` prepares SQLx 0.9 arguments.
- Add binary, date/time and decimal values, typed empty and NULL arrays, and validated multidimensional arrays with nullable elements and explicit lower bounds. Fixed-offset timestamps preserve their instant in UTC.
- Accept runtime identifier strings and iterator-based CTE rows; expose a shared `Statement` for CTEs and EXPLAIN.

### Bug fixes

- Render CTE VALUES inside the required parentheses and collect their parameters before the outer query's parameters.
- Render a one-row default insert as DEFAULT VALUES and preserve its ON CONFLICT and RETURNING clauses.
- Keep each JOIN and TABLESAMPLE attached to the intended FROM item when additional FROM items are added.
- Preserve expression grouping for operators with PostgreSQL's generic operator precedence, and render NULL and boolean truth predicates as SQL keywords rather than bind parameters.
- Render non-finite floating-point values as typed literals and reject statements exceeding PostgreSQL's 65,535-parameter limit.

Earlier releases are available in the [release history](https://github.com/fast/pqb/releases). Unreleased changes are relative to [v0.1.4](https://github.com/fast/pqb/tree/v0.1.4).
