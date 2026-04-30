use crate::{
    HyperedgeIndex,
    HyperedgeKey,
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Adds a hyperedge connecting `vertices` with the given `weight`.
    /// Returns the stable index of the hyperedge.
    ///
    /// Duplicate weights are allowed — multiple hyperedges may carry the same
    /// weight value. The unique key is the `(vertices, weight)` combination: if
    /// an identical pair already exists the existing [`HyperedgeIndex`] is
    /// returned without creating a duplicate entry.
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

        // We don't care about the second member of the tuple returned from
        // the insertion since this is an infallible operation.
        let (internal_index, _) = self
            .hyperedges
            .insert_full(HyperedgeKey::new(internal_vertices.clone(), weight));

        // Update the vertices so that we keep directly track of the hyperedge.
        for vertex in internal_vertices {
            let (_, index_set) = self
                .vertices
                .get_mut(vertex)
                .ok_or(HypergraphError::InternalVertexIndexNotFound(vertex))?;

            index_set.insert(internal_index);
        }

        Ok(self.add_hyperedge_index(internal_index))
    }
}
