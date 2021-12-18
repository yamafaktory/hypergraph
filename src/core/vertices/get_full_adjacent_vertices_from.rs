use crate::{
    core::shared::Connection, errors::HypergraphError, HyperedgeIndex, Hypergraph, SharedTrait,
    VertexIndex,
};

use itertools::Itertools;

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Gets the list of all vertices connected from a given vertex as tuples
    /// of the form (hyperedge index, vertex index).
    pub fn get_full_adjacent_vertices_from(
        &self,
        from: VertexIndex,
    ) -> Result<Vec<(HyperedgeIndex, VertexIndex)>, HypergraphError<V, HE>> {
        let results = self.get_connections(Connection::In(from))?;

        Ok(results
            .into_iter()
            .filter(|(_, vertex_index)| vertex_index.is_some())
            .map(|(hyperedge_index, vertex_index)| (hyperedge_index, vertex_index.unwrap()))
            .sorted()
            .dedup()
            .collect_vec())
    }
}
