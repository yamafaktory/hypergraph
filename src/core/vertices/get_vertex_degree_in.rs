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
    /// Gets the in-degree of a vertex.
    /// <https://en.wikipedia.org/wiki/Directed_graph#Indegree_and_outdegree>
    pub fn get_vertex_degree_in(&self, to: VertexIndex) -> Result<usize, HypergraphError<V, HE>> {
        let results = self.get_connections(&Connection::Out(to))?;

        Ok(results.len())
    }
}
