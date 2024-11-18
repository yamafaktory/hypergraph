use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    fmt::Debug,
};

use rayon::prelude::*;

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
            .then_with(|| self.distance.cmp(&other.distance))
    }
}

impl PartialOrd for Visitor {
    fn partial_cmp(&self, other: &Visitor) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(clippy::type_complexity)]
impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Gets a list of the cheapest path of vertices between two vertices as a
    /// vector of tuples of the form `(VertexIndex, Option<HyperedgeIndex>)`
    /// where the second member is the hyperedge that has been traversed to
    /// reach the vertex.
    /// Please note that the initial tuple holds `None` as hyperedge since none
    /// has been traversed yet.
    /// The implementation of the algorithm is partially based on:
    /// <https://doc.rust-lang.org/std/collections/binary_heap/#examples>
    pub fn get_dijkstra_connections(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<Vec<(VertexIndex, Option<HyperedgeIndex>)>, HypergraphError<V, HE>> {
        // Get the internal indexes of the vertices.
        let internal_from = self.get_internal_vertex(from)?;
        let internal_to = self.get_internal_vertex(to)?;

        // Keep track of the distances.
        let mut distances = HashMap::new();

        let mut maybe_traversed_hyperedge_by_vertex = HashMap::new();

        // Create an empty binary heap.
        let mut to_traverse = BinaryHeap::new();

        // Initialize the first vertex to zero.
        distances.insert(internal_from, 0);

        // Push the first cursor to the heap.
        to_traverse.push(Visitor::new(0, internal_from));

        // Keep track of the traversal path.
        let mut path = Vec::<VertexIndex>::new();

        while let Some(Visitor { distance, index }) = to_traverse.pop() {
            // End of the traversal.
            if index == internal_to {
                // Inject the target vertex.
                path.push(self.get_vertex(internal_to)?);

                return Ok(path
                    .into_par_iter()
                    .map(|vertex_index| {
                        (
                            vertex_index,
                            maybe_traversed_hyperedge_by_vertex
                                .get(&vertex_index)
                                .and_then(|&current| current),
                        )
                    })
                    .collect());
            }

            // Skip if a better path has already been found.
            if distance > distances[&index] {
                continue;
            }

            // Get the VertexIndex associated with the internal index.
            // Proceed by finding all the adjacent vertices as a hashmap whose
            // keys are VertexIndex and values are a vector of HyperedgeIndex.
            let mapped_index = self.get_vertex(index)?;
            let indexes = self.get_full_adjacent_vertices_from(mapped_index)?;

            // For every connected vertex, try to find the lowest distance.
            for (vertex_index, hyperedge_indexes) in indexes {
                let internal_vertex_index = self.get_internal_vertex(vertex_index)?;

                let mut min_cost = usize::MAX;
                let mut best_hyperedge: Option<HyperedgeIndex> = None;

                // Get the lower cost out of all the hyperedges.
                for hyperedge_index in hyperedge_indexes {
                    let hyperedge_weight = self.get_hyperedge_weight(hyperedge_index)?;

                    // Use the trait implementation to get the associated cost
                    // of the hyperedge.
                    let cost = hyperedge_weight.to_owned().into();

                    if cost < min_cost {
                        min_cost = cost;
                        best_hyperedge = Some(hyperedge_index);

                        break;
                    }
                }

                // Prepare the next visitor.
                let next = Visitor::new(distance + min_cost, internal_vertex_index);

                // Check if this is the shorter distance.
                let is_shorter = distances
                    .get(&next.index)
                    .map_or(true, |&current| next.distance < current);

                // If so, add it to the frontier and continue.
                if is_shorter {
                    maybe_traversed_hyperedge_by_vertex.insert(vertex_index, best_hyperedge);

                    // Update the path traversal accordingly.
                    // Keep vertex indexes unique.
                    if !path
                        .par_iter()
                        .any(|current_index| mapped_index == *current_index)
                    {
                        path.push(mapped_index);
                    }

                    // Push it to the heap.
                    to_traverse.push(next);

                    // Relaxation, we have now found a better way
                    distances.insert(internal_vertex_index, next.distance);
                }
            }
        }

        // If we reach this point, this means that there's no solution.
        // Return an empty vector.
        Ok(vec![])
    }
}
