use std::{
    cmp::Reverse,
    collections::BinaryHeap,
};

use ahash::AHashMap;

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
    /// Returns a topological ordering of all vertices using Kahn's algorithm.
    ///
    /// When multiple vertices are ready at the same step, the one with the
    /// smallest [`VertexIndex`] is chosen, giving a deterministic result.
    ///
    /// Returns [`HypergraphError::HypergraphContainsCycle`] if the hypergraph
    /// contains a cycle.
    pub fn topological_sort(&self) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        let all_vertices: Vec<VertexIndex> = self.vertices.keys().copied().collect();
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
}
