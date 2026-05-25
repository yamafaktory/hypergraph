# hypergraph

## After every code change

Run all three before considering a task done:

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo nextest run --all-features
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

Unit tests live in a `#[cfg(test)] mod tests { … }` block at the bottom of the
same file as the code under test. Do not create separate `_test.rs` files.

Shared test helpers live in `src/core/test_support.rs`:

- `W(u8)` / `E(usize)` — lightweight vertex and hyperedge weight types
- `build()` — returns a small acyclic `Hypergraph<W, E>` with 4 vertices and 3
  hyperedges; use this as the default starting point for unit tests
- `disk::WP` / `disk::EP` / `disk::build_persistent(dir)` — equivalent helpers
  for `PersistentHypergraph` (only compiled with `features = ["persistence"]`)

Import them as:

```rust
use crate::core::test_support::{E, W, build};
```

## Adding new algorithms

New graph algorithms belong as **default methods** on the `HypergraphQuery`
trait in `src/core/query/trait_def.rs`. A default implementation means both
`Hypergraph` and `PersistentHypergraph` get the algorithm for free with no
additional impl work.

Every new algorithm needs two sets of tests:

1. **Unit tests** — `#[cfg(test)] mod tests { … }` at the bottom of
   `trait_def.rs`, using `build()` (or an inline custom graph where the
   standard fixture is not suitable).
2. **Integration tests** — a `query_<method_name>` test in
   `tests/integration_query_trait.rs` that calls the method via explicit trait
   dispatch (`HypergraphQuery::method(&g)`).

## Documentation and README

`README.md` and `src/lib.rs` (crate-level `//!` docs) must stay in sync and
exhaustively list every public algorithm and method. Both files contain an
**Algorithms** section organised into tables by category. When a new method is
added or removed, update both files:

- `README.md` — plain method names in the table (rendered on GitHub)
- `src/lib.rs` — linked method names using `[method](HypergraphQuery::method)`
  (resolved to API pages on docs.rs)

The categories are: Graph traversal, Shortest paths, Structural analysis, Graph
properties, Graph projections, Vertex and hyperedge queries, Analytics,
Mutations (`Hypergraph` only).

## HypergraphQuery trait

`src/core/query/trait_def.rs` defines `HypergraphQuery<V, HE>`. The nine
required primitive methods must be implemented for each backend; all other
methods are defaults built on those primitives.

`src/core/query/hypergraph_impl.rs` — impl for `Hypergraph`
`src/core/query/persistent_impl.rs` — impl for `PersistentHypergraph`
`src/core/query/mod.rs` — re-exports only (`pub use trait_def::HypergraphQuery`)
