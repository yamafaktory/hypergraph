use rayon::prelude::*;

use crate::{
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns the vertex list of every hyperedge that includes `vertex_index`.
    ///
    /// Each element of the outer `Vec` is the ordered vertex list of one
    /// hyperedge, in the same order as returned by
    /// [`get_vertex_hyperedges`](Self::get_vertex_hyperedges).
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `vertex_index` does
    /// not exist.
    pub fn get_full_vertex_hyperedges(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>> {
        self.get_vertex_hyperedges(vertex_index)?
            .into_par_iter()
            .map(|he_idx| self.get_hyperedge_vertices(he_idx))
            .collect()
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
    fn returns_vertex_lists() {
        let (g, [v0, v1, _v2, _v3], _) = build();
        let got = g.get_full_vertex_hyperedges(v0).unwrap();
        assert_eq!(got, vec![vec![v0, v1]]);
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.get_full_vertex_hyperedges(VertexIndex(99)).is_err());
    }
}
