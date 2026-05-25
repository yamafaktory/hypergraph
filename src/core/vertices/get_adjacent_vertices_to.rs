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
    /// Returns the unique set of vertices that have a directed hyperedge leading
    /// into `to` (i.e. vertices that immediately precede `to` in some hyperedge's
    /// vertex list).
    ///
    /// The result is sorted by [`VertexIndex`] and deduplicated.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `to` does not exist.
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
    fn returns_predecessors() {
        let (g, [v0, v1, _v2, _v3], _) = build();
        let got = g.get_adjacent_vertices_to(v1).unwrap();
        assert_eq!(got, vec![v0]);
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.get_adjacent_vertices_to(VertexIndex(99)).is_err());
    }
}
