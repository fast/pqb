# Repository Guidelines

## Workflows

Use `cargo x` as the entry point for repository workflows. Read `cargo x --help` and the relevant subcommand's `--help` before running build, check, test, lint, or benchmark commands. Keep these commands, the README development instructions, and CI synchronized when changing the workspace layout.

The workspace separates the published `pqb` library from unpublished development packages. Keep database runtimes, example applications, and benchmark dependencies out of the library's normal dependencies. Preserve the dependency-free default build and check optional features individually; an all-feature build can hide missing feature dependencies.

## Rust Style

Follow the existing consuming-builder API and module organization. Use module visibility to define implementation boundaries, and use the narrowest visibility needed for items reachable through public modules or re-exported types.

Import symbols under their original names, starting intra-crate paths at `crate`. Retain module qualification when it explains the operation or distinguishes names. Avoid aliases without a concrete reason. Use `pub use self::...` for a module's public re-exports; reserve `use super::*` for test modules.

Prefer direct code and a few coherent abstractions. Public API documentation should explain observable behavior, supported PostgreSQL versions, and meaningful error or panic conditions. Comments should explain invariants or PostgreSQL quirks rather than restating the code.

## SQL and Driver Contracts

PQB builds PostgreSQL SQL; it does not manage connections, transactions, migrations, or result decoding. Keep schema-dependent validation and expression type resolution in PostgreSQL rather than introducing a second SQL type system.

Quote identifiers and bind data values. Raw SQL, custom function names, and cast type expressions are explicit trust boundaries. Preserve operator grouping and parameter order when composing expressions and statements. Utility statements such as DDL and COPY inline escaped literals because they do not accept ordinary query parameters.

Validate structural errors consistently across builder entry points and against the final state when setter order can affect validity. Preserve the distinction between fallible dynamic-input APIs and convenience APIs that panic on programmer errors. Do not silently execute queries or enable EXPLAIN ANALYZE.

## Tests and Benchmarks

Keep offline rendering and SQL-parser tests under `pqb/tests`. Real database workflows belong in `tests-integration`, use `PQB_TEST_DATABASE_URL`, and create temporary objects in a disposable database. Keep ordinary test runs independent of an external service; CI explicitly runs the database suite on PostgreSQL 14, 15, and 18, with version-specific cases gated by server capabilities.

Add coverage for observable regressions and non-trivial boundaries. Prefer extending an existing workflow to duplicating it. For changes to SQL rendering or binding, execute representative generated SQL against PostgreSQL; parser acceptance alone does not establish valid execution or correct results.

Benchmarks belong in `benchmarks` and measure builder construction, rendering, or parameter preparation separately from database execution. Keep setup outside the measured region unless setup is the workload. Use representative input sizes and retain results with black boxes. Do not turn noisy timings into CI thresholds or claim general performance improvements from a single workload.

## Examples and Documentation

Keep Markdown paragraphs and list items on one source line. Follow the existing table formatting.

Examples should demonstrate a small application scenario with concise module documentation. Put semantic explanations beside the code, and keep the README to an index and runnable commands. Driver examples should use temporary tables and obtain the connection string from the environment. Register examples explicitly, including any required Cargo features, and keep them compiling in CI.

## Changelog

Update `CHANGELOG.md` in the same PR for significant user-visible changes. Compare the final behavior with the latest release tag, rather than recording the sequence of commits in the current development cycle.

Verify that a bug was present in the latest release before calling it a bug fix. If the affected API is unreleased, describe its final contract in the feature entry instead. Include capabilities, public API migrations, compatibility changes, and demonstrated performance improvements. Exclude implementation history, tests, internal refactors, documentation, CI, tooling, and dependency maintenance unless they change supported or observable behavior.

Keep `Unreleased` at the top and add only categories with entries. Each bullet should describe one coherent user-visible change and any migration it requires. Do not invent release dates or backfill historical release notes from guesses.

## Pull Requests

Use the commit and PR title types configured in `.github/semantic.yml`. Keep each commit coherent and the PR description concise, explaining the final problem and behavior. Use `Summary` for routine changes and add `Design Notes` only when a design decision needs explanation. Report relevant validation and remaining limitations.
