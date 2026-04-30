use std::{cmp::Ordering, collections::BinaryHeap, iter::successors};

use ahash::AHashMap;

use crate::{
    HyperedgeIndex, HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait, errors::HypergraphError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Visitor {
    distance: usize,
    index: usize,
}

impl Visitor {
    fn new(distance: usize, index: usize) -> Self {
        Self { distance, index }
    }
}

// Use a custom implementation of Ord as we want a min-heap BinaryHeap.
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

type DijkstraResult<V, HE> = Result<
    (usize, Vec<(VertexIndex, Option<HyperedgeIndex>)>),
    HypergraphError<V, HE>,
>;

#[allow(clippy::type_complexity)]
impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    fn dijkstra_impl(&self, from: VertexIndex, to: VertexIndex) -> DijkstraResult<V, HE> {
        let internal_from = self.get_internal_vertex(from)?;
        let internal_to = self.get_internal_vertex(to)?;

        let mut distances: AHashMap<usize, usize> = AHashMap::new();
        // Maps each internal vertex index to its (predecessor, hyperedge used to arrive).
        let mut predecessors: AHashMap<usize, (usize, Option<HyperedgeIndex>)> = AHashMap::new();
        let mut to_traverse = BinaryHeap::new();

        distances.insert(internal_from, 0);
        to_traverse.push(Visitor::new(0, internal_from));

        while let Some(Visitor { distance, index }) = to_traverse.pop() {
            if index == internal_to {
                // Walk the predecessor chain from destination back to source,
                // then reverse to get source-to-destination order.
                let path = successors(Some(internal_to), |&current| {
                    (current != internal_from).then(|| predecessors[&current].0)
                })
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(|internal| {
                    Ok((
                        self.get_vertex(internal)?,
                        predecessors.get(&internal).and_then(|&(_, he)| he),
                    ))
                })
                .collect::<Result<Vec<_>, HypergraphError<V, HE>>>()?;

                return Ok((distance, path));
            }

            // Skip stale heap entries.
            if distance > distances[&index] {
                continue;
            }

            let mapped_index = self.get_vertex(index)?;
            let neighbors = self.get_full_adjacent_vertices_from(mapped_index)?;

            for (vertex_index, hyperedge_indexes) in neighbors {
                let internal_neighbor = self.get_internal_vertex(vertex_index)?;

                // Find the minimum-cost hyperedge to this neighbor.
                let mut min_cost = usize::MAX;
                let mut best_hyperedge: Option<HyperedgeIndex> = None;

                for hyperedge_index in hyperedge_indexes {
                    let cost: usize =
                        self.get_hyperedge_weight(hyperedge_index)?.to_owned().into();

                    if cost < min_cost {
                        min_cost = cost;
                        best_hyperedge = Some(hyperedge_index);
                    }
                }

                let next_distance = distance + min_cost;
                let is_shorter = distances
                    .get(&internal_neighbor)
                    .is_none_or(|&current| next_distance < current);

                if is_shorter {
                    distances.insert(internal_neighbor, next_distance);
                    predecessors.insert(internal_neighbor, (index, best_hyperedge));
                    to_traverse.push(Visitor::new(next_distance, internal_neighbor));
                }
            }
        }

        Ok((0, vec![]))
    }

    /// Gets the cheapest path between two vertices as a vector of
    /// `(VertexIndex, Option<HyperedgeIndex>)` tuples.
    ///
    /// The first element always carries `None` as no hyperedge has been
    /// traversed to reach the starting vertex.
    /// The implementation is based on:
    /// <https://doc.rust-lang.org/std/collections/binary_heap/#examples>
    pub fn get_dijkstra_connections(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<Vec<(VertexIndex, Option<HyperedgeIndex>)>, HypergraphError<V, HE>> {
        self.dijkstra_impl(from, to).map(|(_, path)| path)
    }

    /// Gets the cheapest path between two vertices together with the total cost.
    ///
    /// Returns `(total_cost, path)` where `path` is the same format as
    /// [`get_dijkstra_connections`](Self::get_dijkstra_connections).
    /// When no path exists, returns `(0, [])`.
    pub fn get_dijkstra_connections_with_cost(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<(usize, Vec<(VertexIndex, Option<HyperedgeIndex>)>), HypergraphError<V, HE>> {
        self.dijkstra_impl(from, to)
    }
}
