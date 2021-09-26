use crate::{errors::HypergraphError, Hypergraph, SharedTrait, VertexIndex};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Updates the weight of a vertex by index.
    pub fn update_vertex_weight(
        &mut self,
        vertex_index: VertexIndex,
        weight: V,
    ) -> Result<(), HypergraphError<V, HE>> {
        let internal_index = self.get_internal_vertex(vertex_index)?;

        let (previous_weight, index_set) = self
            .vertices
            .get_index(internal_index)
            .map(|(previous_weight, index_set)| (previous_weight.to_owned(), index_set.to_owned()))
            .ok_or(HypergraphError::InternalVertexIndexNotFound(internal_index))?;

        // Return an error if the new weight is the same as the previous one.
        if weight == previous_weight {
            return Err(HypergraphError::VertexWeightUnchanged {
                index: vertex_index,
                weight,
            });
        }

        // Return an error if the new weight is already assigned to another
        // vertex.
        if self.vertices.contains_key(&weight) {
            return Err(HypergraphError::VertexWeightAlreadyAssigned(weight));
        }

        // We can't directly replace the value in the map.
        // First, we need to insert the new weight, it will end up
        // being at the last position.
        // Since we have already checked that the new weight is not in the
        // map, we can safely perform the operation without checking its output.
        self.vertices.insert(weight, index_set);

        // Then we use swap and remove. This will remove the previous weight
        // and insert the new one at the index position of the former.
        // This doesn't alter the indexing.
        // See the update_hyperedge_weight method for more detailed explanation.
        // Since we know that the internal index is correct, we can safely
        // perform the operation without checking its output.
        self.vertices.swap_remove_index(internal_index);

        // Return a unit.
        Ok(())
    }
}
