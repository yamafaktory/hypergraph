use crate::{Hypergraph, SharedTrait, StableVertexIndex, UnstableVertexIndex};

use indexmap::IndexSet;
use itertools::Itertools;
use std::{cmp::Ordering, collections::BinaryHeap, fmt::Debug};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    fn add_stable_vertex_index(
        &mut self,
        unstable_vertex_index: UnstableVertexIndex,
    ) -> StableVertexIndex {
        match self.vertices_mapping_left.get(&unstable_vertex_index) {
            Some(stable_vertex_index) => *stable_vertex_index,
            None => {
                let stable_index = StableVertexIndex(self.vertices_count);

                self.vertices_mapping_left
                    .insert(unstable_vertex_index, stable_index);
                self.vertices_mapping_right
                    .insert(stable_index, unstable_vertex_index);

                // Update the counter.
                self.vertices_count += 1;

                stable_index
            }
        }
    }

    /// Adds a vertex as a custom weight in the hypergraph.
    /// Returns the index of the vertex.
    pub fn add_vertex(&mut self, weight: V) -> StableVertexIndex {
        self.vertices
            .entry(weight)
            .or_insert(IndexSet::with_capacity(0));

        // Assume that unwrapping the index can't be none due to previous insertion.
        self.add_stable_vertex_index(self.vertices.get_index_of(&weight).unwrap())
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
        from: StableVertexIndex,
        to: StableVertexIndex,
    ) -> Option<Vec<StableVertexIndex>> {
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

        // Get the unstable indexes from the mapping.
        let maybe_unstable_from = self.vertices_mapping_right.get(&from);
        let maybe_unstable_to = self.vertices_mapping_right.get(&to);

        // Safe-check: return None if one or the other is None.
        if maybe_unstable_from.is_none() || maybe_unstable_to.is_none() {
            return None;
        }

        // We need to initialize a vector of length equal to the number of vertices.
        // The default value, as per Dijkstra, must be set to infinity.
        // A value of usize::MAX is used.
        let mut distances = (0..self.vertices.len()).map(|_| usize::MAX).collect_vec();

        // Create an empty binary heap.
        let mut heap = BinaryHeap::new();

        // Initialize the first vertex to zero.
        distances[*maybe_unstable_from.unwrap()] = 0;

        // Push the first cursor to the heap.
        heap.push(Cursor {
            distance: 0,
            index: *maybe_unstable_from.unwrap(),
        });

        // Keep track of the traversal path.
        let mut path = Vec::<StableVertexIndex>::new();

        while let Some(Cursor { distance, index }) = heap.pop() {
            // End of the traversal.
            if index == *maybe_unstable_to.unwrap() {
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

            if let Some(stable_vertex_index) = self.vertices_mapping_left.get(&index) {
                // For every connected vertex, try to find the lowest distance.
                for vertex_index in self.get_adjacent_vertices_to(*stable_vertex_index) {
                    let unstable_index = *self.vertices_mapping_right.get(&vertex_index).unwrap();
                    let next = Cursor {
                        // We assume a distance of one by default.
                        distance: distance + 1,
                        index: unstable_index,
                    };

                    // If so, add it to the frontier and continue.
                    if next.distance < distances[next.index] {
                        // Update the traversal accordingly.
                        path.push(*stable_vertex_index);

                        // Push it to the heap.
                        heap.push(next);

                        // Relaxation, we have now found a better way.
                        distances[unstable_index] = next.distance;
                    }
                }
            }
        }

        None
    }

    /// Gets all the vertices adjacent to a given vertex.
    pub fn get_adjacent_vertices_to(&self, from: StableVertexIndex) -> Vec<StableVertexIndex> {
        self.get_connections(from, None)
            .iter()
            .map(|(_, stable_vertex_index)| *stable_vertex_index)
            .sorted()
            .dedup()
            .collect_vec()
    }

    /// Gets the hyperedges as vectors of vertices of a vertex from its index.
    pub fn get_vertex_hyperedges(
        &self,
        stable_vertex_index: StableVertexIndex,
    ) -> Option<Vec<Vec<StableVertexIndex>>> {
        match self.vertices_mapping_right.get(&stable_vertex_index) {
            Some(unstable_vertex_index) => {
                self.vertices
                    .get_index(*unstable_vertex_index)
                    .map(|(_, hyperedges)| {
                        hyperedges
                            .iter()
                            .map(|unstable_hyperedge_weighted_index| {
                                self.get_hyperedge_vertices(
                                    *self
                                        .hyperedges_mapping_left
                                        .get(unstable_hyperedge_weighted_index)
                                        .unwrap(),
                                )
                                .unwrap()
                            })
                            .collect()
                    })
            }

            None => None,
        }
    }

    /// Gets the weight of a vertex from its index.
    pub fn get_vertex_weight(&self, stable_vertex_index: StableVertexIndex) -> Option<V> {
        match self.vertices_mapping_right.get(&stable_vertex_index) {
            Some(unstable_vertex_index) => self
                .vertices
                .get_index(*unstable_vertex_index)
                .map(|(weight, _)| *weight),

            None => None,
        }
    }

    /// Removes a vertex based on its index.
    /// IndexMap doesn't allow holes by design, see:
    /// https://github.com/bluss/indexmap/issues/90#issuecomment-455381877
    /// As a consequence, we have two options. Either we use shift_remove
    /// and it will result in an expensive regeneration of all the indexes
    /// in the map or we use swap_remove and deal with the fact that the last
    /// element will be swapped in place of the removed one and will thus get
    /// a new index. We use the latter solution for performance reasons.
    pub fn remove_vertex(&mut self, index: StableVertexIndex) -> bool {
        match self.vertices_mapping_right.clone().get(&index) {
            Some(unstable_index) => match self.vertices.clone().get_index(*unstable_index) {
                Some((key, hyperedges)) => {
                    // We preemptively store the eventual index of the swapped
                    // vertex to avoid extra loops.
                    let last_index = self.count_vertices() - 1;
                    let maybe_swapped_index = if last_index == *unstable_index {
                        None
                    } else {
                        Some(last_index)
                    };

                    // First, we need to get all the hyperedges' indexes of the
                    // vertex in order to update them accordingly.
                    for [unstable_hyperdge_index, unstable_hyperedge_weight] in hyperedges.iter() {
                        // Find the stable index.
                        let stable_hyperedge_weighted_index = *self
                            .hyperedges_mapping_left
                            .get(&[*unstable_hyperdge_index, *unstable_hyperedge_weight])
                            .unwrap();

                        let hyperedge_vertices = self
                            .get_hyperedge_vertices(stable_hyperedge_weighted_index)
                            .unwrap();

                        // And update its vertices.
                        self.update_hyperedge_vertices(
                            stable_hyperedge_weighted_index,
                            hyperedge_vertices
                                .iter()
                                .filter_map(|current_stable_index| {
                                    let current_unstable_index = self
                                        .vertices_mapping_right
                                        .get(current_stable_index)
                                        .unwrap();

                                    if current_unstable_index != unstable_index {
                                        // Inject the current index or the swapped one.
                                        match maybe_swapped_index {
                                            Some(swapped_index) => {
                                                Some(if swapped_index == *current_unstable_index {
                                                    index
                                                } else {
                                                    *current_stable_index
                                                })
                                            }
                                            None => Some(*current_stable_index),
                                        }
                                    } else {
                                        None
                                    }
                                })
                                .collect_vec(),
                        );
                    }

                    // We also need to update the other hyperedges which are impacted by this change.
                    if let Some(swapped_index) = maybe_swapped_index {
                        if let Some(hyperedge_weights) = self.get_vertex_hyperedges(
                            *self.vertices_mapping_left.get(&swapped_index).unwrap(),
                        ) {
                            let hyperedge_unstable_weights = hyperedge_weights
                                .iter()
                                .map(|hyperedge_weight| {
                                    hyperedge_weight
                                        .iter()
                                        .map(|stable_vertex_index| {
                                            *self
                                                .vertices_mapping_right
                                                .get(stable_vertex_index)
                                                .unwrap()
                                        })
                                        .collect()
                                })
                                .collect::<Vec<Vec<usize>>>();
                            dbg!(hyperedge_unstable_weights.clone());
                            for weight in hyperedge_unstable_weights.iter() {
                                if let Some(hyperedge_index) = self.hyperedges.get_index_of(weight)
                                {
                                    self.update_hyperedge_vertices(
                                        *self
                                            .hyperedges_mapping_left
                                            .get(&[hyperedge_index, 0])
                                            .unwrap(),
                                        weight
                                            .iter()
                                            .map(|current_unstable_index| {
                                                let current_stable_index = self
                                                    .vertices_mapping_left
                                                    .get(current_unstable_index)
                                                    .unwrap();

                                                if *current_unstable_index == swapped_index {
                                                    index
                                                } else {
                                                    *current_stable_index
                                                }
                                            })
                                            .collect_vec(),
                                    );
                                }
                            }
                        }
                    }

                    // TODO
                    match maybe_swapped_index {
                        Some(a) => {
                            self.vertices_mapping_left.remove(unstable_index);
                            self.vertices_mapping_right.remove(&index);

                            self.vertices_mapping_left.insert(a, index);
                            self.vertices_mapping_right.insert(index, a);
                        }
                        None => {
                            self.vertices_mapping_left.remove(unstable_index);
                            self.vertices_mapping_right.remove(&index);
                        }
                    }

                    // Now we can safely remove the vertex.
                    self.vertices.swap_remove(key).is_some()
                }
                None => false,
            },
            None => false,
        }
    }

    /// Updates the weight of a vertex based on its index.
    pub fn update_vertex_weight(&mut self, index: StableVertexIndex, weight: V) -> bool {
        match self.vertices_mapping_right.get(&index) {
            Some(unstable_index) => {
                match self.vertices.clone().get_index(*unstable_index) {
                    Some((key, value)) => {
                        // We can't directly replace the value in the map.
                        // First, we need to insert the new weight, it will end up
                        // being at the last position.
                        self.vertices.insert(weight, value.clone());

                        // Then we use swap and remove. It will remove the old weight
                        // and insert the new one at the index position of the former.
                        self.vertices.swap_remove(key).is_some()
                    }
                    None => false,
                }
            }
            None => false,
        }
    }
}
