use crate::{
    core::shared::Connection, errors::HypergraphError, HyperedgeTrait, Hypergraph, VertexIndex,
    VertexTrait,
};

use itertools::Itertools;

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Gets the list of all vertices connected to a given vertex.
    pub fn get_adjacent_vertices_to(
        &self,
        to: VertexIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        let results = self.get_connections(Connection::Out(to))?;

        Ok(results
            .into_iter()
            .filter_map(|(_, vertex_index)| vertex_index)
            .sorted()
            .dedup()
            .collect_vec())
    }
}
