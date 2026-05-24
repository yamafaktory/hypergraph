use crate::{
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
    /// Returns a reference to the weight of the vertex at `vertex_index`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `vertex_index` does
    /// not exist.
    pub fn get_vertex_weight(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<&V, HypergraphError<V, HE>> {
        self.vertices
            .get(&vertex_index)
            .map(|(weight, _)| weight)
            .ok_or(HypergraphError::VertexIndexNotFound(vertex_index))
    }
}
