use crate::{
    bi_hash_map::BiHashMap,
    core::{shared::Connection, utils::are_slices_equal},
    errors::HypergraphError,
    HyperedgeIndex, HyperedgeKey, Hypergraph, SharedTrait, VertexIndex,
};

use itertools::Itertools;

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    // This private method is infallible since adding the same hyperedge
    // will return the existing index.
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
        // If the provided vertices are empty, skip the update.
        if vertices.is_empty() {
            return Err(HypergraphError::HyperedgeCreationNoVertices(weight));
        }

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

    /// Clears all the hyperedges from the hypergraph.
    pub fn clear_hyperedges(&mut self) -> Result<(), HypergraphError<V, HE>> {
        // Clear the set while keeping its capacity.
        self.hyperedges.clear();

        // Reset the hyperedges mapping.
        self.hyperedges_mapping = BiHashMap::default();

        // Reset the hyperedges counter.
        self.hyperedges_count = 0;

        // Update the vertices accordingly.
        self.vertices
            .iter_mut()
            // Clear the sets while keeping their capacities.
            .for_each(|(_, hyperedges)| hyperedges.clear());

        Ok(())
    }

    /// Returns the number of hyperedges in the hypergraph.
    pub fn count_hyperedges(&self) -> usize {
        self.hyperedges.len()
    }

    /// Gets the hyperedges directly connecting a vertex to another.
    pub fn get_hyperedges_connecting(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        let results = self.get_connections(Connection::InAndOut(from, to))?;

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

    /// Removes a hyperedge by index.
    pub fn remove_hyperedge(
        &mut self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        let internal_index = self.get_internal_hyperedge(hyperedge_index)?;

        let HyperedgeKey { vertices, .. } = self
            .hyperedges
            .get_index(internal_index)
            .map(|hyperedge_key| hyperedge_key.to_owned())
            .ok_or(HypergraphError::InternalHyperedgeIndexNotFound(
                internal_index,
            ))?;

        // Find the last index.
        let last_index = self.hyperedges.len() - 1;

        // Swap and remove by index.
        self.hyperedges.swap_remove_index(internal_index);

        // Update the mapping for the removed hyperedge.
        self.hyperedges_mapping.left.remove(&internal_index);
        self.hyperedges_mapping.right.remove(&hyperedge_index);

        // Remove the hyperedge from the vertices.
        for vertex in vertices.into_iter() {
            match self.vertices.get_index_mut(vertex) {
                Some((_, index_set)) => {
                    index_set.remove(&internal_index);
                }
                None => return Err(HypergraphError::InternalVertexIndexNotFound(vertex)),
            }
        }

        // Given the following bi-mapping with three hyperedges, i.e. an
        // initial set of hyperedges [0, 1, 2]. Let's assume in this example
        // that the first hyperedge will be removed:
        //
        // left                        | Right
        // ---------------------------------------------------------
        // 0usize -> HyperedgeIndex(0) | HyperedgeIndex(0) -> 0usize
        // 1usize -> HyperedgeIndex(1) | HyperedgeIndex(1) -> 1usize
        // 2usize -> HyperedgeIndex(2) | HyperedgeIndex(2) -> 2usize
        //
        // In the previous steps, the current hyperedge has been already nuked.
        // So we now have:
        //
        // left                        | Right
        // ---------------------------------------------------------
        // xxxxxxxxxxxxxxxxxxxxxxxxxxx | xxxxxxxxxxxxxxxxxxxxxxxxxxx
        // 1usize -> HyperedgeIndex(1) | HyperedgeIndex(1) -> 1usize
        // 2usize -> HyperedgeIndex(2) | HyperedgeIndex(2) -> 2usize
        //
        // The next step will be to insert the swapped index on the right:
        //
        // left                        | Right
        // ---------------------------------------------------------
        // xxxxxxxxxxxxxxxxxxxxxxxxxxx | xxxxxxxxxxxxxxxxxxxxxxxxxxx
        // 1usize -> HyperedgeIndex(1) | HyperedgeIndex(1) -> 1usize
        // 2usize -> HyperedgeIndex(2) | HyperedgeIndex(2) -> 0usize
        //
        // Now remove the index which no longer exists on the left:
        //
        // left                        | Right
        // ---------------------------------------------------------
        // xxxxxxxxxxxxxxxxxxxxxxxxxxx | xxxxxxxxxxxxxxxxxxxxxxxxxxx
        // 1usize -> HyperedgeIndex(1) | HyperedgeIndex(1) -> 1usize
        // xxxxxxxxxxxxxxxxxxxxxxxxxxx | HyperedgeIndex(2) -> 0usize
        //
        // Insert the swapped index on the left:
        //
        // left                        | Right
        // ---------------------------------------------------------
        // 0usize -> HyperedgeIndex(2) | xxxxxxxxxxxxxxxxxxxxxxxxxxx
        // 1usize -> HyperedgeIndex(1) | HyperedgeIndex(1) -> 1usize
        // xxxxxxxxxxxxxxxxxxxxxxxxxxx | HyperedgeIndex(2) -> 0usize
        //
        // If the index to remove wasn't the last one, the last hyperedge has
        // been swapped in place of the removed one. Thus we need to update
        // the mapping accordingly.
        if internal_index != last_index {
            // Get the index of the swapped hyperedge.
            let swapped_hyperedge_index = self.get_hyperedge(last_index)?;

            // Proceed with the aforementioned operations.
            self.hyperedges_mapping
                .right
                .insert(swapped_hyperedge_index, internal_index);
            self.hyperedges_mapping.left.remove(&last_index);
            self.hyperedges_mapping
                .left
                .insert(internal_index, swapped_hyperedge_index);

            // Get the vertices of the swapped hyperedge.
            let HyperedgeKey {
                vertices: swapped_vertices,
                ..
            } = self
                .hyperedges
                .get_index(internal_index)
                .map(|hyperedge_key| hyperedge_key.to_owned())
                .ok_or(HypergraphError::InternalHyperedgeIndexNotFound(
                    internal_index,
                ))?;

            // Update the impacted vertices accordingly.
            for vertex in swapped_vertices.into_iter() {
                match self.vertices.get_index_mut(vertex) {
                    Some((_, index_set)) => {
                        // Update the by performing an insertion of the current
                        //  hyperedge and a removal of the swapped one.
                        index_set.insert(internal_index);
                        index_set.remove(&last_index);
                    }
                    None => return Err(HypergraphError::InternalVertexIndexNotFound(vertex)),
                }
            }
        }

        // Return a unit.
        Ok(())
    }

    // Reverses a hyperedge.
    pub fn reverse_hyperedge(
        &mut self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        // Get the vertices of the hyperedge.
        let vertices = self.get_hyperedge_vertices(hyperedge_index)?;

        // Update the hyperedge with the reversed vertices.
        self.update_hyperedge_vertices(hyperedge_index, vertices.into_iter().rev().collect_vec())
    }

    /// Updates the weight of a hyperedge by index.
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
        // Entry to remove
        //  |              1.Insert new entry
        //  |                     |
        //  v                     v
        // [a, b, c] -> [a, b, c, d] -> [d, b, c, _]
        //                               ^        ^
        //                               |        |
        //                               +--------+
        //                         2.Swap and remove
        //
        // -----------------------------------------
        //
        // Second case: no index alteration.
        //
        // Entry to remove
        //        |        1.Insert new entry
        //        |               |
        //        v               v
        // [a, b, c] -> [a, b, c, d] -> [a, b, d, _]
        //                                     ^  ^
        //                                     |  |
        //                                     +--+
        //                         2.Swap and remove
        //
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

    /// Updates the vertices of a hyperedge by index.
    pub fn update_hyperedge_vertices(
        &mut self,
        hyperedge_index: HyperedgeIndex,
        vertices: Vec<VertexIndex>,
    ) -> Result<(), HypergraphError<V, HE>> {
        // If the provided vertices are empty, skip the update.
        if vertices.is_empty() {
            return Err(HypergraphError::HyperedgeUpdateNoVertices(hyperedge_index));
        }

        let internal_index = self.get_internal_hyperedge(hyperedge_index)?;

        let internal_vertices = self.get_internal_vertices(vertices)?;

        let HyperedgeKey {
            vertices: previous_vertices,
            weight,
        } = self
            .hyperedges
            .get_index(internal_index)
            .map(|hyperedge_key| hyperedge_key.to_owned())
            .ok_or(HypergraphError::InternalHyperedgeIndexNotFound(
                internal_index,
            ))?;

        // If the new vertices are the same as the old ones, skip the update.
        if are_slices_equal(&internal_vertices, &previous_vertices) {
            return Err(HypergraphError::HyperedgeVerticesUnchanged(hyperedge_index));
        }

        // Find the vertices which have been added.
        let added = internal_vertices
            .iter()
            .fold(vec![], |mut acc: Vec<usize>, index| {
                if !previous_vertices
                    .iter()
                    .any(|current_index| current_index == index)
                {
                    acc.push(*index)
                }

                acc
            })
            .into_iter()
            .sorted()
            .dedup()
            .collect_vec();

        // Find the vertices which have been removed.
        let removed = previous_vertices
            .iter()
            .filter_map(|index| {
                if !internal_vertices
                    .iter()
                    .any(|current_index| index == current_index)
                {
                    Some(*index)
                } else {
                    None
                }
            })
            .sorted()
            .dedup()
            .collect_vec();

        // Update the added vertices.
        for index in added.into_iter() {
            match self.vertices.get_index_mut(index) {
                Some((_, index_set)) => {
                    index_set.insert(internal_index);
                }
                None => return Err(HypergraphError::InternalVertexIndexNotFound(index)),
            }
        }

        // Update the removed vertices.
        for index in removed.into_iter() {
            match self.vertices.get_index_mut(index) {
                Some((_, index_set)) => {
                    // This has an impact on the internal indexing for the set.
                    // However since this is not exposed to the user - i.e. no
                    // mapping is involved - we can safely perform the operation.
                    index_set.swap_remove_index(internal_index);
                }
                None => return Err(HypergraphError::InternalVertexIndexNotFound(index)),
            }
        }

        // Insert the new entry.
        // Since we are not altering the weight, we can safely perform the
        // operation without checking its output.
        self.hyperedges.insert(HyperedgeKey {
            vertices: internal_vertices,
            weight,
        });

        // Swap and remove by index.
        // Since we know that the internal index is correct, we can safely
        // perform the operation without checking its output.
        self.hyperedges.swap_remove_index(internal_index);

        // Return a unit.
        Ok(())
    }
}
