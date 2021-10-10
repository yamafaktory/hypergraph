use crate::{errors::HypergraphError, Hypergraph, SharedTrait, VertexIndex};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    // Private method to get a vector of VertexIndex from a vector of internal indexes.
    pub(crate) fn get_vertices(
        &self,
        vertices: Vec<usize>,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        vertices
            .iter()
            .map(|vertex_index| self.get_vertex(*vertex_index))
            .collect()
    }
}
