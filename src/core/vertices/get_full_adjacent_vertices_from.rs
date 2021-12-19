use crate::{
    core::shared::Connection, errors::HypergraphError, HyperedgeIndex, Hypergraph, SharedTrait,
    VertexIndex,
};

use indexmap::IndexMap;
use itertools::{fold, Itertools};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Gets the list of all vertices connected from a given vertex as tuples
    /// of the form (VertexIndex, Vec<HyperedgeIndex>)
    pub fn get_full_adjacent_vertices_from(
        &self,
        from: VertexIndex,
    ) -> Result<Vec<(VertexIndex, Vec<HyperedgeIndex>)>, HypergraphError<V, HE>> {
        let results = self.get_connections(Connection::In(from))?;

        Ok(fold(
            results,
            IndexMap::<VertexIndex, Vec<HyperedgeIndex>>::new(),
            |mut acc, (hyperedge_index, vertex_index)| {
                if vertex_index.is_some() {
                    let hyperedges = acc.entry(vertex_index.unwrap()).or_insert(vec![]);

                    hyperedges.push(hyperedge_index);
                }

                acc
            },
        )
        .into_iter()
        .collect_vec())
    }
}
