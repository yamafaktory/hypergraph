use crate::{Hypergraph, SharedTrait, VertexIndex};

use indexmap::IndexSet;
use itertools::Itertools;
use std::{cmp::Ordering, collections::BinaryHeap, fmt::Debug};

use super::error::HypergraphError;

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    // This private method is infallible since adding the same vertex
    // will return the existing index.
    fn add_vertex_index(&mut self, internal_index: usize) -> VertexIndex {
        match self.vertices_mapping.left.get(&internal_index) {
            Some(vertex_index) => *vertex_index,
            None => {
                let vertex_index = VertexIndex(self.vertices_count);

                if self
                    .vertices_mapping
                    .left
                    .insert(internal_index, vertex_index)
                    .is_none()
                {
                    // Update the counter only for the first insertion.
                    self.vertices_count += 1;
                }

                self.vertices_mapping
                    .right
                    .insert(vertex_index, internal_index);

                vertex_index
            }
        }
    }

    /// Adds a vertex with a custom weight to the hypergraph.
    /// Returns the index of the vertex.
    pub fn add_vertex(&mut self, weight: V) -> Result<VertexIndex, HypergraphError<V, HE>> {
        self.vertices
            .entry(weight)
            .or_insert(IndexSet::with_capacity(0));

        let internal_index = self
            .vertices
            .get_index_of(&weight)
            .ok_or(HypergraphError::VertexWeightNotFound(weight))?;

        Ok(self.add_vertex_index(internal_index))
    }

    /// Returns the number of vertices in the hypergraph.
    pub fn count_vertices(&self) -> usize {
        self.vertices.len()
    }

    // /// Gets a list of the shortest path of vertices between two vertices.
    // /// The implementation of the algorithm is based on
    // /// <https://doc.rust-lang.org/std/collections/binary_heap/#examples>
    // pub fn get_dijkstra_connections(
    //     &self,
    //     from: StableVertexIndex,
    //     to: StableVertexIndex,
    // ) -> Option<Vec<StableVertexIndex>> {
    //     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    //     struct Cursor {
    //         distance: usize,
    //         index: usize,
    //     }

    //     // Use a custom implementation of Ord as we want a min-heap BinaryHeap.
    //     impl Ord for Cursor {
    //         fn cmp(&self, other: &Cursor) -> Ordering {
    //             other
    //                 .distance
    //                 .cmp(&self.distance)
    //                 .then_with(|| self.distance.cmp(&other.distance))
    //         }
    //     }

    //     impl PartialOrd for Cursor {
    //         fn partial_cmp(&self, other: &Cursor) -> Option<Ordering> {
    //             Some(self.cmp(other))
    //         }
    //     }

    //     // Get the unstable indexes from the mapping.
    //     let maybe_unstable_from = self.vertices_mapping_right.get(&from);
    //     let maybe_unstable_to = self.vertices_mapping_right.get(&to);

    //     // Safe-check: return None if one or the other is None.
    //     if maybe_unstable_from.is_none() || maybe_unstable_to.is_none() {
    //         return None;
    //     }

    //     // We need to initialize a vector of length equal to the number of vertices.
    //     // The default value, as per Dijkstra, must be set to infinity.
    //     // A value of usize::MAX is used.
    //     let mut distances = (0..self.vertices.len()).map(|_| usize::MAX).collect_vec();

    //     // Create an empty binary heap.
    //     let mut heap = BinaryHeap::new();

    //     // Initialize the first vertex to zero.
    //     distances[*maybe_unstable_from.unwrap()] = 0;

    //     // Push the first cursor to the heap.
    //     heap.push(Cursor {
    //         distance: 0,
    //         index: *maybe_unstable_from.unwrap(),
    //     });

    //     // Keep track of the traversal path.
    //     let mut path = Vec::<StableVertexIndex>::new();

    //     while let Some(Cursor { distance, index }) = heap.pop() {
    //         // End of the traversal.
    //         if index == *maybe_unstable_to.unwrap() {
    //             // We need to inject the index of the target vertex.
    //             path.push(to);

    //             // Remove duplicates generated during the iteration of the algorithm.
    //             path.dedup();

    //             return Some(path);
    //         }

    //         // Skip if a better path has already been found.
    //         if distance > distances[index] {
    //             continue;
    //         }

    //         if let Some(stable_vertex_index) = self.vertices_mapping_left.get(&index) {
    //             // For every connected vertex, try to find the lowest distance.
    //             for vertex_index in self.get_adjacent_vertices_to(*stable_vertex_index) {
    //                 let unstable_index = *self.vertices_mapping_right.get(&vertex_index).unwrap();
    //                 let next = Cursor {
    //                     // We assume a distance of one by default.
    //                     distance: distance + 1,
    //                     index: unstable_index,
    //                 };

    //                 // If so, add it to the frontier and continue.
    //                 if next.distance < distances[next.index] {
    //                     // Update the traversal accordingly.
    //                     path.push(*stable_vertex_index);

    //                     // Push it to the heap.
    //                     heap.push(next);

    //                     // Relaxation, we have now found a better way.
    //                     distances[unstable_index] = next.distance;
    //                 }
    //             }
    //         }
    //     }

    //     None
    // }

    // /// Gets all the vertices adjacent to a given vertex.
    // pub fn get_adjacent_vertices_to(&self, from: StableVertexIndex) -> Vec<StableVertexIndex> {
    //     self.get_connections(from, None)
    //         .iter()
    //         .map(|(_, stable_vertex_index)| *stable_vertex_index)
    //         .sorted()
    //         .dedup()
    //         .collect_vec()
    // }

    // /// Gets the hyperedges including vertex as indexes.
    // pub fn get_vertex_hyperedges(
    //     &self,
    //     stable_vertex_index: StableVertexIndex,
    // ) -> Option<Vec<StableHyperedgeWeightedIndex>> {
    //     match self.vertices_mapping_right.get(&stable_vertex_index) {
    //         Some(unstable_vertex_index) => {
    //             self.vertices
    //                 .get_index(*unstable_vertex_index)
    //                 .map(|(_, hyperedges)| {
    //                     hyperedges
    //                         .iter()
    //                         .map(|unstable_hyperedge_weighted_index| {
    //                             // dbg!(
    //                             //     unstable_hyperedge_weighted_index,
    //                             //     self.hyperedges_mapping_left.clone(),
    //                             //     self.hyperedges_mapping_right.clone()
    //                             // );
    //                             *self
    //                                 .hyperedges_mapping_left
    //                                 .get(unstable_hyperedge_weighted_index)
    //                                 .unwrap()
    //                         })
    //                         .collect()
    //                 })
    //         }

    //         None => None,
    //     }
    // }

    // /// Gets the hyperedges including vertex as vectors of vertices.
    // pub fn get_vertex_hyperedges_full(
    //     &self,
    //     stable_vertex_index: StableVertexIndex,
    // ) -> Option<Vec<Vec<StableVertexIndex>>> {
    //     self.get_vertex_hyperedges(stable_vertex_index)
    //         .map(|hyperedges| {
    //             hyperedges
    //                 .iter()
    //                 .map(|index| self.get_hyperedge_vertices(*index).unwrap())
    //                 .collect()
    //         })
    // }

    // /// Gets the weight of a vertex from its index.
    // pub fn get_vertex_weight(&self, stable_vertex_index: StableVertexIndex) -> Option<V> {
    //     match self.vertices_mapping_right.get(&stable_vertex_index) {
    //         Some(unstable_vertex_index) => self
    //             .vertices
    //             .get_index(*unstable_vertex_index)
    //             .map(|(weight, _)| *weight),

    //         None => None,
    //     }
    // }

    // /// Removes a vertex based on its index.
    // pub fn remove_vertex(&mut self, index: StableVertexIndex) -> bool {
    //     // We first need to try to get the unstable index from the mapping.
    //     let maybe_unstable_index = self.vertices_mapping_right.get(&index);

    //     // Safe-check: return false if the index is not in the mapping.
    //     if maybe_unstable_index.is_none() {
    //         return false;
    //     }

    //     // We can safely unwrap for further use.
    //     let unstable_index = *maybe_unstable_index.unwrap();

    //     // IndexMap doesn't allow holes by design, see:
    //     // https://github.com/bluss/indexmap/issues/90#issuecomment-455381877
    //     // As a consequence, we have two options. Either we use shift_remove
    //     // and it will result in an expensive regeneration of all the indexes
    //     // in the map or we use swap_remove and deal with the fact that the last
    //     // element will be swapped in place of the removed one and will thus get
    //     // a new index. We use the latter solution for performance reasons and
    //     // might need to update the mapping accordingly.
    //     let last_index = self.count_vertices() - 1;
    //     let needs_remapping = last_index != unstable_index;
    //     // Get the stable index of latest unstable index, i.e. the swapped one.
    //     let stable_index_of_swapped_one = self.vertices_mapping_left[&last_index];

    //     // Iterate through the hyperedges of the vertex.
    //     for hyperedge in self.get_vertex_hyperedges(index).unwrap().iter() {
    //         // Update the hyperedge's vertices.
    //         self.update_hyperedge_vertices(
    //             *hyperedge,
    //             self.get_hyperedge_vertices(*hyperedge)
    //                 .unwrap()
    //                 .into_iter()
    //                 // Filter out the target vertex which will be dropped.
    //                 .filter(|&stable_vertex_index| stable_vertex_index != index)
    //                 // Last step: we need to take care of the potential remapping.
    //                 .fold(
    //                     vec![],
    //                     |mut acc: Vec<StableVertexIndex>, current_stable_index| {
    //                         acc.push(
    //                             if needs_remapping
    //                                 && stable_index_of_swapped_one == current_stable_index
    //                             {
    //                                 index
    //                             } else {
    //                                 current_stable_index
    //                             },
    //                         );

    //                         acc
    //                     },
    //                 ),
    //         );
    //     }

    //     // Now we can safely remove the vertex.
    //     let key = self.get_vertex_weight(index).unwrap();
    //     let boolean_flag = self.vertices.swap_remove(&key).is_some();

    //     // Update the mapping if necessary.
    //     if needs_remapping {
    //         // Remap the stable index of the swapped one to the current unstable one.
    //         self.vertices_mapping_left
    //             .insert(unstable_index, stable_index_of_swapped_one);
    //         self.vertices_mapping_right
    //             .insert(stable_index_of_swapped_one, unstable_index);
    //     }

    //     // Now remove the garbage stable and unstable indexes.
    //     self.vertices_mapping_left.remove(&last_index);
    //     self.vertices_mapping_right.remove(&index);

    //     // Use the swap and remove operation output as a boolean flag.
    //     boolean_flag
    // }

    // /// Updates the weight of a vertex based on its index.
    // pub fn update_vertex_weight(&mut self, index: StableVertexIndex, weight: V) -> bool {
    //     match self.vertices_mapping_right.get(&index) {
    //         Some(unstable_index) => {
    //             match self.vertices.clone().get_index(*unstable_index) {
    //                 Some((key, value)) => {
    //                     // We can't directly replace the value in the map.
    //                     // First, we need to insert the new weight, it will end up
    //                     // being at the last position.
    //                     self.vertices.insert(weight, value.clone());

    //                     // Then we use swap and remove. It will remove the old weight
    //                     // and insert the new one at the index position of the former.
    //                     self.vertices.swap_remove(key).is_some()
    //                 }
    //                 None => false,
    //             }
    //         }
    //         None => false,
    //     }
    // }
}
