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
    /// Returns the ordered vertex list of the hyperedge at `hyperedge_index`.
    ///
    /// The order reflects the direction of the hyperedge — i.e. the sequence in
    /// which vertices were provided when the hyperedge was created or last updated.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist.
    pub fn get_hyperedge_vertices(
        &self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        self.hyperedges
            .get(&hyperedge_index)
            .map(|(vertices, _)| vertices.clone())
            .ok_or(HypergraphError::HyperedgeIndexNotFound(hyperedge_index))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        HyperedgeIndex,
        Hypergraph,
        core::test_support::{
            E,
            W,
            build,
        },
    };

    #[test]
    fn returns_vertex_list() {
        let (g, [v0, v1, _v2, _v3], [e0, _e1, _e2]) = build();
        assert_eq!(g.get_hyperedge_vertices(e0).unwrap(), vec![v0, v1]);
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.get_hyperedge_vertices(HyperedgeIndex(99)).is_err());
    }
}
