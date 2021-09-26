use crate::{errors::HypergraphError, HyperedgeIndex, Hypergraph, SharedTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    // Private method to get the internal hyperedge matching a HyperedgeIndex.
    pub(crate) fn get_internal_hyperedge(
        &self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<usize, HypergraphError<V, HE>> {
        match self.hyperedges_mapping.right.get(&hyperedge_index) {
            Some(index) => Ok(*index),
            None => Err(HypergraphError::HyperedgeIndexNotFound(hyperedge_index)),
        }
    }
}
