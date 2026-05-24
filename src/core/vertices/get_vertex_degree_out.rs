use crate::{
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    core::shared::Connection,
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns the out-degree of the vertex at `from`.
    ///
    /// The out-degree is the number of directed connections that leave `from`
    /// across all hyperedges (counting each `from → successor` pair once per
    /// hyperedge). See <https://en.wikipedia.org/wiki/Directed_graph#Indegree_and_outdegree>.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `from` does not exist.
    pub fn get_vertex_degree_out(
        &self,
        from: VertexIndex,
    ) -> Result<usize, HypergraphError<V, HE>> {
        let results = self.get_connections(&Connection::In(from))?;

        Ok(results.len())
    }
}
