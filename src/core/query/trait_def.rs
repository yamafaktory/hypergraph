use std::collections::{
    BinaryHeap,
    VecDeque,
};

use ahash::{
    AHashMap,
    AHashSet,
};

use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    VertexIndex,
    VertexTrait,
    core::shared::Visitor,
    errors::HypergraphError,
};

type DijkstraPath<V, HE> =
    Result<(usize, Vec<(VertexIndex, Option<HyperedgeIndex>)>), HypergraphError<V, HE>>;

fn dijkstra_pair<V, HE, Q>(graph: &Q, from: VertexIndex, to: VertexIndex) -> DijkstraPath<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    graph.get_vertex_weight(from)?;
    graph.get_vertex_weight(to)?;

    let mut distances: AHashMap<VertexIndex, usize> = AHashMap::new();
    let mut predecessors: AHashMap<VertexIndex, (VertexIndex, Option<HyperedgeIndex>)> =
        AHashMap::new();
    let mut heap = BinaryHeap::new();

    distances.insert(from, 0);
    heap.push(Visitor::new(0, from));

    while let Some(Visitor { distance, index }) = heap.pop() {
        if index == to {
            let mut path = Vec::new();
            let mut cur = to;
            while cur != from {
                let (prev, he) = predecessors[&cur];
                path.push((cur, he));
                cur = prev;
            }
            path.push((from, None));
            path.reverse();
            return Ok((distance, path));
        }

        if distance > distances[&index] {
            continue;
        }

        for (neighbor, he_indices) in graph.get_full_adjacent_vertices_from(index)? {
            let mut min_cost = usize::MAX;
            let mut best_he: Option<HyperedgeIndex> = None;
            for he_idx in he_indices {
                let cost: usize = graph.get_hyperedge_weight(he_idx)?.into();
                if cost < min_cost {
                    min_cost = cost;
                    best_he = Some(he_idx);
                }
            }
            let next = distance + min_cost;
            if distances.get(&neighbor).is_none_or(|&d| next < d) {
                distances.insert(neighbor, next);
                predecessors.insert(neighbor, (index, best_he));
                heap.push(Visitor::new(next, neighbor));
            }
        }
    }

    Ok((0, vec![]))
}

fn dijkstra_from<V, HE, Q>(
    graph: &Q,
    from: VertexIndex,
) -> Result<AHashMap<VertexIndex, usize>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    graph.get_vertex_weight(from)?;

    let mut distances: AHashMap<VertexIndex, usize> = AHashMap::new();
    let mut heap = BinaryHeap::new();

    distances.insert(from, 0);
    heap.push(Visitor::new(0, from));

    while let Some(Visitor { distance, index }) = heap.pop() {
        if distance > distances[&index] {
            continue;
        }

        for (neighbor, he_indices) in graph.get_full_adjacent_vertices_from(index)? {
            let mut min_cost = usize::MAX;
            for he_idx in he_indices {
                let cost: usize = graph.get_hyperedge_weight(he_idx)?.into();
                if cost < min_cost {
                    min_cost = cost;
                }
            }
            let next = distance + min_cost;
            if distances.get(&neighbor).is_none_or(|&d| next < d) {
                distances.insert(neighbor, next);
                heap.push(Visitor::new(next, neighbor));
            }
        }
    }

    Ok(distances)
}

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
        let he_indices = self.get_vertex_hyperedges(from)?;
        let mut neighbors: Vec<VertexIndex> = Vec::new();
        for he_idx in he_indices {
            let vertices = self.get_hyperedge_vertices(he_idx)?;
            for w in vertices.windows(2) {
                if w[0] == from && !neighbors.contains(&w[1]) {
                    neighbors.push(w[1]);
                }
            }
        }
        neighbors.sort();
        Ok(neighbors)
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
        let he_indices = self.get_vertex_hyperedges(to)?;
        let mut predecessors: Vec<VertexIndex> = Vec::new();
        for he_idx in he_indices {
            let vertices = self.get_hyperedge_vertices(he_idx)?;
            for w in vertices.windows(2) {
                if w[1] == to && !predecessors.contains(&w[0]) {
                    predecessors.push(w[0]);
                }
            }
        }
        predecessors.sort();
        Ok(predecessors)
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
        let he_indices = self.get_vertex_hyperedges(from)?;
        let mut map: AHashMap<VertexIndex, Vec<HyperedgeIndex>> = AHashMap::new();
        for he_idx in he_indices {
            let vertices = self.get_hyperedge_vertices(he_idx)?;
            for w in vertices.windows(2) {
                if w[0] == from {
                    map.entry(w[1]).or_default().push(he_idx);
                }
            }
        }
        Ok(map.into_iter().collect())
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
        let he_indices = self.get_vertex_hyperedges(to)?;
        let mut map: AHashMap<VertexIndex, Vec<HyperedgeIndex>> = AHashMap::new();
        for he_idx in he_indices {
            let vertices = self.get_hyperedge_vertices(he_idx)?;
            for w in vertices.windows(2) {
                if w[1] == to {
                    map.entry(w[0]).or_default().push(he_idx);
                }
            }
        }
        Ok(map.into_iter().collect())
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
        let he_indices = self.get_vertex_hyperedges(to)?;
        let mut count = 0;
        for he_idx in he_indices {
            let vertices = self.get_hyperedge_vertices(he_idx)?;
            for w in vertices.windows(2) {
                if w[1] == to {
                    count += 1;
                }
            }
        }
        Ok(count)
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
        let he_indices = self.get_vertex_hyperedges(from)?;
        let mut count = 0;
        for he_idx in he_indices {
            let vertices = self.get_hyperedge_vertices(he_idx)?;
            for w in vertices.windows(2) {
                if w[0] == from {
                    count += 1;
                }
            }
        }
        Ok(count)
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
        let he_indices = self.get_vertex_hyperedges(from)?;
        let mut result = Vec::new();
        for he_idx in he_indices {
            let vertices = self.get_hyperedge_vertices(he_idx)?;
            for w in vertices.windows(2) {
                if w[0] == from && w[1] == to {
                    result.push(he_idx);
                }
            }
        }
        Ok(result)
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
        if hyperedges.len() < 2 {
            return Err(HypergraphError::HyperedgesInvalidIntersections);
        }
        let vertex_sets: Vec<Vec<VertexIndex>> = hyperedges
            .iter()
            .map(|&he_idx| self.get_hyperedge_vertices(he_idx))
            .collect::<Result<Vec<_>, _>>()?;

        let mut result: Vec<VertexIndex> = vertex_sets[0]
            .iter()
            .filter(|v| vertex_sets[1..].iter().all(|s| s.contains(v)))
            .copied()
            .collect();
        result.sort();
        result.dedup();
        Ok(result)
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
    fn find_hyperedges_by_weight(
        &self,
        weight: HE,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        let mut result = Vec::new();
        for idx in self.hyperedge_indices()? {
            if self.get_hyperedge_weight(idx)? == weight {
                result.push(idx);
            }
        }
        Ok(result)
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
        self.get_vertex_hyperedges(v)?
            .into_iter()
            .map(|he_idx| self.get_hyperedge_vertices(he_idx))
            .collect()
    }

    /// Returns `true` if at least one vertex with the given `weight` exists.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] on I/O failure (persistent
    /// backend only).
    fn contains_vertex(&self, weight: V) -> Result<bool, HypergraphError<V, HE>> {
        for idx in self.vertex_indices()? {
            if self.get_vertex_weight(idx)? == weight {
                return Ok(true);
            }
        }
        Ok(false)
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
        let mut result = Vec::new();
        for idx in self.vertex_indices()? {
            if self.get_vertex_weight(idx)? == weight {
                result.push(idx);
            }
        }
        Ok(result)
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
        self.get_vertex_weight(from)?;

        let mut visited: AHashSet<VertexIndex> = AHashSet::new();
        let mut queue: VecDeque<VertexIndex> = VecDeque::new();
        let mut result: Vec<VertexIndex> = Vec::new();

        visited.insert(from);
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            result.push(current);
            for neighbor in self.get_adjacent_vertices_from(current)? {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }

        Ok(result)
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
        self.get_vertex_weight(from)?;

        let mut visited: AHashSet<VertexIndex> = AHashSet::new();
        let mut stack: Vec<VertexIndex> = vec![from];
        let mut result: Vec<VertexIndex> = Vec::new();

        while let Some(current) = stack.pop() {
            if visited.insert(current) {
                result.push(current);
                let neighbors = self.get_adjacent_vertices_from(current)?;
                for neighbor in neighbors.into_iter().rev() {
                    if !visited.contains(&neighbor) {
                        stack.push(neighbor);
                    }
                }
            }
        }

        Ok(result)
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
        self.get_vertex_weight(from)?;
        self.get_vertex_weight(to)?;

        if from == to {
            return Ok(true);
        }

        let mut visited: AHashSet<VertexIndex> = AHashSet::new();
        let mut queue: VecDeque<VertexIndex> = VecDeque::new();

        visited.insert(from);
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.get_adjacent_vertices_from(current)? {
                if neighbor == to {
                    return Ok(true);
                }
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }

        Ok(false)
    }

    /// Returns `true` if the hypergraph contains no directed cycles.
    ///
    /// Implemented as a topological sort: returns `false` when
    /// [`topological_sort`](Self::topological_sort) would fail.
    fn is_acyclic(&self) -> bool {
        self.topological_sort().is_ok()
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
        self.get_vertex_weight(from)?;
        self.get_vertex_weight(to)?;

        if from == to {
            return Ok(vec![vec![from]]);
        }

        let mut all_paths: Vec<Vec<VertexIndex>> = Vec::new();
        let mut current_path: Vec<VertexIndex> = vec![from];
        let mut visited: AHashSet<VertexIndex> = AHashSet::from([from]);
        let mut stack: Vec<(VertexIndex, Vec<VertexIndex>, usize)> =
            vec![(from, self.get_adjacent_vertices_from(from)?, 0)];

        while let Some(frame) = stack.last_mut() {
            let (current, neighbors, idx) = frame;
            let current = *current;

            if *idx >= neighbors.len() {
                stack.pop();
                current_path.pop();
                visited.remove(&current);
                continue;
            }

            let next = neighbors[*idx];
            *idx += 1;

            if visited.contains(&next) {
                continue;
            }

            if next == to {
                let mut path = current_path.clone();
                path.push(to);
                all_paths.push(path);
                continue;
            }

            visited.insert(next);
            current_path.push(next);
            let next_neighbors = self.get_adjacent_vertices_from(next)?;
            stack.push((next, next_neighbors, 0));
        }

        Ok(all_paths)
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
        use std::{
            cmp::Reverse,
            collections::BinaryHeap,
        };

        let all_vertices = self.vertex_indices()?;
        let vertex_count = all_vertices.len();

        let mut in_degree: AHashMap<VertexIndex, usize> =
            all_vertices.iter().map(|&v| (v, 0)).collect();

        for &v in &all_vertices {
            for neighbor in self.get_adjacent_vertices_from(v)? {
                *in_degree.entry(neighbor).or_insert(0) += 1;
            }
        }

        let mut heap: BinaryHeap<Reverse<VertexIndex>> = in_degree
            .iter()
            .filter_map(|(&v, &deg)| (deg == 0).then_some(Reverse(v)))
            .collect();

        let mut result: Vec<VertexIndex> = Vec::with_capacity(vertex_count);

        while let Some(Reverse(current)) = heap.pop() {
            result.push(current);
            for neighbor in self.get_adjacent_vertices_from(current)? {
                let deg = in_degree.entry(neighbor).or_insert(0);
                *deg -= 1;
                if *deg == 0 {
                    heap.push(Reverse(neighbor));
                }
            }
        }

        if result.len() == vertex_count {
            Ok(result)
        } else {
            Err(HypergraphError::HypergraphContainsCycle)
        }
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
        let mut all_vertices = self.vertex_indices()?;
        all_vertices.sort();

        let mut visited: AHashSet<VertexIndex> = AHashSet::new();
        let mut finish_order: Vec<VertexIndex> = Vec::new();

        for &start in &all_vertices {
            if visited.contains(&start) {
                continue;
            }
            let mut stack: Vec<(VertexIndex, bool)> = vec![(start, false)];
            while let Some((v, exiting)) = stack.pop() {
                if exiting {
                    finish_order.push(v);
                    continue;
                }
                if !visited.insert(v) {
                    continue;
                }
                stack.push((v, true));
                for neighbor in self.get_adjacent_vertices_from(v)? {
                    if !visited.contains(&neighbor) {
                        stack.push((neighbor, false));
                    }
                }
            }
        }

        let mut visited2: AHashSet<VertexIndex> = AHashSet::new();
        let mut sccs: Vec<Vec<VertexIndex>> = Vec::new();

        for &start in finish_order.iter().rev() {
            if visited2.contains(&start) {
                continue;
            }
            let mut scc: Vec<VertexIndex> = Vec::new();
            let mut stack: Vec<VertexIndex> = vec![start];
            visited2.insert(start);
            while let Some(v) = stack.pop() {
                scc.push(v);
                for predecessor in self.get_adjacent_vertices_to(v)? {
                    if visited2.insert(predecessor) {
                        stack.push(predecessor);
                    }
                }
            }
            scc.sort();
            sccs.push(scc);
        }

        Ok(sccs)
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
        let mut all_vertices = self.vertex_indices()?;
        all_vertices.sort();

        let mut visited: AHashSet<VertexIndex> = AHashSet::new();
        let mut components: Vec<Vec<VertexIndex>> = Vec::new();

        for start in all_vertices {
            if visited.contains(&start) {
                continue;
            }
            let mut component: Vec<VertexIndex> = Vec::new();
            let mut queue: VecDeque<VertexIndex> = VecDeque::new();
            visited.insert(start);
            queue.push_back(start);
            while let Some(current) = queue.pop_front() {
                component.push(current);
                for neighbor in self.get_adjacent_vertices_from(current)? {
                    if visited.insert(neighbor) {
                        queue.push_back(neighbor);
                    }
                }
                for neighbor in self.get_adjacent_vertices_to(current)? {
                    if visited.insert(neighbor) {
                        queue.push_back(neighbor);
                    }
                }
            }
            component.sort();
            components.push(component);
        }

        Ok(components)
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
        dijkstra_pair(self, from, to).map(|(_, path)| path)
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
        dijkstra_pair(self, from, to)
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
        dijkstra_from(self, from)
    }
}
