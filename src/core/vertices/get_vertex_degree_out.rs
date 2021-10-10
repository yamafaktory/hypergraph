use crate::{
    core::shared::Connection, errors::HypergraphError, Hypergraph, SharedTrait, VertexIndex,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Gets the out-degree of a vertex.
    /// <https://en.wikipedia.org/wiki/Directed_graph#Indegree_and_outdegree>
    pub fn get_vertex_degree_out(
        &self,
        from: VertexIndex,
    ) -> Result<usize, HypergraphError<V, HE>> {
        let results = self.get_connections(Connection::In(from))?;

        Ok(results.len())
    }
}
