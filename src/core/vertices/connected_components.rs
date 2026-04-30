use std::collections::VecDeque;

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
    /// Returns the weakly connected components of the hypergraph.
    ///
    /// Each component is a sorted `Vec<VertexIndex>` of vertices that are
    /// mutually reachable when edge direction is ignored. Isolated vertices
    /// form their own single-element component. The outer `Vec` is sorted by
    /// the smallest index in each component, giving a deterministic result.
    ///
    /// Returns an empty `Vec` for an empty hypergraph.
    pub fn connected_components(&self) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>> {
        let mut all_vertices: Vec<VertexIndex> =
            self.vertices_mapping.left.values().copied().collect();
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

                // Treat edges as undirected: follow both outgoing and incoming.
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
}
