use crate::{errors::HypergraphError, Hypergraph, SharedTrait, VertexIndex};

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
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        struct Cursor {
            distance: usize,
            index: usize,
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
        heap.push(Cursor {
            distance: 0,
            index: internal_from,
        });

        // Keep track of the traversal path.
        let mut path = Vec::<usize>::new();

        while let Some(Cursor { distance, index }) = heap.pop() {
            // End of the traversal.
            if index == internal_to {
                // We need to inject the index of the target vertex.
                path.push(internal_to);

                // Remove duplicates generated during the iteration of the algorithm.
                path.dedup();

                return self.get_vertices(path);
            }

            // Skip if a better path has already been found.
            if distance > distances[index] {
                continue;
            }

            let mapped_index = self.get_vertex(index)?;
            let indexes = self.get_adjacent_vertices_from(mapped_index)?;
            let internal_indexes = self.get_internal_vertices(indexes)?;

            // For every connected vertex, try to find the lowest distance.
            for vertex_index in internal_indexes {
                let next = Cursor {
                    // We assume a distance of one by default since vertices
                    // have custom weights.
                    distance: distance + 1,
                    index: vertex_index,
                };

                // If so, add it to the frontier and continue.
                if next.distance < distances[next.index] {
                    // Update the traversal accordingly.
                    path.push(index);

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
