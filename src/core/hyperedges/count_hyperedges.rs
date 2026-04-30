use crate::{HyperedgeTrait, Hypergraph, VertexTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns the number of hyperedges in the hypergraph.
    #[must_use]
    pub fn count_hyperedges(&self) -> usize {
        self.hyperedges.len()
    }
}
