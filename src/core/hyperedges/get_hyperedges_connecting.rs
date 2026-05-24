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
    /// Returns the indices of all hyperedges that contain a direct `from → to`
    /// consecutive connection.
    ///
    /// A hyperedge qualifies when `from` and `to` appear as adjacent entries in
    /// its vertex list (in that order). Supports self-loops when `from == to`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if either `from` or `to`
    /// does not exist.
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
