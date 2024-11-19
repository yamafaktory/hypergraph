use rayon::prelude::*;

use crate::{
    HyperedgeIndex,
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
    /// Gets the hyperedges directly connecting a vertex to another.
    pub fn get_hyperedges_connecting(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        let results = self.get_connections(&Connection::InAndOut(from, to))?;

        Ok(results
            .into_par_iter()
            .map(|(hyperedged_index, _)| hyperedged_index)
            .collect())
    }
}
