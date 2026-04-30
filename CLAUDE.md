# CLAUDE.md

Notes for AI agents working on this repo. Keep this file short and current.

## What this is

`schermz` is a small Rust CLI that walks a JSON file and prints a schema-like
summary (which keys, which types, string min/max lengths, distinct object
shapes). It's published on crates.io as a binary crate.

## Layout

```
src/
  main.rs                — CLI entry point (clap, error handling, exits)
  schema.rs              — Schema + SchemaValueType, conversion logic
  schema/value_type.rs   — internal AST built from serde_json::Value
  schema/tests.rs        — all schema tests (insta snapshots)
  schema/snapshots/      — committed insta snapshot files
.github/workflows/ci.yml — check / test / fmt / clippy on push and PR
```

## Commands

```bash
cargo build
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo run -- -f sample.json            # try it
cargo run -- -f sample.json -m         # with -m (merge object shapes)
```

CI runs all four (check, test, fmt, clippy) and clippy is `-D warnings`. Don't
push code that warns.

## Testing

Tests use [insta](https://insta.rs/) snapshot tests. To update snapshots after
an intentional behavior change:

```bash
cargo install cargo-insta   # one-time
cargo insta test --review   # review and accept changed snapshots interactively
```

If you change snapshot output unintentionally, **investigate** — the project's
output format is the public contract. Snapshots being byte-identical is the
norm.

When adding tests, prefer snapshot tests over manual `assert_eq!` against a
giant JSON literal.

## Style

- Edition 2024. Inlined format args (`format!("{x}")`, not `format!("{}", x)`)
  — clippy enforces this.
- No `unwrap()`/`expect()` reachable from valid user input. `main.rs` returns
  friendly errors; the schema panics only on programmer errors (e.g. calling
  `SchemaObject::from_json` on a non-object).
- Public surface is small: `Schema::from_json`, `Schema::to_json`,
  `SchemaValueType`. Don't expand it without reason.
- Comments explain *why*, not *what*. The code is short — let it speak.

## Conventions when adding features

- Keep the output format stable. If you must change it, update every affected
  snapshot in one PR and call it out clearly.
- The `-m` (merge_objects) flag affects whether distinct object shapes for the
  same key are listed separately (`false`) or merged into a single combined
  shape (`true`). New behavior should respect this distinction.
- Bool / Null / Number → `"BOOL"` / `"NULL"` / `"NUMBER"` primitives.
  Strings → `"STRING(len)"` or `"STRING(min, max)"`.
- Empty arrays produce no entry in the output (we have no element-type info to
  record). Keep this — it's intentional.

## Out of scope (don't do these unprompted)

- Don't add a logging framework, async runtime, or extra error-handling crate.
  The CLI is ~30 lines for a reason.
- Don't replace `serde_json::Value` with a custom JSON parser.
- Don't introduce a library/binary split. It's a single binary crate.
- Don't add features just because they'd be "nice to have". Ask first.
