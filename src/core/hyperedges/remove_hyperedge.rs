use crate::{
    errors::HypergraphError, HyperedgeIndex, HyperedgeKey, HyperedgeTrait, Hypergraph, VertexTrait,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
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
                        // Perform an insertion of the current hyperedge and a
                        // removal of the swapped one.
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
}
