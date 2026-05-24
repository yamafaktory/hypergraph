use rayon::prelude::*;

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
    /// Returns the unique set of vertices directly reachable from `from` via a
    /// directed hyperedge (i.e. vertices that follow `from` in some hyperedge's
    /// vertex list).
    ///
    /// The result is sorted by [`VertexIndex`] and deduplicated.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `from` does not exist.
    pub fn get_adjacent_vertices_from(
        &self,
        from: VertexIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        let mut results = self
            .get_connections(&Connection::In(from))?
            .into_par_iter()
            .filter_map(|(_, vertex_index)| vertex_index)
            .collect::<Vec<VertexIndex>>();

        // We use `par_sort_unstable` here which means that the order of equal
        // elements is not preserved but this is fine since we dedupe them
        // afterwards.
        results.par_sort_unstable();
        results.dedup();

        Ok(results)
    }
}
