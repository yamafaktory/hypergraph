use crate::{errors::HypergraphError, Hypergraph, SharedTrait, VertexIndex};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    // Private method to get the internal vertices from a vector of VertexIndex.
    pub(crate) fn get_internal_vertices(
        &self,
        vertices: Vec<VertexIndex>,
    ) -> Result<Vec<usize>, HypergraphError<V, HE>> {
        vertices
            .iter()
            .map(|vertex_index| self.get_internal_vertex(*vertex_index))
            .collect()
    }
}
