use rayon::prelude::*;

use crate::{
    HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait, core::shared::Connection,
    errors::HypergraphError,
};

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
