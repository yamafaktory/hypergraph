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

## 📐 API reference

### Graph primitives

Available on both `Hypergraph` and `PersistentHypergraph` via the [`HypergraphQuery`](https://docs.rs/hypergraph) trait.

| Method | Description |
|---|---|
| `count_vertices()` / `count_hyperedges()` | Number of elements |
| `is_empty()` | Whether the graph has any vertices |
| `vertex_indices()` / `hyperedge_indices()` | All stable indices |
| `get_vertex_weight(idx)` / `get_hyperedge_weight(idx)` | Weight lookup by index |
| `get_vertex_hyperedges(idx)` | Hyperedge indices that include a vertex |
| `get_hyperedge_vertices(idx)` | Ordered vertex list of a hyperedge |

### Vertex and hyperedge lookups

| Method | Description |
|---|---|
| `contains_vertex(weight)` | Whether any vertex has the given weight |
| `get_vertex_index(weight)` | Indices of all vertices with the given weight |
| `find_hyperedges_by_weight(weight)` | Indices of all hyperedges with the given weight |
| `get_adjacent_vertices_from(v)` | Vertices directly reachable from `v` |
| `get_adjacent_vertices_to(v)` | Vertices with a direct edge into `v` |
| `get_full_adjacent_vertices_from(v)` | Neighbours from `v` paired with their connecting hyperedges |
| `get_full_adjacent_vertices_to(v)` | Predecessors of `v` paired with their connecting hyperedges |
| `get_full_vertex_hyperedges(v)` | Vertex lists of every hyperedge containing `v` |
| `get_vertex_degree_in(v)` / `get_vertex_degree_out(v)` | In/out degree |
| `get_hyperedges_intersections(edges)` | Shared vertices across multiple hyperedges |
| `get_hyperedges_connecting(from, to)` | Hyperedges that contain a directed `from→to` pair |

### Graph traversal

| Method | Description |
|---|---|
| `get_bfs(from)` | Breadth-first traversal order from a vertex |
| `get_dfs(from)` | Depth-first traversal order from a vertex |
| `is_reachable(from, to)` | Whether `to` is reachable from `from` |
| `get_all_paths(from, to)` | All simple paths between two vertices |
| `topological_sort()` | Kahn's algorithm; returns an error on cycles |

### Shortest paths

| Method | Description |
|---|---|
| `get_dijkstra_connections(from, to)` | Cheapest path with hyperedge trace |
| `get_dijkstra_connections_with_cost(from, to)` | Same, plus the total cost |
| `get_dijkstra_from(from)` | Cheapest cost to every reachable vertex |

### Structural analysis

| Method | Description |
|---|---|
| `strongly_connected_components()` | Kosaraju's algorithm |
| `connected_components()` | Weakly connected components |
| `is_acyclic()` | Cycle detection |
| `find_cut_vertices()` | Articulation points via iterative Tarjan DFS |
| `subgraph(vertices)` | Induced subgraph over a vertex set |

### Graph properties

| Method | Description |
|---|---|
| `get_orphan_vertices()` | Vertices belonging to no hyperedge |
| `get_orphan_hyperedges()` | Hyperedges with an empty vertex list |
| `get_endpoints()` | `(sources, sinks)` — in-degree 0 / out-degree 0 |
| `get_inclusions()` | All proper subset/superset pairs of hyperedges |
| `is_k_uniform(k)` | Whether every hyperedge has exactly `k` vertices |
| `get_core(min_degree, min_size)` | k-core decomposition via iterative peeling |

### Graph projections

| Method | Description |
|---|---|
| `expand_to_graph()` | Directed graph from consecutive vertex pairs |
| `expand_to_star()` | Bipartite vertex–hyperedge membership pairs |

### Analytics

| Method | Description |
|---|---|
| `compute_page_rank(damping, iterations)` | Iterative PageRank power method |

### Mutations (`Hypergraph` only)

| Method | Description |
|---|---|
| `new()` / `with_capacity(n)` | Create an empty graph |
| `add_vertex(weight)` | Add a vertex; returns its stable `VertexIndex` |
| `add_hyperedge(vertices, weight)` | Add a hyperedge; returns its stable `HyperedgeIndex` |
| `remove_vertex(idx)` | Remove a vertex and all hyperedges that contain it |
| `remove_hyperedge(idx)` | Remove a hyperedge |
| `update_vertex_weight(idx, weight)` | Replace a vertex's weight |
| `update_hyperedge_weight(idx, weight)` | Replace a hyperedge's weight |
| `update_hyperedge_vertices(idx, vertices)` | Replace a hyperedge's vertex list |
| `retain_vertices(predicate)` | Remove vertices that fail the predicate |
| `retain_hyperedges(predicate)` | Remove hyperedges that fail the predicate |
| `contract_hyperedge_vertices(edge, merge, into)` | Contract a set of vertices to one |
| `join_hyperedges(edges)` | Merge hyperedges into their union |
| `reverse_hyperedge(edge)` | Reverse the vertex ordering of a hyperedge |
| `clear_hyperedges()` | Remove all hyperedges, keeping vertices |
| `clear()` | Remove everything |

### Iterators (`Hypergraph` only)

| Method | Description |
|---|---|
| `iter()` | Borrowing iterator over `(&HE, Vec<&V>)` tuples |
| `vertices_iter()` | Iterator over `(VertexIndex, &V)` pairs |
| `hyperedges_iter()` | Iterator over `(HyperedgeIndex, &HE)` pairs |
| `into_iter()` | Consuming iterator over `(HE, Vec<V>)` tuples |

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
