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
    /// Returns the in-degree of the vertex at `to`.
    ///
    /// The in-degree is the number of directed connections that arrive at `to`
    /// across all hyperedges (counting each `predecessor → to` pair once per
    /// hyperedge). See <https://en.wikipedia.org/wiki/Directed_graph#Indegree_and_outdegree>.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `to` does not exist.
    pub fn get_vertex_degree_in(&self, to: VertexIndex) -> Result<usize, HypergraphError<V, HE>> {
        let results = self.get_connections(&Connection::Out(to))?;

        Ok(results.len())
    }
}
