use crate::{
    errors::HypergraphError, HyperedgeIndex, HyperedgeKey, Hypergraph, SharedTrait, VertexIndex,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Gets the vertices of a hyperedge.
    pub fn get_hyperedge_vertices(
        &self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        let internal_index = self.get_internal_hyperedge(hyperedge_index)?;

        let HyperedgeKey { vertices, .. } = self.hyperedges.get_index(internal_index).ok_or(
            HypergraphError::InternalHyperedgeIndexNotFound(internal_index),
        )?;

        self.get_vertices(vertices.to_owned())
    }
}
