use crate::{errors::HypergraphError, HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    // Private method to get the VertexIndex matching an internal index.
    pub(crate) fn get_vertex(
        &self,
        vertex_index: usize,
    ) -> Result<VertexIndex, HypergraphError<V, HE>> {
        match self.vertices_mapping.left.get(&vertex_index) {
            Some(index) => Ok(*index),
            None => Err(HypergraphError::InternalVertexIndexNotFound(vertex_index)),
        }
    }
}
