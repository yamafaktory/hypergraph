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
    /// Gets the hyperedges directly connecting a vertex to another.
    pub fn get_hyperedges_connecting(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        let results = self.get_connections(Connection::InAndOut(from, to))?;

        Ok(results
            .into_iter()
            .map(|(hyperedged_index, _)| hyperedged_index)
            .collect_vec())
    }
}
