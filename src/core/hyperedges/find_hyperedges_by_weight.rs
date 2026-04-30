use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    Hypergraph,
    VertexTrait,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns the stable indexes of all hyperedges whose weight equals `weight`.
    ///
    /// Multiple hyperedges may share the same weight, so a `Vec` is returned.
    /// Returns an empty `Vec` if no match is found.
    #[must_use]
    pub fn find_hyperedges_by_weight(&self, weight: HE) -> Vec<HyperedgeIndex> {
        self.hyperedges_iter()
            .filter_map(|(idx, w)| (*w == weight).then_some(idx))
            .collect()
    }
}
