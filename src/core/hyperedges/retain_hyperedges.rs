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
