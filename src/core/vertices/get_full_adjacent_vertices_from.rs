use indexmap::IndexMap;
use itertools::{fold, Itertools};

use crate::{
    core::shared::Connection, errors::HypergraphError, HyperedgeIndex, HyperedgeTrait, Hypergraph,
    VertexIndex, VertexTrait,
};

#[allow(clippy::type_complexity)]
impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Gets the list of all vertices connected from a given vertex as tuples
    /// of the form (`VertexIndex`, Vec<HyperedgeIndex>).
    pub fn get_full_adjacent_vertices_from(
        &self,
        from: VertexIndex,
    ) -> Result<Vec<(VertexIndex, Vec<HyperedgeIndex>)>, HypergraphError<V, HE>> {
        let results = self.get_connections(&Connection::In(from))?;

        Ok(fold(
            results,
            IndexMap::<VertexIndex, Vec<HyperedgeIndex>>::new(),
            |mut acc, (hyperedge_index, vertex_index)| {
                if let Some(index) = vertex_index {
                    let hyperedges = acc.entry(index).or_default();

                    hyperedges.push(hyperedge_index);
                }

                acc
            },
        )
        .into_iter()
        .collect_vec())
    }
}
