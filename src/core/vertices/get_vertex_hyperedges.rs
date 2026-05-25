use crate::{
    HyperedgeIndex,
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
    /// Returns the indices of all hyperedges that include `vertex_index`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `vertex_index` does
    /// not exist.
    pub fn get_vertex_hyperedges(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        self.vertices
            .get(&vertex_index)
            .map(|(_, he_set)| he_set.iter().copied().collect())
            .ok_or(HypergraphError::VertexIndexNotFound(vertex_index))
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
    fn returns_hyperedge_indices() {
        let (g, [_v0, v1, _v2, _v3], [e0, e1, e2]) = build();
        let mut got = g.get_vertex_hyperedges(v1).unwrap();
        got.sort();
        assert_eq!(got, vec![e0, e1, e2]);
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.get_vertex_hyperedges(VertexIndex(99)).is_err());
    }
}
