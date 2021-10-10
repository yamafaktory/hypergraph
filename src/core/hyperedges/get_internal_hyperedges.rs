use crate::{errors::HypergraphError, HyperedgeIndex, Hypergraph, SharedTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    // Private method to get the internal hyperedges from a vector of HyperedgeIndex.
    pub(crate) fn get_internal_hyperedges(
        &self,
        hyperedges: Vec<HyperedgeIndex>,
    ) -> Result<Vec<usize>, HypergraphError<V, HE>> {
        hyperedges
            .iter()
            .map(|hyperedge_index| self.get_internal_hyperedge(*hyperedge_index))
            .collect()
    }
}
