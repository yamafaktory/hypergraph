use crate::{
    errors::HypergraphError, HyperedgeIndex, HyperedgeKey, HyperedgeTrait, Hypergraph, VertexIndex,
    VertexTrait,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
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
}
