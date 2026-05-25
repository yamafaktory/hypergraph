use ahash::AHashMap;

use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

/// Shared read/query interface for [`Hypergraph`](crate::Hypergraph) and
/// [`PersistentHypergraph`](crate::PersistentHypergraph).
///
/// Implement the nine required primitive methods; every graph algorithm is
/// provided as a default built on top of those primitives. Concrete types may
/// override any default with a more efficient implementation.
pub trait HypergraphQuery<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns the number of vertices in the hypergraph.
    fn count_vertices(&self) -> usize;

    /// Returns the number of hyperedges in the hypergraph.
    fn count_hyperedges(&self) -> usize;

    /// Returns `true` if the hypergraph contains no vertices.
    fn is_empty(&self) -> bool;

    /// Returns the stable index of every vertex currently in the hypergraph.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] on I/O failure (persistent
    /// backend only; in-memory always returns `Ok`).
    fn vertex_indices(&self) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>>;

    /// Returns the stable index of every hyperedge currently in the hypergraph.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] on I/O failure (persistent
    /// backend only; in-memory always returns `Ok`).
    fn hyperedge_indices(&self) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>>;

    /// Returns the weight of the vertex at `idx`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `idx` does not
    /// exist, or [`HypergraphError::StorageError`] on I/O failure.
    fn get_vertex_weight(&self, idx: VertexIndex) -> Result<V, HypergraphError<V, HE>>;

    /// Returns the weight of the hyperedge at `idx`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `idx` does not
    /// exist, or [`HypergraphError::StorageError`] on I/O failure.
    fn get_hyperedge_weight(&self, idx: HyperedgeIndex) -> Result<HE, HypergraphError<V, HE>>;

    /// Returns the indices of all hyperedges that include `idx`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `idx` does not
    /// exist, or [`HypergraphError::StorageError`] on I/O failure.
    fn get_vertex_hyperedges(
        &self,
        idx: VertexIndex,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>>;

    /// Returns the ordered vertex list of the hyperedge at `idx`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `idx` does not
    /// exist, or [`HypergraphError::StorageError`] on I/O failure.
    fn get_hyperedge_vertices(
        &self,
        idx: HyperedgeIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>>;

    /// Returns the unique set of vertices directly reachable from `from` via a
    /// directed hyperedge.
    ///
    /// A vertex `b` is adjacent from `a` when `a` and `b` appear as consecutive
    /// entries (in that order) in some hyperedge's vertex list. The result is
    /// sorted by [`VertexIndex`] and deduplicated.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `from` does not
    /// exist.
    fn get_adjacent_vertices_from(
        &self,
        from: VertexIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        super::lookups::get_adjacent_vertices_from(self, from)
    }

    /// Returns the unique set of vertices that have a directed connection
    /// leading into `to`.
    ///
    /// A vertex `a` is adjacent to `b` when `a` and `b` appear as consecutive
    /// entries (in that order) in some hyperedge's vertex list. The result is
    /// sorted by [`VertexIndex`] and deduplicated.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `to` does not
    /// exist.
    fn get_adjacent_vertices_to(
        &self,
        to: VertexIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        super::lookups::get_adjacent_vertices_to(self, to)
    }

    /// Returns all vertices directly reachable from `from`, each grouped with
    /// the hyperedges through which they are reached.
    ///
    /// Each element is `(neighbor, hyperedge_indices)`. Use this over
    /// [`get_adjacent_vertices_from`](Self::get_adjacent_vertices_from) when
    /// you also need the carrying hyperedges (e.g. for Dijkstra).
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `from` does not
    /// exist.
    #[allow(clippy::type_complexity)]
    fn get_full_adjacent_vertices_from(
        &self,
        from: VertexIndex,
    ) -> Result<Vec<(VertexIndex, Vec<HyperedgeIndex>)>, HypergraphError<V, HE>> {
        super::lookups::get_full_adjacent_vertices_from(self, from)
    }

    /// Returns all vertices that have a directed connection into `to`, each
    /// grouped with the hyperedges through which they reach it.
    ///
    /// The incoming-edge counterpart of
    /// [`get_full_adjacent_vertices_from`](Self::get_full_adjacent_vertices_from).
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `to` does not
    /// exist.
    #[allow(clippy::type_complexity)]
    fn get_full_adjacent_vertices_to(
        &self,
        to: VertexIndex,
    ) -> Result<Vec<(VertexIndex, Vec<HyperedgeIndex>)>, HypergraphError<V, HE>> {
        super::lookups::get_full_adjacent_vertices_to(self, to)
    }

    /// Returns the in-degree of `to`.
    ///
    /// Counts the number of directed `predecessor → to` consecutive pairs
    /// across all hyperedges (one count per matching pair per hyperedge).
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `to` does not
    /// exist.
    fn get_vertex_degree_in(&self, to: VertexIndex) -> Result<usize, HypergraphError<V, HE>> {
        super::lookups::get_vertex_degree_in(self, to)
    }

    /// Returns the out-degree of `from`.
    ///
    /// Counts the number of directed `from → successor` consecutive pairs
    /// across all hyperedges (one count per matching pair per hyperedge).
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `from` does not
    /// exist.
    fn get_vertex_degree_out(&self, from: VertexIndex) -> Result<usize, HypergraphError<V, HE>> {
        super::lookups::get_vertex_degree_out(self, from)
    }

    /// Returns the indices of all hyperedges that contain a direct `from → to`
    /// consecutive connection.
    ///
    /// A hyperedge qualifies when `from` and `to` appear as adjacent entries
    /// in its vertex list (in that order). Supports self-loops when
    /// `from == to`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `from` does not
    /// exist.
    fn get_hyperedges_connecting(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        super::lookups::get_hyperedges_connecting(self, from, to)
    }

    /// Returns the vertices present in every hyperedge in `hyperedges`.
    ///
    /// The result is sorted by [`VertexIndex`] and deduplicated. Returns an
    /// empty `Vec` when the hyperedges share no common vertices.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgesInvalidIntersections`] if fewer
    /// than two indices are provided, or
    /// [`HypergraphError::HyperedgeIndexNotFound`] if any index does not
    /// exist.
    fn get_hyperedges_intersections(
        &self,
        hyperedges: &[HyperedgeIndex],
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        super::lookups::get_hyperedges_intersections(self, hyperedges)
    }

    /// Returns the stable indices of all hyperedges whose weight equals
    /// `weight`.
    ///
    /// Multiple hyperedges may share the same weight. Returns an empty `Vec`
    /// if no match is found.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] on I/O failure (persistent
    /// backend only).
    #[must_use]
    #[allow(clippy::double_must_use)]
    fn find_hyperedges_by_weight(
        &self,
        weight: HE,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        super::lookups::find_hyperedges_by_weight(self, weight)
    }

    /// Returns the vertex list of every hyperedge that includes `v`.
    ///
    /// Each element of the outer `Vec` is the ordered vertex list of one
    /// hyperedge, in the same order as returned by
    /// [`get_vertex_hyperedges`](Self::get_vertex_hyperedges).
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `v` does not
    /// exist.
    fn get_full_vertex_hyperedges(
        &self,
        v: VertexIndex,
    ) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>> {
        super::lookups::get_full_vertex_hyperedges(self, v)
    }

    /// Returns `true` if at least one vertex with the given `weight` exists.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] on I/O failure (persistent
    /// backend only).
    fn contains_vertex(&self, weight: V) -> Result<bool, HypergraphError<V, HE>> {
        super::lookups::contains_vertex(self, weight)
    }

    /// Returns the stable indices of all vertices whose weight equals `weight`.
    ///
    /// Because vertex weights are not required to be unique, multiple indices
    /// may be returned. Returns an empty `Vec` if no match is found.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] on I/O failure (persistent
    /// backend only).
    fn get_vertex_index(&self, weight: V) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        super::lookups::get_vertex_index(self, weight)
    }

    /// Returns the vertices reachable from `from` in breadth-first order.
    ///
    /// The starting vertex is always the first element of the result. Only
    /// vertices reachable via directed hyperedges are included.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `from` does not
    /// exist.
    fn get_bfs(&self, from: VertexIndex) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        super::traversal::get_bfs(self, from)
    }

    /// Returns the vertices reachable from `from` in depth-first order.
    ///
    /// The starting vertex is always the first element of the result. Only
    /// vertices reachable via directed hyperedges are included.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `from` does not
    /// exist.
    fn get_dfs(&self, from: VertexIndex) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        super::traversal::get_dfs(self, from)
    }

    /// Returns `true` if `to` is reachable from `from` via directed
    /// hyperedges. A vertex is always reachable from itself.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if either index does
    /// not exist.
    fn is_reachable(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<bool, HypergraphError<V, HE>> {
        super::traversal::is_reachable(self, from, to)
    }

    /// Returns `true` if the hypergraph contains no directed cycles.
    ///
    /// Implemented as a topological sort: returns `false` when
    /// [`topological_sort`](Self::topological_sort) would fail.
    fn is_acyclic(&self) -> bool {
        super::structural::is_acyclic(self)
    }

    /// Returns all simple paths (no repeated vertices) from `from` to `to`.
    ///
    /// Each path is a `Vec<VertexIndex>` that includes both endpoints. When
    /// `from == to` the result is `vec![vec![from]]`. Paths are emitted in
    /// DFS discovery order and are not sorted.
    ///
    /// **Warning**: the number of simple paths can grow exponentially with
    /// graph size.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if either index does
    /// not exist.
    fn get_all_paths(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>> {
        super::traversal::get_all_paths(self, from, to)
    }

    /// Returns a topological ordering of all vertices using Kahn's algorithm.
    ///
    /// When multiple vertices are ready at the same step, the one with the
    /// smallest [`VertexIndex`] is chosen, giving a deterministic result.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HypergraphContainsCycle`] if the hypergraph
    /// contains a cycle.
    fn topological_sort(&self) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        super::structural::topological_sort(self)
    }

    /// Returns the strongly connected components (SCCs) of the hypergraph
    /// using Kosaraju's algorithm.
    ///
    /// Each SCC is a sorted `Vec<VertexIndex>` of mutually reachable vertices.
    /// A vertex with no edges forms its own single-element SCC. The order of
    /// the outer `Vec` follows reverse finish order from the first DFS pass.
    /// Returns an empty `Vec` for an empty hypergraph.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] on I/O failure (persistent
    /// backend only).
    fn strongly_connected_components(
        &self,
    ) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>> {
        super::structural::strongly_connected_components(self)
    }

    /// Returns the weakly connected components of the hypergraph.
    ///
    /// Each component is a sorted `Vec<VertexIndex>` of vertices mutually
    /// reachable when edge direction is ignored. Isolated vertices form their
    /// own single-element component. The outer `Vec` is sorted by the smallest
    /// index in each component, giving a deterministic result. Returns an
    /// empty `Vec` for an empty hypergraph.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] on I/O failure (persistent
    /// backend only).
    fn connected_components(&self) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>> {
        super::structural::connected_components(self)
    }

    /// Gets the cheapest path between two vertices as a vector of
    /// `(VertexIndex, Option<HyperedgeIndex>)` tuples.
    ///
    /// The first element always carries `None` as no hyperedge has been
    /// traversed to reach the starting vertex. When no path exists, returns
    /// an empty `Vec`. Uses Dijkstra's algorithm.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if either vertex does
    /// not exist.
    #[allow(clippy::type_complexity)]
    fn get_dijkstra_connections(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<Vec<(VertexIndex, Option<HyperedgeIndex>)>, HypergraphError<V, HE>> {
        super::paths::get_dijkstra_connections(self, from, to)
    }

    /// Gets the cheapest path between two vertices together with the total
    /// cost.
    ///
    /// Returns `(total_cost, path)` where `path` uses the same format as
    /// [`get_dijkstra_connections`](Self::get_dijkstra_connections). When no
    /// path exists, returns `(0, [])`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if either vertex does
    /// not exist.
    #[allow(clippy::type_complexity)]
    fn get_dijkstra_connections_with_cost(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<(usize, Vec<(VertexIndex, Option<HyperedgeIndex>)>), HypergraphError<V, HE>> {
        super::paths::get_dijkstra_connections_with_cost(self, from, to)
    }

    /// Returns the minimum cost to reach every vertex reachable from `from`.
    ///
    /// The result is a map of `VertexIndex → cost`. The source vertex itself
    /// is always included with cost `0`. Vertices not reachable from `from`
    /// are absent from the map.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `from` does not
    /// exist.
    fn get_dijkstra_from(
        &self,
        from: VertexIndex,
    ) -> Result<AHashMap<VertexIndex, usize>, HypergraphError<V, HE>> {
        super::paths::get_dijkstra_from(self, from)
    }

    /// Returns all vertex indices that belong to no hyperedge.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn get_orphan_vertices(&self) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        super::properties::get_orphan_vertices(self)
    }

    /// Returns all hyperedge indices whose vertex list is empty.
    ///
    /// In a well-formed hypergraph this will always be empty, but the method
    /// is provided for completeness and for custom [`HypergraphQuery`]
    /// implementations that allow degenerate hyperedges.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn get_orphan_hyperedges(&self) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        super::properties::get_orphan_hyperedges(self)
    }

    /// Returns `(sources, sinks)` where sources have in-degree 0 and sinks
    /// have out-degree 0.
    ///
    /// A vertex may appear in both lists if it is completely isolated.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn get_endpoints(
        &self,
    ) -> Result<(Vec<VertexIndex>, Vec<VertexIndex>), HypergraphError<V, HE>> {
        super::properties::get_endpoints(self)
    }

    /// Returns all `(subset, superset)` pairs of hyperedge indices where the
    /// vertex set of `subset` is a *proper* subset of the vertex set of
    /// `superset`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn get_inclusions(
        &self,
    ) -> Result<Vec<(HyperedgeIndex, HyperedgeIndex)>, HypergraphError<V, HE>> {
        super::properties::get_inclusions(self)
    }

    /// Returns `true` if every hyperedge contains exactly `k` vertices.
    ///
    /// Returns `true` vacuously on an empty hypergraph.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn is_k_uniform(&self, k: usize) -> Result<bool, HypergraphError<V, HE>> {
        super::properties::is_k_uniform(self, k)
    }

    /// Returns the articulation points (cut vertices) of the hypergraph.
    ///
    /// Each hyperedge is first expanded into an undirected clique (all pairs
    /// of its vertices become undirected edges), then the standard iterative
    /// Tarjan DFS algorithm finds all vertices whose removal would disconnect
    /// the resulting undirected graph.
    ///
    /// The result is sorted.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn find_cut_vertices(&self) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        super::properties::find_cut_vertices(self)
    }

    /// Computes the k-core of the hypergraph via iterative peeling.
    ///
    /// A vertex survives if it participates in at least `min_vertex_degree`
    /// active hyperedges; a hyperedge survives if it spans at least
    /// `min_edge_size` active vertices. Peeling continues until stable.
    ///
    /// Returns `(surviving_vertices, surviving_hyperedges)`, both sorted.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn get_core(
        &self,
        min_vertex_degree: usize,
        min_edge_size: usize,
    ) -> Result<(Vec<VertexIndex>, Vec<HyperedgeIndex>), HypergraphError<V, HE>> {
        super::projections::get_core(self, min_vertex_degree, min_edge_size)
    }

    /// Projects the hypergraph to a directed graph via consecutive vertex pairs.
    ///
    /// For a hyperedge `[v0, v1, v2, …]` the pairs `(v0, v1)`, `(v1, v2)`, …
    /// are emitted. Duplicate pairs are deduplicated. The result is sorted.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn expand_to_graph(&self) -> Result<Vec<(VertexIndex, VertexIndex)>, HypergraphError<V, HE>> {
        super::projections::expand_to_graph(self)
    }

    /// Returns the bipartite vertex–hyperedge membership pairs.
    ///
    /// Each pair `(v, e)` means vertex `v` belongs to hyperedge `e`. Duplicate
    /// memberships (a vertex appearing multiple times in a hyperedge) are
    /// deduplicated. The result is sorted.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn expand_to_star(&self) -> Result<Vec<(VertexIndex, HyperedgeIndex)>, HypergraphError<V, HE>> {
        super::projections::expand_to_star(self)
    }

    /// Computes approximate `PageRank` scores via the iterative power method.
    ///
    /// Out-neighbours of each vertex are determined by
    /// [`get_adjacent_vertices_from`](Self::get_adjacent_vertices_from) with
    /// duplicates removed. Dangling vertices (no out-edges) redistribute their
    /// rank uniformly. After `iterations` steps the scores sum to
    /// approximately `1.0`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn compute_page_rank(
        &self,
        damping: f64,
        iterations: usize,
    ) -> Result<AHashMap<VertexIndex, f64>, HypergraphError<V, HE>> {
        super::projections::compute_page_rank(self, damping, iterations)
    }

    /// Returns `true` if the hypergraph is connected (undirected / clique-expansion sense).
    ///
    /// BFS from the first vertex, treating hyperedges as undirected cliques.
    /// An empty graph is considered connected.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn is_connected(&self) -> Result<bool, HypergraphError<V, HE>> {
        super::structural::is_connected(self)
    }

    /// Returns the neighbourhood of `v`: all vertices co-occurring in any hyperedge with `v`.
    ///
    /// The result excludes `v` itself, is deduplicated, and sorted.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `v` does not exist.
    fn get_vertex_neighborhood(
        &self,
        v: VertexIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        super::lookups::get_vertex_neighborhood(self, v)
    }

    /// Returns the transitive closure as directed reachability pairs `(src, dst)`.
    ///
    /// For each source vertex, follows directed (consecutive-pair) edges via
    /// [`get_adjacent_vertices_from`](Self::get_adjacent_vertices_from). Self-pairs
    /// are excluded. The result is sorted and deduplicated.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn get_transitive_closure(
        &self,
    ) -> Result<Vec<(VertexIndex, VertexIndex)>, HypergraphError<V, HE>> {
        super::structural::get_transitive_closure(self)
    }

    /// Returns the nestedness profile: per-size statistics about hyperedge inclusion.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn get_nestedness_profile(
        &self,
    ) -> Result<Vec<super::NestednessEntry>, HypergraphError<V, HE>> {
        super::properties::get_nestedness_profile(self)
    }

    /// Computes degree, closeness, and betweenness centrality for every vertex.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn compute_centrality(
        &self,
    ) -> Result<AHashMap<VertexIndex, super::CentralityScores>, HypergraphError<V, HE>> {
        super::projections::compute_centrality(self)
    }

    /// Performs a random walk of `steps` from `start` using an Xorshift64 PRNG seeded by `seed`.
    ///
    /// Returns a path of length `steps + 1` including the start vertex.
    /// Seed value `0` is treated as `1`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `start` does not exist.
    fn random_walk(
        &self,
        start: VertexIndex,
        steps: usize,
        seed: u64,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        super::traversal::random_walk(self, start, steps, seed)
    }

    /// Returns the incidence matrix as `(vertex_order, edge_order, matrix)`.
    ///
    /// `matrix[i][j] == 1` iff `vertex_order[i]` appears in `edge_order[j]`.
    /// Both orders are sorted by index.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    #[allow(clippy::type_complexity)]
    fn to_incidence_matrix(
        &self,
    ) -> Result<(Vec<VertexIndex>, Vec<HyperedgeIndex>, Vec<Vec<u8>>), HypergraphError<V, HE>> {
        super::projections::to_incidence_matrix(self)
    }

    /// Returns the sparse (COO) incidence matrix as `(vertex_order, edge_order, coords)`.
    ///
    /// Only `(row, col)` pairs where the value is 1 are included, sorted by `(row, col)`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    #[allow(clippy::type_complexity)]
    fn to_incidence_matrix_coo(
        &self,
    ) -> Result<(Vec<VertexIndex>, Vec<HyperedgeIndex>, Vec<(usize, usize)>), HypergraphError<V, HE>>
    {
        super::projections::to_incidence_matrix_coo(self)
    }

    /// Returns the hypergraph Laplacian as `(vertex_order, n×n matrix)`.
    ///
    /// Uses clique-expansion with hyperedge weights. If `normalized` is `true`,
    /// the normalised Laplacian is returned.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    #[allow(clippy::type_complexity)]
    fn to_laplacian(
        &self,
        normalized: bool,
    ) -> Result<(Vec<VertexIndex>, Vec<Vec<f64>>), HypergraphError<V, HE>> {
        super::projections::to_laplacian(self, normalized)
    }

    /// Returns the line graph: pairs of hyperedges sharing at least one vertex.
    ///
    /// Pairs are unordered `(min, max)`, sorted and deduplicated.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    fn get_line_graph(
        &self,
    ) -> Result<Vec<(HyperedgeIndex, HyperedgeIndex)>, HypergraphError<V, HE>> {
        super::projections::get_line_graph(self)
    }

    /// Returns the dual hypergraph representation.
    ///
    /// For each original vertex (sorted by index), returns the list of incident
    /// hyperedges (sorted). This is the dual: original vertices become "hyperedges"
    /// and original hyperedges become "vertices".
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if the underlying graph primitives fail.
    #[allow(clippy::type_complexity)]
    fn get_dual(&self) -> Result<Vec<(VertexIndex, Vec<HyperedgeIndex>)>, HypergraphError<V, HE>> {
        super::projections::get_dual(self)
    }
}
