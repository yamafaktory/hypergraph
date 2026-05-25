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
