use crate::{HyperedgeVertices, Hypergraph, SharedTrait, VertexIndex};

use indexmap::IndexSet;
use std::{cmp::Ordering, collections::BinaryHeap, fmt::Debug};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Adds a vertex as a custom weight in the hypergraph.
    /// Returns the index of the vertex.
    pub fn add_vertex(&mut self, weight: V) -> VertexIndex {
        self.vertices
            .entry(weight)
            .or_insert(IndexSet::with_capacity(0));

        // Assume that unwrapping the index can't be none due to previous insertion.
        self.vertices.get_index_of(&weight).unwrap()
    }

    /// Returns the number of vertices in the hypergraph.
    pub fn count_vertices(&self) -> usize {
        self.vertices.len()
    }

    /// Gets a list of the shortest path of vertices between two vertices.
    /// The implementation of the algorithm is based on
    /// <https://doc.rust-lang.org/std/collections/binary_heap/#examples>
    pub fn get_dijkstra_connections(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Option<Vec<VertexIndex>> {
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

        // We need to initialize a vector of length equal to the number of vertices.
        // The default value, as per Dijkstra, must be set to infinity.
        // A value of usize::MAX is used.
        let mut distances = (0..self.vertices.len())
            .map(|_| usize::MAX)
            .collect::<Vec<usize>>();

        // Create an empty binary heap.
        let mut heap = BinaryHeap::new();

        // Initialize the first vertex to zero.
        distances[from] = 0;

        // Push the first cursor to the heap.
        heap.push(Cursor {
            distance: 0,
            index: from,
        });

        // Keep track of the traversal path.
        let mut path = Vec::<usize>::new();

        while let Some(Cursor { distance, index }) = heap.pop() {
            // End of the traversal.
            if index == to {
                // We need to inject the index of the target vertex.
                path.push(to);

                // Remove duplicates generated during the iteration of the algorithm.
                path.dedup();

                return Some(path);
            }

            // Skip if a better path has already been found.
            if distance > distances[index] {
                continue;
            }

            // For every connected vertex, try to find the lowest distance.
            for vertex_index in self.get_vertex_connections(index) {
                let next = Cursor {
                    // We assume a distance of one by default.
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

        None
    }

    /// Gets the list of all vertices connected to a given vertex.
    pub fn get_vertex_connections(&self, from: VertexIndex) -> Vec<VertexIndex> {
        self.get_connections(from, None)
    }

    /// Gets the hyperedges as vectors of vertices of a vertex from its index.
    pub fn get_vertex_hyperedges(&self, index: VertexIndex) -> Option<Vec<HyperedgeVertices>> {
        self.vertices
            .get_index(index)
            .map(|(_, hyperedges)| hyperedges)
            .map(|index_set| index_set.iter().cloned().collect())
    }

    /// Gets the weight of a vertex from its index.
    pub fn get_vertex_weight(&self, index: VertexIndex) -> Option<&V> {
        self.vertices.get_index(index).map(|(weight, _)| weight)
    }

    /// Removes a vertex based on its index.
    pub fn remove_vertex(&mut self, index: VertexIndex) -> bool {
        // IndexMap doesn't allow holes by design, see:
        // https://github.com/bluss/indexmap/issues/90#issuecomment-455381877
        // As a consequence, we have two options. Either we use shift_remove
        // and it will result in an expensive regeneration of all the indexes
        // in the map or we use swap_remove and deal with the fact that the last
        // element will be swapped in place of the removed one and will thus get
        // a new index. We use the latter solution for performance reasons.
        match self.vertices.clone().get_index(index) {
            Some((key, _)) => {
                self.vertices.swap_remove(key);
                dbg!(self.hyperedges.clone());
                todo!();
                false
            }
            None => false,
        }
    }

    /// Updates the weight of a vertex based on its index.
    pub fn update_vertex_weight(&mut self, index: VertexIndex, weight: V) -> bool {
        match self.vertices.clone().get_index(index) {
            Some((key, value)) => {
                // We can't directly replace the value in the map.
                // First, we need to insert the new weight, it will end up
                // being at the last position.
                self.vertices.insert(weight, value.to_owned());

                // Then we use swap and remove. It will remove the old weight
                // and insert the new one at the index position of the former.
                self.vertices.swap_remove(key).is_some()
            }
            None => false,
        }
    }
}
