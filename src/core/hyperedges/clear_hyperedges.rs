use crate::{
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
    /// Removes all hyperedges from the hypergraph.
    ///
    /// Vertices are retained with their weights, but their hyperedge
    /// back-reference sets are cleared. Always returns `Ok(())`.
    pub fn clear_hyperedges(&mut self) -> Result<(), HypergraphError<V, HE>> {
        self.hyperedges.clear();
        self.hyperedges_count = 0;

        for (_, he_set) in self.vertices.values_mut() {
            he_set.clear();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::test_support::build;

    #[test]
    fn clears_all_hyperedges_keeps_vertices() {
        let (mut g, _, _) = build();
        g.clear_hyperedges().unwrap();
        assert_eq!(g.count_hyperedges(), 0);
        assert_eq!(g.count_vertices(), 4);
    }
}
