//! Hypergraph is data structure library to generate directed [hypergraphs](https://en.wikipedia.org/wiki/Hypergraph).
//!
//! > A hypergraph is a generalization of a graph in which a hyperedge can join any number of vertices.
//!
//! ## Features
//!
//! This library enables you to:
//!
//! - represent **non-simple** hypergraphs with two or more hyperedges containing the exact same set of vertices
//! - represent **self-loops** — i.e., hyperedges containing vertices directed to themselves one or more times
//! - represent **unaries** — i.e., hyperedges containing a unique vertex
//!
//! Additional features:
//!
//! - Safe Rust implementation
//! - Proper error handling
//! - Stable indexes for each hyperedge and each vertex — identity is the index, not the weight; duplicate weights are permitted on both sides
//! - Graph traversal: BFS, DFS, topological sort, reachability check, all simple paths
//! - Shortest paths: Dijkstra point-to-point and single-source
//! - Structural analysis: strongly connected components, weakly connected components, subgraph extraction, cycle detection
//! - Filtered views: `retain_vertices`, `retain_hyperedges`
//! - Generic query interface: [`HypergraphQuery`] trait — implement 9 primitives to get all
//!   graph algorithms for free; write generic code that works with either backend
//! - Optional **`serde`** feature for serialization/deserialization support
//!   (enable with `features = ["serde"]` in `Cargo.toml`)
//! - Optional **`persistence`** feature for disk-backed graphs larger than RAM
//!   (enable with `features = ["persistence"]` in `Cargo.toml`)
//!
//! ## API reference
//!
//! ### Graph primitives
//!
//! Available on [`Hypergraph`] and [`PersistentHypergraph`] via [`HypergraphQuery`].
//!
//! | Method | Description |
//! |---|---|
//! | [`count_vertices`](HypergraphQuery::count_vertices) / [`count_hyperedges`](HypergraphQuery::count_hyperedges) | Number of elements |
//! | [`is_empty`](HypergraphQuery::is_empty) | Whether the graph has any vertices |
//! | [`vertex_indices`](HypergraphQuery::vertex_indices) / [`hyperedge_indices`](HypergraphQuery::hyperedge_indices) | All stable indices |
//! | [`get_vertex_weight`](HypergraphQuery::get_vertex_weight) / [`get_hyperedge_weight`](HypergraphQuery::get_hyperedge_weight) | Weight lookup by index |
//! | [`get_vertex_hyperedges`](HypergraphQuery::get_vertex_hyperedges) | Hyperedge indices that include a vertex |
//! | [`get_hyperedge_vertices`](HypergraphQuery::get_hyperedge_vertices) | Ordered vertex list of a hyperedge |
//!
//! ### Vertex and hyperedge lookups
//!
//! | Method | Description |
//! |---|---|
//! | [`contains_vertex`](HypergraphQuery::contains_vertex) | Whether any vertex has the given weight |
//! | [`get_vertex_index`](HypergraphQuery::get_vertex_index) | Indices of all vertices with the given weight |
//! | [`find_hyperedges_by_weight`](HypergraphQuery::find_hyperedges_by_weight) | Indices of all hyperedges with the given weight |
//! | [`get_adjacent_vertices_from`](HypergraphQuery::get_adjacent_vertices_from) | Vertices directly reachable from a vertex |
//! | [`get_adjacent_vertices_to`](HypergraphQuery::get_adjacent_vertices_to) | Vertices with a direct edge into a vertex |
//! | [`get_full_adjacent_vertices_from`](HypergraphQuery::get_full_adjacent_vertices_from) | Neighbours paired with their connecting hyperedges |
//! | [`get_full_adjacent_vertices_to`](HypergraphQuery::get_full_adjacent_vertices_to) | Predecessors paired with their connecting hyperedges |
//! | [`get_full_vertex_hyperedges`](HypergraphQuery::get_full_vertex_hyperedges) | Vertex lists of every hyperedge containing a vertex |
//! | [`get_vertex_degree_in`](HypergraphQuery::get_vertex_degree_in) / [`get_vertex_degree_out`](HypergraphQuery::get_vertex_degree_out) | In/out degree |
//! | [`get_hyperedges_intersections`](HypergraphQuery::get_hyperedges_intersections) | Shared vertices across multiple hyperedges |
//! | [`get_hyperedges_connecting`](HypergraphQuery::get_hyperedges_connecting) | Hyperedges containing a directed pair |
//! | [`get_vertex_neighborhood`](HypergraphQuery::get_vertex_neighborhood) | Co-members of a vertex across all its hyperedges |
//!
//! ### Graph traversal
//!
//! | Method | Description |
//! |---|---|
//! | [`get_bfs`](HypergraphQuery::get_bfs) | Breadth-first traversal order |
//! | [`get_dfs`](HypergraphQuery::get_dfs) | Depth-first traversal order |
//! | [`is_reachable`](HypergraphQuery::is_reachable) | Whether one vertex can reach another |
//! | [`get_all_paths`](HypergraphQuery::get_all_paths) | All simple paths between two vertices |
//! | [`topological_sort`](HypergraphQuery::topological_sort) | Kahn's algorithm; errors on cycles |
//! | [`random_walk`](HypergraphQuery::random_walk) | Weighted random walk from a starting vertex |
//!
//! ### Shortest paths
//!
//! | Method | Description |
//! |---|---|
//! | [`get_dijkstra_connections`](HypergraphQuery::get_dijkstra_connections) | Cheapest path with hyperedge trace |
//! | [`get_dijkstra_connections_with_cost`](HypergraphQuery::get_dijkstra_connections_with_cost) | Same, plus total cost |
//! | [`get_dijkstra_from`](HypergraphQuery::get_dijkstra_from) | Cheapest cost to every reachable vertex |
//!
//! ### Structural analysis
//!
//! | Method | Description |
//! |---|---|
//! | [`strongly_connected_components`](HypergraphQuery::strongly_connected_components) | Kosaraju's algorithm |
//! | [`connected_components`](HypergraphQuery::connected_components) | Weakly connected components |
//! | [`is_acyclic`](HypergraphQuery::is_acyclic) | Cycle detection |
//! | [`is_connected`](HypergraphQuery::is_connected) | Undirected connectivity (clique-expansion BFS) |
//! | [`get_transitive_closure`](HypergraphQuery::get_transitive_closure) | All directed reachability pairs |
//! | [`find_cut_vertices`](HypergraphQuery::find_cut_vertices) | Articulation points via iterative Tarjan DFS |
//! | [`subgraph`](crate::core::Hypergraph::subgraph) | Induced subgraph over a vertex set |
//!
//! ### Graph properties
//!
//! | Method | Description |
//! |---|---|
//! | [`get_orphan_vertices`](HypergraphQuery::get_orphan_vertices) | Vertices belonging to no hyperedge |
//! | [`get_orphan_hyperedges`](HypergraphQuery::get_orphan_hyperedges) | Hyperedges with an empty vertex list |
//! | [`get_endpoints`](HypergraphQuery::get_endpoints) | `(sources, sinks)` — in-degree 0 / out-degree 0 |
//! | [`get_inclusions`](HypergraphQuery::get_inclusions) | Proper subset/superset pairs of hyperedges |
//! | [`is_k_uniform`](HypergraphQuery::is_k_uniform) | Whether every hyperedge has exactly `k` vertices |
//! | [`get_core`](HypergraphQuery::get_core) | k-core decomposition via iterative peeling |
//! | [`get_nestedness_profile`](HypergraphQuery::get_nestedness_profile) | Per-size inclusion statistics ([`NestednessEntry`]) |
//!
//! ### Graph projections
//!
//! | Method | Description |
//! |---|---|
//! | [`expand_to_graph`](HypergraphQuery::expand_to_graph) | Directed graph from consecutive vertex pairs |
//! | [`expand_to_star`](HypergraphQuery::expand_to_star) | Bipartite vertex–hyperedge membership pairs |
//! | [`to_incidence_matrix`](HypergraphQuery::to_incidence_matrix) | Dense vertex×hyperedge incidence matrix |
//! | [`to_incidence_matrix_coo`](HypergraphQuery::to_incidence_matrix_coo) | Sparse COO incidence matrix |
//! | [`to_laplacian`](HypergraphQuery::to_laplacian) | Clique-expansion Laplacian (normalised or raw) |
//! | [`get_line_graph`](HypergraphQuery::get_line_graph) | Hyperedge pairs sharing at least one vertex |
//! | [`get_dual`](HypergraphQuery::get_dual) | Dual: per-vertex list of incident hyperedges |
//!
//! ### Analytics
//!
//! | Method | Description |
//! |---|---|
//! | [`compute_page_rank`](HypergraphQuery::compute_page_rank) | Iterative `PageRank` power method |
//! | [`compute_centrality`](HypergraphQuery::compute_centrality) | Degree, closeness, and betweenness ([`CentralityScores`]) |
//!
//! ### Mutations (`Hypergraph` only)
//!
//! | Method | Description |
//! |---|---|
//! | [`new`](Hypergraph::new) / [`with_capacity`](Hypergraph::with_capacity) | Create an empty graph |
//! | [`add_vertex`](Hypergraph::add_vertex) | Add a vertex; returns its stable [`VertexIndex`] |
//! | [`add_hyperedge`](Hypergraph::add_hyperedge) | Add a hyperedge; returns its stable [`HyperedgeIndex`] |
//! | [`remove_vertex`](Hypergraph::remove_vertex) | Remove a vertex and all hyperedges that contain it |
//! | [`remove_hyperedge`](Hypergraph::remove_hyperedge) | Remove a hyperedge |
//! | [`update_vertex_weight`](Hypergraph::update_vertex_weight) | Replace a vertex's weight |
//! | [`update_hyperedge_weight`](Hypergraph::update_hyperedge_weight) | Replace a hyperedge's weight |
//! | [`update_hyperedge_vertices`](Hypergraph::update_hyperedge_vertices) | Replace a hyperedge's vertex list |
//! | [`retain_vertices`](Hypergraph::retain_vertices) | Remove vertices that fail the predicate |
//! | [`retain_hyperedges`](Hypergraph::retain_hyperedges) | Remove hyperedges that fail the predicate |
//! | [`contract_hyperedge_vertices`](Hypergraph::contract_hyperedge_vertices) | Contract a set of vertices to one |
//! | [`join_hyperedges`](Hypergraph::join_hyperedges) | Merge hyperedges into their union |
//! | [`reverse_hyperedge`](Hypergraph::reverse_hyperedge) | Reverse the vertex ordering of a hyperedge |
//! | [`append_vertex_to_hyperedge`](Hypergraph::append_vertex_to_hyperedge) | Append a vertex to a hyperedge |
//! | [`prepend_vertex_to_hyperedge`](Hypergraph::prepend_vertex_to_hyperedge) | Prepend a vertex to a hyperedge |
//! | [`insert_vertex_into_hyperedge`](Hypergraph::insert_vertex_into_hyperedge) | Insert a vertex at a given position |
//! | [`delete_vertex_from_hyperedge`](Hypergraph::delete_vertex_from_hyperedge) | Remove the first occurrence of a vertex from a hyperedge |
//! | [`split_hyperedge`](Hypergraph::split_hyperedge) | Split a hyperedge at a position into two |
//! | [`split_vertex`](Hypergraph::split_vertex) | Duplicate a vertex into selected hyperedges |
//! | [`merge_vertices`](Hypergraph::merge_vertices) | Merge a set of vertices into one |
//! | [`get_k_skeleton`](Hypergraph::get_k_skeleton) | Sub-hypergraph of edges with at most `k` vertices |
//! | [`get_edge_induced_subhypergraph`](Hypergraph::get_edge_induced_subhypergraph) | Sub-hypergraph induced by a set of edges |
//! | [`clear_hyperedges`](Hypergraph::clear_hyperedges) | Remove all hyperedges, keeping vertices |
//! | [`clear`](Hypergraph::clear) | Remove everything |
//!
//! ### Iterators (`Hypergraph` only)
//!
//! | Method | Description |
//! |---|---|
//! | [`iter`](Hypergraph::iter) | Borrowing iterator over `(&HE, Vec<&V>)` tuples |
//! | [`vertices_iter`](Hypergraph::vertices_iter) | Iterator over `(VertexIndex, &V)` pairs |
//! | [`hyperedges_iter`](Hypergraph::hyperedges_iter) | Iterator over `(HyperedgeIndex, &HE)` pairs |
//! | `into_iter` | Consuming iterator over `(HE, Vec<V>)` tuples |
//!
//! ## Persistent disk-backed graphs
//!
//! Enable the `persistence` feature to unlock [`PersistentHypergraph`], a variant
//! backed by an [LSM-tree](https://en.wikipedia.org/wiki/Log-structured_merge-tree)
//! (via [fjall](https://github.com/fjall-rs/fjall)) with an in-memory hot-data
//! cache. It supports graphs that exceed available RAM and survives process
//! restarts without any manual serialization step.
//!
//! ```toml
//! # Cargo.toml
//! [dependencies]
//! hypergraph = { version = "*", features = ["persistence"] }
//! ```
//!
//! ### Vertex and hyperedge types
//!
//! In addition to the usual [`VertexTrait`] / [`HyperedgeTrait`] bounds, both
//! types must implement `serde::Serialize + serde::DeserializeOwned` so they
//! can be encoded to disk.
//!
//! ### Opening a graph
//!
//! [`PersistentHypergraph::open`] creates the database directory if it does not
//! exist, or recovers all data if it does.
//!
//! ```ignore
//! use std::sync::Arc;
//! use hypergraph::PersistentHypergraph;
//!
//! // Open (or create) a persistent graph.
//! let g = Arc::new(PersistentHypergraph::<MyVertex, MyEdge>::open("/var/data/my-graph")?);
//!
//! // All write methods take &self, so the Arc can be shared across threads.
//! let g2 = Arc::clone(&g);
//! std::thread::spawn(move || -> anyhow::Result<()> {
//!     g2.add_vertex(my_vertex)?;
//!     Ok(())
//! });
//! ```
//!
//! ### Thread safety
//!
//! [`PersistentHypergraph`] is `Send + Sync`. All write methods take `&self` and use
//! atomic counters internally, so the same instance can be shared across threads
//! via an `Arc` without an external `Mutex`.
//!
//! > **Note**: individual multi-step operations such as `add_hyperedge` are not
//! > serializable with respect to concurrent writers. If full operation-level
//! > isolation is required, wrap the `Arc<PersistentHypergraph>` in a `Mutex`.
//!
//! ### Persistence
//!
//! Writes are appended to the WAL immediately and are durable on process crash.
//! Call [`PersistentHypergraph::persist`] to additionally fsync to the physical
//! medium when you need a hard durability guarantee.
//!
//! ### Cache capacity
//!
//! By default the hot-data cache holds up to 10 000 entries per layer (vertices
//! and hyperedges). Use [`PersistentHypergraph::open_with_capacity`] to tune this for
//! your workload.
//!
//! ## Example
//!
//! Please notice that the hyperedges and the vertices must implement the
//! [`HyperedgeTrait`] and the [`VertexTrait`] respectively.
//!
//! ```
//! use hypergraph::{HyperedgeIndex, Hypergraph, VertexIndex};
//! use std::fmt::{Display, Formatter, Result};
//!
//! // Create a new struct to represent a person.
//! #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
//! pub struct Person<'a> {
//!     name: &'a str,
//! }
//!
//! impl<'a> Person<'a> {
//!     pub fn new(name: &'a str) -> Self {
//!         Self { name }
//!     }
//! }
//!
//! impl<'a> Display for Person<'a> {
//!     fn fmt(&self, f: &mut Formatter<'_>) -> Result {
//!         write!(f, "{}", self.name)
//!     }
//! }
//!
//! // Create a new struct to represent a relation.
//! #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
//! pub struct Relation<'a> {
//!     cost: usize,
//!     name: &'a str,
//! }
//!
//! impl<'a> Relation<'a> {
//!     pub fn new(name: &'a str, cost: usize) -> Self {
//!         Self { cost, name }
//!     }
//! }
//!
//! impl<'a> Display for Relation<'a> {
//!     fn fmt(&self, f: &mut Formatter<'_>) -> Result {
//!         write!(f, "{}", self.name)
//!     }
//! }
//!
//! impl<'a> From<Relation<'a>> for usize {
//!     fn from(relation: Relation<'a>) -> usize {
//!         relation.cost
//!     }
//! }
//!
//! fn main() -> std::result::Result<(),  Box<dyn std::error::Error>> {
//!     let mut graph = Hypergraph::<Person, Relation>::new();
//!
//!     // Create some folks.
//!     let ava_person = Person::new("Ava");
//!     let bianca_person = Person::new("Bianca");
//!     let charles_person = Person::new("Charles");
//!     let daena_person = Person::new("Daena");
//!     let ewan_person = Person::new("Ewan");
//!     let faarooq_person = Person::new("Faarooq");
//!     let ghanda_person = Person::new("Ghanda");
//!
//!     // Add those to the graph as vertices.
//!     let ava = graph.add_vertex(ava_person)?;
//!     let bianca = graph.add_vertex(bianca_person)?;
//!     let charles = graph.add_vertex(charles_person)?;
//!     let daena = graph.add_vertex(daena_person)?;
//!     let ewan = graph.add_vertex(ewan_person)?;
//!     let faarooq = graph.add_vertex(faarooq_person)?;
//!     let ghanda = graph.add_vertex(ghanda_person)?;
//!  
//!     // Each vertex gets a unique index by insertion order.
//!     assert_eq!(ava, VertexIndex(0));
//!     assert_eq!(ghanda, VertexIndex(6));
//!
//!     // Get the weight of a vertex.
//!     assert_eq!(graph.get_vertex_weight(VertexIndex(0)), Ok(&ava_person));
//!     
//!     // The hypergraph has seven vertices.
//!     assert_eq!(graph.count_vertices(), 7);
//!
//!     // Create some relations.
//!     let cat_video = Relation::new("share a viral video with a cat", 1);
//!     let dog_video = Relation::new("share a viral video with a dog", 1);
//!     let beaver_video = Relation::new("share a viral video with a beaver", 1);
//!     let playing_online = Relation::new("play online", 1);
//!     let passing_ball = Relation::new("pass the ball", 1);
//!
//!     // Add those to the graph as hyperedges.
//!     let first_relation = graph.add_hyperedge(vec![faarooq, ava, ghanda], cat_video)?;
//!     let second_relation = graph.add_hyperedge(vec![faarooq, ava, ghanda], dog_video)?;
//!     let third_relation = graph.add_hyperedge(vec![ewan, ava, bianca], beaver_video)?;
//!     let fourth_relation = graph.add_hyperedge(vec![daena], playing_online)?;
//!     let fifth_relation = graph.add_hyperedge(vec![ewan, charles, bianca, bianca, ewan], passing_ball)?;
//!
//!     // Each hyperedge gets a unique index by insertion order.
//!     assert_eq!(first_relation, HyperedgeIndex(0));
//!     assert_eq!(fifth_relation, HyperedgeIndex(4));
//!
//!     // Get the weight of a hyperedge.
//!     assert_eq!(graph.get_hyperedge_weight(HyperedgeIndex(0)), Ok(&cat_video));
//!
//!     // Get the vertices of a hyperedge.
//!     assert_eq!(graph.get_hyperedge_vertices(HyperedgeIndex(0)), Ok(vec![faarooq, ava, ghanda]));
//!
//!     // The hypergraph has 5 hyperedges.
//!     assert_eq!(graph.count_hyperedges(), 5);
//!
//!     // Get the hyperedges of a vertex.
//!     assert_eq!(graph.get_vertex_hyperedges(VertexIndex(0)), Ok(vec![first_relation, second_relation, third_relation]));
//!     assert_eq!(graph.get_full_vertex_hyperedges(VertexIndex(0)), Ok(vec![vec![faarooq, ava, ghanda], vec![faarooq, ava, ghanda], vec![ewan, ava, bianca]]));
//!     
//!     // Get the intersection of some hyperedges.
//!     assert_eq!(graph.get_hyperedges_intersections(&[second_relation, third_relation]), Ok(vec![ava]));
//!
//!     // Find a hyperedge containing a connection between two vertices.
//!     assert_eq!(graph.get_hyperedges_connecting(bianca, bianca), Ok(vec![fifth_relation]));
//!
//!     // Get the adjacent vertices from a vertex.
//!     assert_eq!(graph.get_adjacent_vertices_from(VertexIndex(0)), Ok(vec![bianca, ghanda]));
//!
//!     // Get the adjacent vertices to a vertex.
//!     assert_eq!(graph.get_adjacent_vertices_to(VertexIndex(0)), Ok(vec![ewan, faarooq]));
//!
//!     // Find the shortest paths between some vertices.
//!     assert_eq!(graph.get_dijkstra_connections(faarooq, bianca), Ok(vec![(faarooq, None), (ava, Some(first_relation)), (bianca, Some(third_relation))]));
//!
//!     // Update the weight of a vertex.
//!     graph.update_vertex_weight(ava, Person::new("Avā"))?;
//!     
//!     // Update the weight of a hyperedge.
//!     graph.update_hyperedge_weight(third_relation, Relation::new("share a viral video with a capybara", 1))?;
//!
//!     // Update the vertices of a hyperedge.
//!     graph.update_hyperedge_vertices(third_relation, vec![ewan, ava, daena])?;
//!
//!     // Remove a hyperedge.
//!     graph.remove_hyperedge(first_relation)?;
//!
//!     // Remove a vertex.
//!     graph.remove_vertex(ewan)?;
//!
//!     // Reverse a hyperedge.
//!     graph.reverse_hyperedge(fifth_relation)?;
//!
//!     // Get the in-degree of a vertex.
//!     assert_eq!(graph.get_vertex_degree_in(ava), Ok(1));
//!
//!     // Get the out-degree of a vertex.
//!     assert_eq!(graph.get_vertex_degree_out(ghanda), Ok(0));
//!
//!     // Contract a hyperedge's vertices.
//!     graph.contract_hyperedge_vertices(fifth_relation, vec![bianca, charles], bianca)?;
//!
//!     // Join some hyperedges.
//!     graph.join_hyperedges(&[fifth_relation, third_relation]);
//!
//!     // Clear the hyperedges.
//!     graph.clear_hyperedges()?;
//!
//!     // Clear the whole hypergraph.
//!     graph.clear();
//!
//!     Ok(())
//! }
//! ```

#[doc(hidden)]
pub mod core;

// Reexport of the public API.
#[doc(inline)]
pub use crate::core::*;
