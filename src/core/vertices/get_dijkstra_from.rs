use std::{
    cmp::Ordering,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Visitor {
    distance: usize,
    index: VertexIndex,
}

impl Visitor {
    fn new(distance: usize, index: VertexIndex) -> Self {
        Self { distance, index }
    }
}

impl Ord for Visitor {
    fn cmp(&self, other: &Visitor) -> Ordering {
        other
            .distance
            .cmp(&self.distance)
            .then_with(|| self.index.cmp(&other.index))
    }
}

impl PartialOrd for Visitor {
    fn partial_cmp(&self, other: &Visitor) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns the minimum cost to reach every vertex reachable from `from`.
    ///
    /// The result is a map of `VertexIndex → cost`. The source vertex itself
    /// is always included with cost `0`. Vertices not reachable from `from`
    /// are absent from the map.
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `from` does not exist.
    pub fn get_dijkstra_from(
        &self,
        from: VertexIndex,
    ) -> Result<AHashMap<VertexIndex, usize>, HypergraphError<V, HE>> {
        if !self.vertices.contains_key(&from) {
            return Err(HypergraphError::VertexIndexNotFound(from));
        }

        let mut distances: AHashMap<VertexIndex, usize> = AHashMap::new();
        let mut to_traverse = BinaryHeap::new();

        distances.insert(from, 0);
        to_traverse.push(Visitor::new(0, from));

        while let Some(Visitor { distance, index }) = to_traverse.pop() {
            if distance > distances[&index] {
                continue;
            }

            let neighbors = self.get_full_adjacent_vertices_from(index)?;

            for (vertex_index, hyperedge_indexes) in neighbors {
                let mut min_cost = usize::MAX;
                for hyperedge_index in hyperedge_indexes {
                    let cost: usize = self
                        .get_hyperedge_weight(hyperedge_index)?
                        .to_owned()
                        .into();
                    if cost < min_cost {
                        min_cost = cost;
                    }
                }

                let next_distance = distance + min_cost;
                let is_shorter = distances
                    .get(&vertex_index)
                    .is_none_or(|&current| next_distance < current);

                if is_shorter {
                    distances.insert(vertex_index, next_distance);
                    to_traverse.push(Visitor::new(next_distance, vertex_index));
                }
            }
        }

        Ok(distances)
    }
}
