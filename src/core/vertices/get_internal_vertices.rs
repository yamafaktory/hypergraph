use crate::{errors::HypergraphError, Hypergraph, SharedTrait, VertexIndex};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    // Private method to get the internal vertices from a vector of VertexIndex.
    pub(crate) fn get_internal_vertices<E: AsRef<Vec<VertexIndex>>>(
        &self,
        vertices: E,
    ) -> Result<Vec<usize>, HypergraphError<V, HE>> {
        vertices
            .as_ref()
            .iter()
            .map(|vertex_index| self.get_internal_vertex(*vertex_index))
            .collect()
    }
}
