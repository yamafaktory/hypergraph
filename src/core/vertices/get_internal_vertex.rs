use crate::{errors::HypergraphError, Hypergraph, SharedTrait, VertexIndex};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    // Private method to get the internal vertex matching a VertexIndex.
    pub(crate) fn get_internal_vertex(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<usize, HypergraphError<V, HE>> {
        match self.vertices_mapping.right.get(&vertex_index) {
            Some(index) => Ok(*index),
            None => Err(HypergraphError::VertexIndexNotFound(vertex_index)),
        }
    }
}
