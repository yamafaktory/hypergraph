# hypergraph

## After every code change

Run both of these before considering a task done:

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo nextest run
```

## Module structure

`mod.rs` files must only contain module declarations (`mod foo;`) and re-exports
(`pub use`). All type definitions, trait impls, and method implementations
belong in dedicated subfiles — one logical group per file (e.g. `types.rs`,
`helpers.rs`, `graph.rs`, `vertices.rs`, `hyperedges.rs`).

## Comments

Do not use ASCII ruler/banner comments as section dividers, e.g.:

```rust
// ──────────────────────────────────────────────────────────────────────────────
// Section title
// ──────────────────────────────────────────────────────────────────────────────
```

or inline section labels such as:

```rust
// ── Shortest paths ───────────────────────────────────────────────────────────
```

Instead, document public functions and methods with idiomatic Rust doc comments
(`///`). Include `# Errors`, `# Panics`, and `# Returns` sections where
relevant. Every public method in a `pub` API module must have a doc comment.

## Testing

Every public function and method must have at least one test that exercises it
directly.
