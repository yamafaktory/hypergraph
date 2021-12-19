use crate::{errors::HypergraphError, HyperedgeIndex, Hypergraph, SharedTrait, VertexIndex};

use std::{cmp::Ordering, collections::BinaryHeap, fmt::Debug};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Gets a list of the shortest path of vertices between two vertices.
    /// The implementation of the algorithm is based on
    /// <https://doc.rust-lang.org/std/collections/binary_heap/#examples>
    pub fn get_dijkstra_connections(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<Vec<(HyperedgeIndex, VertexIndex)>, HypergraphError<V, HE>> {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        struct Cursor {
            distance: usize,
            index: usize,
        }

        impl Cursor {
            fn new(distance: usize, index: usize) -> Self {
                Self { distance, index }
            }
        }

        // Use a custom implementation of Ord as we want a min-heap BinaryHeap.
        impl Ord for Cursor {
            fn cmp(&self, other: &Cursor) -> Ordering {
                other
                    .distance
                    .cmp(&self.distance)
                    .then_with(|| self.distance.cmp(&other.distance))
            }
        }

        impl PartialOrd for Cursor {
            fn partial_cmp(&self, other: &Cursor) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        // Get the internal indexes of the vertices.
        let internal_from = self.get_internal_vertex(from)?;
        let internal_to = self.get_internal_vertex(to)?;

        // We need to initialize a vector of length equal to the number of vertices.
        // The default value, as per Dijkstra, must be set to infinity.
        // A value of usize::MAX is used.
        let mut distances = (0..self.vertices.len())
            .map(|_| usize::MAX)
            .collect::<Vec<usize>>();

        // Create an empty binary heap.
        let mut heap = BinaryHeap::new();

        // Initialize the first vertex to zero.
        distances[internal_from] = 0;

        // Push the first cursor to the heap.
        heap.push(Cursor::new(0, internal_from));

        // Keep track of the traversal path.
        let mut path = Vec::<(HyperedgeIndex, VertexIndex)>::new();

        while let Some(Cursor { distance, index }) = heap.pop() {
            // End of the traversal.
            if index == internal_to {
                // We need to inject the last tuple for the target vertex.
                // We get the connecting hyperedge from the last element of the
                // path.
                let last_connecting_hyperedge = path.last().unwrap().0;

                path.push((last_connecting_hyperedge, self.get_vertex(internal_to)?));

                return Ok(path);
            }

            // Skip if a better path has already been found.
            if distance > distances[index] {
                continue;
            }

            // Get the VertexIndex associated with the internal index.
            // Proceed by finding all the adjacent vertices as a hashmap whose
            // keys are VertexIndex and values are a vector of HyperedgeIndex.
            let mapped_index = self.get_vertex(index)?;
            let indexes = self.get_full_adjacent_vertices_from(mapped_index)?;

            // For every connected vertex, try to find the lowest distance.
            for (vertex_index, hyperedge_indexes) in indexes {
                let vertex_index = self.get_internal_vertex(vertex_index)?;

                let mut min_cost = usize::MAX;
                let mut best_hyperedge: Option<HyperedgeIndex> = None;

                // Get the lower cost.
                for hyperedge_index in hyperedge_indexes.into_iter() {
                    let hyperedge_weight = self.get_hyperedge_weight(hyperedge_index)?;

                    // Use the trait implementation to get the associated cost
                    // of the hyperedge.
                    let cost = hyperedge_weight.into();

                    if cost < min_cost {
                        min_cost = cost;
                        best_hyperedge = Some(hyperedge_index);
                    }
                }

                let next = Cursor::new(distance + min_cost, vertex_index);

                // If so, add it to the frontier and continue.
                if next.distance < distances[next.index] && best_hyperedge.is_some() {
                    dbg!(self.get_vertex(index)?, best_hyperedge);
                    // Update the traversal accordingly.
                    path.push((best_hyperedge.unwrap(), self.get_vertex(index)?));

                    // Push it to the heap.
                    heap.push(next);

                    // Relaxation, we have now found a better way
                    distances[vertex_index] = next.distance;
                }
            }
        }

        // If we reach this point, return an empty vector.
        Ok(vec![])
    }
}
