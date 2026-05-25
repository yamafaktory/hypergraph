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
    /// Retains only the hyperedges for which `predicate` returns `true`.
    ///
    /// Vertices are unaffected. The predicate receives the stable index and a
    /// reference to the weight of each hyperedge.
    pub fn retain_hyperedges<F>(&mut self, mut predicate: F) -> Result<(), HypergraphError<V, HE>>
    where
        F: FnMut(HyperedgeIndex, &HE) -> bool,
    {
        let to_remove: Vec<HyperedgeIndex> = self
            .hyperedges_iter()
            .filter_map(|(idx, weight)| (!predicate(idx, weight)).then_some(idx))
            .collect();

        for idx in to_remove {
            self.remove_hyperedge(idx)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Hypergraph,
        core::test_support::{
            E,
            W,
            build,
        },
    };

    #[test]
    fn retains_matching_removes_rest() {
        let (mut g, _, [e0, _e1, _e2]) = build();
        g.retain_hyperedges(|idx, _| idx == e0).unwrap();
        assert_eq!(g.count_hyperedges(), 1);
    }

    #[test]
    fn keep_all_leaves_graph_intact() {
        let (mut g, _, _) = build();
        g.retain_hyperedges(|_, _| true).unwrap();
        assert_eq!(g.count_hyperedges(), 3);
    }
}
