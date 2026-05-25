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
    /// Updates the weight of a hyperedge by index.
    ///
    /// The new weight need not be unique — multiple hyperedges may carry the
    /// same weight value. Returns [`HypergraphError::HyperedgeWeightUnchanged`]
    /// if `weight` equals the current weight (no-op guard).
    pub fn update_hyperedge_weight(
        &mut self,
        hyperedge_index: HyperedgeIndex,
        weight: HE,
    ) -> Result<(), HypergraphError<V, HE>> {
        let (_, current_weight) = self
            .hyperedges
            .get(&hyperedge_index)
            .ok_or(HypergraphError::HyperedgeIndexNotFound(hyperedge_index))?;

        if weight == *current_weight {
            return Err(HypergraphError::HyperedgeWeightUnchanged {
                index: hyperedge_index,
                weight,
            });
        }

        if let Some((_, w)) = self.hyperedges.get_mut(&hyperedge_index) {
            *w = weight;
        }

        Ok(())
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
    fn updates_weight() {
        let (mut g, _, [e0, _e1, _e2]) = build();
        g.update_hyperedge_weight(e0, E(99)).unwrap();
        assert_eq!(g.get_hyperedge_weight(e0).unwrap(), &E(99));
    }

    #[test]
    fn unchanged_weight_returns_error() {
        let (mut g, _, [e0, _e1, _e2]) = build();
        assert!(g.update_hyperedge_weight(e0, E(1)).is_err());
    }

    #[test]
    fn not_found_returns_error() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.update_hyperedge_weight(HyperedgeIndex(99), E(1)).is_err());
    }
}
