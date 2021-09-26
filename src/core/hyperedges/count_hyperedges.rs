use crate::{Hypergraph, SharedTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Returns the number of hyperedges in the hypergraph.
    pub fn count_hyperedges(&self) -> usize {
        self.hyperedges.len()
    }
}
