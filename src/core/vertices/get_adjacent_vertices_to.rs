use rayon::prelude::*;

use crate::{
    core::shared::Connection, errors::HypergraphError, HyperedgeTrait, Hypergraph, VertexIndex,
    VertexTrait,
};

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
        let mut results = self
            .get_connections(&Connection::Out(to))?
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
