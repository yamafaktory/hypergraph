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

#[cfg(test)]
mod tests {
    use crate::{
        Hypergraph,
        VertexIndex,
        core::test_support::{
            E,
            W,
            build,
        },
    };

    #[test]
    fn returns_direct_neighbors() {
        let (g, [_v0, v1, v2, v3], _) = build();
        let mut got = g.get_adjacent_vertices_from(v1).unwrap();
        got.sort();
        assert_eq!(got, vec![v2, v3]);
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.get_adjacent_vertices_from(VertexIndex(99)).is_err());
    }
}
