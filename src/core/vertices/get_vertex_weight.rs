use crate::{errors::HypergraphError, HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Gets the weight of a vertex from its index.
    pub fn get_vertex_weight(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<V, HypergraphError<V, HE>> {
        let internal_index = self.get_internal_vertex(vertex_index)?;

        self.vertices
            .get_index(internal_index)
            .map(|(weight, _)| *weight)
            .ok_or(HypergraphError::InternalVertexIndexNotFound(internal_index))
    }
}
