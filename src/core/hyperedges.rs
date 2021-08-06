use crate::{
    core::utils::are_arrays_equal, HyperedgeIndex, HyperedgeKey, Hypergraph, SharedTrait,
    VertexIndex,
};

use indexmap::IndexSet;
use itertools::{max, Itertools};

use super::error::HypergraphError;

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    fn add_hyperedge_index(&mut self, internal_index: usize) -> HyperedgeIndex {
        match self.hyperedges_mapping.left.get(&internal_index) {
            Some(hyperedge_index) => *hyperedge_index,
            None => {
                let hyperedge_index = HyperedgeIndex(self.hyperedges_count);

                if self
                    .hyperedges_mapping
                    .left
                    .insert(internal_index, hyperedge_index)
                    .is_none()
                {
                    // Update the counter only for the first insertion.
                    self.hyperedges_count += 1;
                }

                self.hyperedges_mapping
                    .right
                    .insert(hyperedge_index, internal_index);

                hyperedge_index
            }
        }
    }

    // Private method to get the HyperedgeIndex matching an internal index.
    pub(crate) fn get_hyperedge(
        &self,
        hyperedge_index: usize,
    ) -> Result<HyperedgeIndex, HypergraphError<V, HE>> {
        match self.hyperedges_mapping.left.get(&hyperedge_index) {
            Some(index) => Ok(*index),
            None => Err(HypergraphError::InternalHyperedgeIndexNotFound(
                hyperedge_index,
            )),
        }
    }

    // Private method to get a vector of HyperedgeIndex from a vector of internal indexes.
    pub(crate) fn get_hyperedges(
        &self,
        hyperedges: Vec<usize>,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        hyperedges
            .iter()
            .map(|hyperedge_index| self.get_hyperedge(*hyperedge_index))
            .collect()
    }

    // Private method to get the internal hyperedge matching a HyperedgeIndex.
    pub(crate) fn get_internal_hyperedge(
        &self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<usize, HypergraphError<V, HE>> {
        match self.hyperedges_mapping.right.get(&hyperedge_index) {
            Some(index) => Ok(*index),
            None => Err(HypergraphError::HyperedgeIndexNotFound(hyperedge_index)),
        }
    }

    // Private method to get the internal hyperedges from a vector of HyperedgeIndex.
    pub(crate) fn get_internal_hyperedges(
        &self,
        hyperedges: Vec<HyperedgeIndex>,
    ) -> Result<Vec<usize>, HypergraphError<V, HE>> {
        hyperedges
            .iter()
            .map(|hyperedge_index| self.get_internal_hyperedge(*hyperedge_index))
            .collect()
    }

    /// Adds a hyperedge as an array of vertices indexes and a custom weight in the hypergraph.
    /// Returns the weighted index of the hyperedge.
    pub fn add_hyperedge(
        &mut self,
        vertices: Vec<VertexIndex>,
        weight: HE,
    ) -> Result<HyperedgeIndex, HypergraphError<V, HE>> {
        // Safely try to get the internal vertices.
        let internal_vertices = self.get_internal_vertices(vertices)?;

        // Return an error if the weight is already assigned to another
        // hyperedge.
        // We can't use the contains method here since the key is a combination
        // of the weight and the vertices.
        if self.hyperedges.iter().any(
            |HyperedgeKey {
                 weight: current_weight,
                 ..
             }| { *current_weight == weight },
        ) {
            return Err(HypergraphError::HyperedgeWeightAlreadyAssigned(weight));
        }

        // We don't care about the second member of the tuple returned from
        // the insertion since this is an infallible operation.
        let (internal_index, _) = self
            .hyperedges
            .insert_full(HyperedgeKey::new(internal_vertices.clone(), weight));

        // Update the vertices so that we keep directly track of the hyperedge.
        for vertex in internal_vertices.into_iter() {
            let (_, index_set) = self
                .vertices
                .get_index_mut(vertex)
                .ok_or(HypergraphError::InternalVertexIndexNotFound(vertex))?;

            index_set.insert(internal_index);
        }

        Ok(self.add_hyperedge_index(internal_index))
    }

    /// Returns the number of hyperedges in the hypergraph.
    pub fn count_hyperedges(&self) -> usize {
        self.vertices.len()
    }

    // // Returns an iterator of all the hyperedges.
    // pub fn get_hyperedges(
    //     &self,
    // ) -> impl Iterator<Item = (&Vec<UnstableVertexIndex>, &IndexSet<HE>)> {
    //     self.hyperedges.iter()
    // }

    /// Gets the hyperedges directly connecting a vertex to another.
    pub fn get_hyperedges_connecting(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        let results = self.get_connections(from, Some(to))?;

        Ok(results
            .into_iter()
            .map(|(hyperedged_index, _)| hyperedged_index)
            .collect_vec())
    }

    /// Gets the vertices of a hyperedge.
    pub fn get_hyperedge_vertices(
        &self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        let internal_index = self.get_internal_hyperedge(hyperedge_index)?;

        let HyperedgeKey { vertices, .. } = self.hyperedges.get_index(internal_index).ok_or(
            HypergraphError::InternalHyperedgeIndexNotFound(internal_index),
        )?;

        self.get_vertices(vertices.to_owned())
    }

    /// Gets the weight of a hyperedge from its index.
    pub fn get_hyperedge_weight(
        &self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<HE, HypergraphError<V, HE>> {
        let internal_index = self.get_internal_hyperedge(hyperedge_index)?;

        let hyperedge_key = self
            .hyperedges
            .get_index(internal_index)
            .ok_or(HypergraphError::InternalVertexIndexNotFound(internal_index))?;

        Ok(hyperedge_key.weight)
    }

    /// Gets the intersections of a set of hyperedges as a vector of vertices.
    pub fn get_hyperedges_intersections(
        &self,
        hyperedges: Vec<HyperedgeIndex>,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        // Keep track of the number of hyperedges.
        let number_of_hyperedges = hyperedges.len();

        // Early exit if less than two hyperedges are provided.
        if number_of_hyperedges < 2 {
            return Err(HypergraphError::HyperedgesIntersections);
        }

        // Get the internal vertices of the hyperedges and keep the eventual error.
        let vertices = hyperedges
            .into_iter()
            .map(
                |hyperedge_index| match self.get_internal_hyperedge(hyperedge_index) {
                    Ok(internal_index) => match self.hyperedges.get_index(internal_index).ok_or(
                        HypergraphError::InternalHyperedgeIndexNotFound(internal_index),
                    ) {
                        Ok(HyperedgeKey { vertices, .. }) => {
                            // Keep the unique vertices.
                            Ok(vertices.iter().unique().cloned().collect_vec())
                        }
                        Err(error) => Err(error),
                    },
                    Err(error) => Err(error),
                },
            )
            .collect::<Result<Vec<Vec<usize>>, HypergraphError<V, HE>>>();

        match vertices {
            Ok(vertices) => {
                self.get_vertices(
                    vertices
                        .into_iter()
                        // Flatten and sort the vertices.
                        .flatten()
                        .sorted()
                        // Map the result to tuples where the second term is an arbitrary value.
                        // The goal is to group them by indexes.
                        .map(|index| (index, 0))
                        .into_group_map()
                        .into_iter()
                        // Filter the groups having the same size as the hyperedge.
                        .filter_map(|(index, occurences)| {
                            if occurences.len() == number_of_hyperedges {
                                Some(index)
                            } else {
                                None
                            }
                        })
                        .sorted()
                        .collect_vec(),
                )
            }
            Err(error) => Err(error),
        }
    }

    // /// Removes a hyperedge based on its index.
    // /// IndexMap doesn't allow holes by design, see:
    // /// https://github.com/bluss/indexmap/issues/90#issuecomment-455381877
    // /// As a consequence, we have two options. Either we use shift_remove
    // /// and it will result in an expensive regeneration of all the indexes
    // /// in the map or we use swap_remove and deal with the fact that the last
    // /// element will be swapped in place of the removed one and will thus get
    // /// a new index. We use the latter solution for performance reasons.\
    // Find the last index.
    // let last_index = self.hyperedges.len() - 1;
    // // Insert the new entry.
    // self.hyperedges.insert_full(HyperedgeKey {
    //     vertices: vertices.to_owned(),
    //     weight,
    // });
    // // Swap and remove by index.
    // self.hyperedges.swap_remove_index(internal_index);
    // // If the index to remove was the last one, no other operations
    // // are needed at this point.
    // if internal_index == last_index {
    //     return Ok(());
    // }
    // pub fn remove_hyperedge(
    //     &mut self,
    //     stable_hyperedge_weighted_index: StableHyperedgeWeightedIndex,
    // ) -> bool {
    //     match self
    //         .hyperedges_mapping_right
    //         .get(&stable_hyperedge_weighted_index)
    //     {
    //         Some([hyperedge_index, _]) => {
    //             match self.hyperedges.clone().get_index(*hyperedge_index) {
    //                 Some((vertices, weights)) => {
    //                     // Either we have multiple weights for the index or only one.
    //                     // In the first case, we only want to drop the weight.
    //                     if weights.len() > 1 {
    //                         let mut new_weights = weights.clone();

    //                         match self.get_hyperedge_weight(stable_hyperedge_weighted_index) {
    //                             Some(weight) => {
    //                                 // Use swap and remove.
    //                                 // This can potentially alter the indexes but only
    //                                 // at the internal hyperedge level, i.e. it doesn't
    //                                 // break the stability of the indexes.
    //                                 new_weights.swap_remove(&weight);

    //                                 // Update with the new weights.
    //                                 self.hyperedges
    //                                     .insert(vertices.clone(), new_weights)
    //                                     .is_some()
    //                             }
    //                             None => false,
    //                         }
    //                     } else {
    //                         // In the second case, we need to remove the hyperedge completely.
    //                         // First update the vertices accordingly.
    //                         for index in vertices.iter() {
    //                             if let Some((weight, vertex)) =
    //                                 self.vertices.clone().get_index(*index)
    //                             {
    //                                 self.vertices.insert(
    //                                     *weight,
    //                                     vertex.iter().fold(
    //                                         IndexSet::new(),
    //                                         |mut new_index_set, hyperedge| {
    //                                             if !are_arrays_equal(hyperedge, vertices) {
    //                                                 new_index_set.insert(*hyperedge);
    //                                             }

    //                                             new_index_set
    //                                         },
    //                                     ),
    //                                 );
    //                             }
    //                         }

    //                         // Finally remove it.
    //                         self.hyperedges.swap_remove(vertices).is_some()
    //                     }
    //                 }
    //                 None => false,
    //             }
    //         }
    //         None => false,
    //     }
    // }

    /// Updates the weight of a hyperedge based on its weighted index.
    pub fn update_hyperedge_weight(
        &mut self,
        hyperedge_index: HyperedgeIndex,
        weight: HE,
    ) -> Result<(), HypergraphError<V, HE>> {
        let internal_index = self.get_internal_hyperedge(hyperedge_index)?;

        let HyperedgeKey {
            vertices,
            weight: previous_weight,
        } = self
            .hyperedges
            .get_index(internal_index)
            .map(|hyperedge_key| hyperedge_key.to_owned())
            .ok_or(HypergraphError::InternalHyperedgeIndexNotFound(
                internal_index,
            ))?;

        // Return an error if the new weight is the same as the previous one.
        if weight == previous_weight {
            return Err(HypergraphError::HyperedgeWeightUnchanged(
                hyperedge_index,
                weight,
            ));
        }

        // Return an error if the new weight is already assigned to another
        // hyperedge.
        // We can't use the contains method here since the key is a combination
        // of the weight and the vertices.
        if self.hyperedges.iter().any(
            |HyperedgeKey {
                 weight: current_weight,
                 ..
             }| { *current_weight == weight },
        ) {
            return Err(HypergraphError::HyperedgeWeightAlreadyAssigned(weight));
        }

        // IndexMap doesn't allow holes by design, see:
        // https://github.com/bluss/indexmap/issues/90#issuecomment-455381877
        //
        // As a consequence, we have two options. Either we use shift_remove
        // and it will result in an expensive regeneration of all the indexes
        // in the map/set or we use swap_remove methods and deal with the fact
        // that the last element will be swapped in place of the removed one
        // and will thus get a new index.
        //
        // In our case, since we are inserting an entry upfront, it circumvents
        // the aforementioned issue.
        //
        // First case: index alteration is avoided.
        //
        // Index to remove
        //  |              1.Insert new entry
        //  |                     |
        //  v                     v
        // [a, b, c] -> [a, b, c, d] -> [d, b, c, x]
        //                               ^        ^
        //                               |        |
        //                               +--------+
        //                         2.Swap and remove
        //
        // -----------------------------------------
        //
        // Second case: no index alteration.
        //
        // Index to remove
        //        |        1.Insert new entry
        //        |               |
        //        v               v
        // [a, b, c] -> [a, b, c, d] -> [a, b, d, x]
        //                                     ^  ^
        //                                     |  |
        //                                     +--+
        //                         2.Swap and remove

        // Insert the new entry.
        // Since we have already checked that the new weight is not in the
        // map, we can safely perform the operation without checking its output.
        self.hyperedges.insert(HyperedgeKey { vertices, weight });

        // Swap and remove by index.
        // Since we know that the internal index is correct, we can safely
        // perform the operation without checking its output.
        self.hyperedges.swap_remove_index(internal_index);

        // Return a unit.
        Ok(())
    }

    // /// Updates the vertices of a hyperedge based on its index.
    // pub fn update_hyperedge_vertices(
    //     &mut self,
    //     stable_hyperedge_weighted_index: StableHyperedgeWeightedIndex,
    //     vertices: Vec<StableVertexIndex>,
    // ) -> bool {
    //     match self
    //         .hyperedges_mapping_right
    //         .clone()
    //         .get(&stable_hyperedge_weighted_index)
    //     {
    //         Some([unstable_hyperedge_index, unstable_hyperedge_weight]) => {
    //             match self.hyperedges.clone().get_index(*unstable_hyperedge_index) {
    //                 Some((key, value)) => {
    //                     let current_hyperedge_weight = self
    //                         .get_hyperedge_weight(stable_hyperedge_weighted_index)
    //                         .unwrap();

    //                     let new_vertices = vertices
    //                         .iter()
    //                         .map(|vertex| *self.vertices_mapping_right.get(vertex).unwrap())
    //                         .collect_vec();

    //                     // If the new vertices are the same as the old ones, skip the update.
    //                     if are_arrays_equal(key, &new_vertices) {
    //                         return true;
    //                     }

    //                     // Keep track of the initial indexes.
    //                     let previous_vertices = self
    //                         .get_hyperedge_vertices(stable_hyperedge_weighted_index)
    //                         .unwrap()
    //                         .iter()
    //                         .map(|vertex| *self.vertices_mapping_right.get(vertex).unwrap())
    //                         .collect_vec();

    //                     // Find the indexes which have been added.
    //                     let added = new_vertices
    //                         .iter()
    //                         .fold(vec![], |mut acc: Vec<usize>, index| {
    //                             if !previous_vertices
    //                                 .iter()
    //                                 .any(|current_index| current_index == index)
    //                             {
    //                                 acc.push(*index)
    //                             }

    //                             acc
    //                         })
    //                         .into_iter()
    //                         .sorted()
    //                         .dedup()
    //                         .collect_vec();

    //                     // Find the indexes which have been removed.
    //                     let removed = previous_vertices
    //                         .iter()
    //                         .filter_map(|index| {
    //                             if !new_vertices
    //                                 .iter()
    //                                 .any(|current_index| index == current_index)
    //                             {
    //                                 Some(*index)
    //                             } else {
    //                                 None
    //                             }
    //                         })
    //                         .sorted()
    //                         .dedup()
    //                         .collect_vec();

    //                     // Finally get the unchanged ones.
    //                     let unchanged = previous_vertices
    //                         .iter()
    //                         .filter_map(|index| {
    //                             if !removed.iter().any(|current_index| index == current_index) {
    //                                 Some(*index)
    //                             } else {
    //                                 None
    //                             }
    //                         })
    //                         .sorted()
    //                         .dedup()
    //                         .collect_vec();

    //                     if value.len() == 1 {
    //                         // Simple case. We can use the insert and swap_remove operations.
    //                         // No index mapping is affected.
    //                         let mut new_index_set = IndexSet::new();
    //                         new_index_set.insert(current_hyperedge_weight);
    //                         self.hyperedges.insert(new_vertices, new_index_set);
    //                         self.hyperedges.swap_remove(key);
    //                     } else {
    //                         // Non-simple case.
    //                         // First, get all the weights.
    //                         let (_, weights) = self
    //                             .hyperedges
    //                             .get_index(*unstable_hyperedge_weight)
    //                             .unwrap();
    //                         let mut new_weights = weights.clone();
    //                         // Remove the one that belongs to the hyperedge.
    //                         // We have to check if the removed entry is the last one or not.
    //                         // If not, we need to adjust the mapping accordingly.
    //                         let not_the_last_one =
    //                             new_weights.len() - 1 != *unstable_hyperedge_weight;
    //                         if not_the_last_one {
    //                             // Not the last one, adjust the mapping.
    //                             let previous_unstable_hyperedge =
    //                                 &[*unstable_hyperedge_index, new_weights.len() - 1];
    //                             let next_hyperedge_index =
    //                                 [*unstable_hyperedge_index, *unstable_hyperedge_weight];
    //                             let stable_index = *self
    //                                 .hyperedges_mapping_left
    //                                 .get(&[*unstable_hyperedge_index, new_weights.len() - 1])
    //                                 .unwrap();
    //                             let vertices = self.get_hyperedge_vertices(stable_index).unwrap();
    //                             self.hyperedges_mapping_left
    //                                 .remove(previous_unstable_hyperedge);
    //                             self.hyperedges_mapping_left
    //                                 .insert(next_hyperedge_index, stable_index);
    //                             self.hyperedges_mapping_right
    //                                 .insert(stable_index, next_hyperedge_index);
    //                             // Update the vertices of this hyperedge.
    //                             for vertex in vertices.into_iter() {
    //                                 //
    //                                 let unstable_vertex_index =
    //                                     self.vertices_mapping_right.get(&vertex).unwrap();
    //                                 let (weight, index_set) =
    //                                     self.vertices.get_index(*unstable_vertex_index).unwrap();
    //                                 // Replace the weighted indexes of the modified hyperedge.
    //                                 let new_index_set = index_set.iter().fold(
    //                                     IndexSet::<UnstableHyperedgeWeightedIndex>::new(),
    //                                     |mut acc, entry| {
    //                                         if are_arrays_equal(entry, previous_unstable_hyperedge)
    //                                         {
    //                                             acc.insert(next_hyperedge_index);
    //                                         } else {
    //                                             acc.insert(*entry);
    //                                         }

    //                                         acc
    //                                     },
    //                                 );
    //                                 self.vertices.insert(*weight, new_index_set);
    //                             }
    //                         }

    //                         // Remove the weight of the current hyperedge.
    //                         new_weights.swap_remove(&current_hyperedge_weight);

    //                         // Update mapping for the other hyperedges.
    //                         // The last weight it going to take the place of the removed one.
    //                         if not_the_last_one {
    //                             let r = self.hyperedges_mapping_left.clone();
    //                             let hyperdge_weighted_index_to_remap = r
    //                                 .get(&[*unstable_hyperedge_index, new_weights.len() - 1])
    //                                 .unwrap();
    //                             dbg!(hyperdge_weighted_index_to_remap);
    //                             // One entry has been dropped from the index set,
    //                             // so we don't need to remove one on the length!
    //                             self.hyperedges_mapping_left
    //                                 .remove(&[*unstable_hyperedge_index, new_weights.len()]);
    //                             self.hyperedges_mapping_left.insert(
    //                                 [*unstable_hyperedge_index, *unstable_hyperedge_weight],
    //                                 *hyperdge_weighted_index_to_remap,
    //                             );
    //                             self.hyperedges_mapping_right.insert(
    //                                 *hyperdge_weighted_index_to_remap,
    //                                 [*unstable_hyperedge_index, *unstable_hyperedge_weight],
    //                             );
    //                             dbg!(
    //                                 // old
    //                                 [*unstable_hyperedge_index, new_weights.len()],
    //                                 // new
    //                                 [*unstable_hyperedge_index, *unstable_hyperedge_weight]
    //                             );
    //                         }

    //                         // Update the entry so that the other hypergedges are kept.
    //                         self.hyperedges.insert(previous_vertices, new_weights);

    //                         // Insert the new hyperedge with its own weight.
    //                         let mut new_index_set = IndexSet::new();
    //                         new_index_set.insert(current_hyperedge_weight);
    //                         let (new_key, _) =
    //                             self.hyperedges.insert_full(new_vertices, new_index_set);
    //                         // Update the mapping.
    //                         // self.hyperedges_mapping_left
    //                         //     .remove(&[*unstable_hyperedge_index, *unstable_hyperedge_weight]);
    //                         self.hyperedges_mapping_left
    //                             .insert([new_key, 0], stable_hyperedge_weighted_index);
    //                         self.hyperedges_mapping_right
    //                             .insert(stable_hyperedge_weighted_index, [new_key, 0])
    //                             .is_some();
    //                     }

    //                     // Update the vertices. Since each vertex holds a
    //                     // reference of the hyperedges, we need to update them
    //                     // accordingly.
    //                     // Process the updated ones.
    //                     for index in unchanged.iter() {
    //                         if let Some((weight, vertex)) = self.vertices.clone().get_index(*index)
    //                         {
    //                             self.vertices.insert(
    //                                 *weight,
    //                                 vertex.iter().fold(
    //                                     IndexSet::new(),
    //                                     |mut new_index_set, hyperedge| {
    //                                         new_index_set.insert(
    //                                             // Insert the new ones if it's a match.
    //                                             if are_arrays_equal(
    //                                                 hyperedge,
    //                                                 &[
    //                                                     *unstable_hyperedge_index,
    //                                                     *unstable_hyperedge_weight,
    //                                                 ],
    //                                             ) {
    //                                                 [
    //                                                     if value.len() > 1 {
    //                                                         self.hyperedges.len() - 1
    //                                                     } else {
    //                                                         *unstable_hyperedge_index
    //                                                     },
    //                                                     if value.len() > 1 {
    //                                                         0
    //                                                     } else {
    //                                                         *unstable_hyperedge_weight
    //                                                     },
    //                                                 ]
    //                                             } else {
    //                                                 *hyperedge
    //                                             },
    //                                         );

    //                                         new_index_set
    //                                     },
    //                                 ),
    //                             );
    //                         };
    //                     }

    //                     // Process the removed ones.
    //                     for index in removed.iter() {
    //                         if let Some((weight, hyperedges)) =
    //                             self.vertices.clone().get_index(*index)
    //                         {
    //                             self.vertices.insert(
    //                                 *weight,
    //                                 hyperedges.iter().fold(
    //                                     IndexSet::new(),
    //                                     |mut new_index_set, hyperedge| {
    //                                         // Skip the removed ones, i.e. if there's no match.
    //                                         if !are_arrays_equal(
    //                                             hyperedge,
    //                                             &[
    //                                                 *unstable_hyperedge_index,
    //                                                 *unstable_hyperedge_weight,
    //                                             ],
    //                                         ) {
    //                                             new_index_set.insert(*hyperedge);
    //                                         }

    //                                         new_index_set
    //                                     },
    //                                 ),
    //                             );
    //                         };
    //                     }

    //                     // Process the added ones.
    //                     for index in added.iter() {
    //                         if let Some((vertex_weight, hyperedges)) =
    //                             self.vertices.to_owned().get_index_mut(*index)
    //                         {
    //                             // Insert the new vertices.
    //                             hyperedges.insert([
    //                                 if value.len() > 1 {
    //                                     self.hyperedges.len() - 1
    //                                 } else {
    //                                     *unstable_hyperedge_index
    //                                 },
    //                                 if value.len() > 1 {
    //                                     0
    //                                 } else {
    //                                     *unstable_hyperedge_weight
    //                                 },
    //                             ]);

    //                             // Update.
    //                             self.vertices.insert(*vertex_weight, hyperedges.to_owned());
    //                         };
    //                     }

    //                     true
    //                 }
    //                 None => false,
    //             }
    //         }
    //         None => false,
    //     }
    // }
}
