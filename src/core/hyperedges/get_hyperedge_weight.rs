use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    Hypergraph,
    VertexTrait,
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns a reference to the weight of the hyperedge at `hyperedge_index`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist.
    pub fn get_hyperedge_weight(
        &self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<&HE, HypergraphError<V, HE>> {
        self.hyperedges
            .get(&hyperedge_index)
            .map(|(_, weight)| weight)
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
    fn returns_weight() {
        let (g, _, [e0, _e1, _e2]) = build();
        assert_eq!(g.get_hyperedge_weight(e0).unwrap(), &E(1));
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.get_hyperedge_weight(HyperedgeIndex(99)).is_err());
    }
}
