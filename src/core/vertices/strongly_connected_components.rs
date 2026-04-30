use ahash::AHashSet;

use crate::{
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns the strongly connected components (SCCs) of the hypergraph using
    /// Kosaraju's algorithm.
    ///
    /// Each SCC is a sorted `Vec<VertexIndex>` of vertices that are mutually
    /// reachable following directed hyperedges. A vertex with no edges forms its
    /// own single-element SCC. The order of the outer `Vec` follows reverse
    /// finish order from the first DFS pass (topological order for DAGs).
    ///
    /// Returns an empty `Vec` for an empty hypergraph.
    pub fn strongly_connected_components(
        &self,
    ) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>> {
        let mut all_vertices: Vec<VertexIndex> =
            self.vertices_mapping.left.values().copied().collect();
        all_vertices.sort();

        // Phase 1 — iterative DFS on the original graph; record finish order.
        let mut visited: AHashSet<VertexIndex> = AHashSet::new();
        let mut finish_order: Vec<VertexIndex> = Vec::new();

        for &start in &all_vertices {
            if visited.contains(&start) {
                continue;
            }

            // Each stack entry: (vertex, exiting).
            // Push (v, false) to enter v, then (v, true) to record finish.
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

        // Phase 2 — iterative DFS on the transposed graph in reverse finish order.
        // The transposed graph is traversed by following incoming edges via
        // `get_adjacent_vertices_to`.
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
}
