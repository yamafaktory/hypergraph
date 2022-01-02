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
    /// Gets the list of all vertices connected from a given vertex.
    pub fn get_adjacent_vertices_from(
        &self,
        from: VertexIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        let results = self.get_connections(Connection::In(from))?;

        Ok(results
            .into_iter()
            .filter_map(|(_, vertex_index)| vertex_index)
            .sorted()
            .dedup()
            .collect_vec())
    }
}
