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
        let mut all_vertices: Vec<VertexIndex> = self.vertices.keys().copied().collect();
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
}

#[cfg(test)]
mod tests {
    use crate::{
        Hypergraph,
        core::test_support::{
            E,
            W,
        },
    };

    #[test]
    fn two_disconnected_components() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let a = g.add_vertex(W(0)).unwrap();
        let b = g.add_vertex(W(1)).unwrap();
        let c = g.add_vertex(W(2)).unwrap();
        let d = g.add_vertex(W(3)).unwrap();
        g.add_hyperedge(vec![a, b], E(1)).unwrap();
        g.add_hyperedge(vec![c, d], E(1)).unwrap();
        let components = g.connected_components().unwrap();
        assert_eq!(components.len(), 2);
    }

    #[test]
    fn empty_graph_returns_empty_vec() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.connected_components().unwrap().is_empty());
    }
}
