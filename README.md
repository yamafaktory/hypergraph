![graph](hypergraph.svg)

---

[<img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/yamafaktory/hypergraph/ci.yml?branch=main&logo=github&style=flat-square">](https://github.com/yamafaktory/hypergraph/actions/workflows/ci.yml) [<img alt="Crates.io" src="https://img.shields.io/crates/v/hypergraph?style=flat-square">](https://crates.io/crates/hypergraph/versions?sort=semver) [<img alt="docs.rs" src="https://img.shields.io/docsrs/hypergraph?style=flat-square">](https://docs.rs/hypergraph)

Hypergraph is a data structure library to generate **directed** [hypergraphs](https://en.wikipedia.org/wiki/Hypergraph).

A hypergraph is a generalization of a graph in which a hyperedge can join any number of vertices.

## 📣 Goal

This library aims at providing the necessary methods for modeling complex, multiway (non-pairwise) relational data found in complex networks.
One of the main advantages of using a hypergraph model over a graph one is to provide a more flexible and natural framework to represent entities and their relationships (e.g. Alice uses some social network, shares some data to Bob, who shares it to Carol, etc).

## 🎁 Features

This library enables you to represent:

- **non-simple** hypergraphs with two or more hyperedges containing the exact same set of vertices
- **self-loops** — i.e., hyperedges containing vertices directed to themselves one or more times
- **unaries** — i.e., hyperedges containing a unique vertex

And to compute:

- Graph traversal: **BFS**, **DFS**, reachability, topological sort
- Shortest paths: **Dijkstra** point-to-point and single-source
- Structural analysis: strongly connected components, weakly connected components, all simple paths, subgraph extraction, cycle detection
- Filtered views: `retain_vertices`, `retain_hyperedges`
- Generic query interface: `HypergraphQuery` trait works over both `Hypergraph` and `PersistentHypergraph`

## ⚗️ Implementation

- 100% safe Rust
- Proper error handling
- Stable indexes for each hyperedge and each vertex — identity is the index, not the weight; duplicate weights are allowed on both sides
- Parallelism (with Rayon)
- `HypergraphQuery<V, HE>` trait — implement 9 primitives to get all graph algorithms for free; use it for generic functions and trait objects that work with either backend
- Optional `serde` support (`features = ["serde"]` in `Cargo.toml`)
- Optional `persistence` support (`features = ["persistence"]` in `Cargo.toml`)

## 🛠️ Installation

Add this to your `Cargo.toml` (replace _current_version_ with the [latest version of the library](https://crates.io/crates/hypergraph)):

```toml
[dependencies]
hypergraph = "current_version"
```

To enable disk-backed persistent graphs:

```toml
[dependencies]
hypergraph = { version = "current_version", features = ["persistence"] }
```

## 💾 Persistent graphs

The `persistence` feature unlocks [`PersistentHypergraph`](https://docs.rs/hypergraph), a disk-backed variant built on an [LSM-tree](https://en.wikipedia.org/wiki/Log-structured_merge-tree) (via [fjall](https://github.com/fjall-rs/fjall)) with an in-memory hot-data cache. It supports graphs that exceed available RAM and survives process restarts without any manual serialization step.

```rust
use std::sync::Arc;
use hypergraph::PersistentHypergraph;

// Opens the database directory, or creates it if it doesn't exist.
let g = Arc::new(PersistentHypergraph::<MyVertex, MyEdge>::open("/var/data/my-graph")?);

// All write methods take &self — share freely across threads.
let g2 = Arc::clone(&g);
std::thread::spawn(move || {
    g2.add_vertex(my_vertex)?;
    Ok(())
});
```

Vertex and hyperedge types must implement `serde::Serialize + serde::DeserializeOwned` in addition to the usual trait bounds.

A bounded LRU cache (via [quick-cache](https://github.com/arthurprs/quick-cache)) sits in front of the disk store, keeping hot vertex weights and hyperedges in memory. The default capacity is 10 000 entries per layer; use `PersistentHypergraph::open_with_capacity` to tune it for your workload.

## ⚡️ Usage

Please read the [documentation](https://docs.rs/hypergraph) to get started.
