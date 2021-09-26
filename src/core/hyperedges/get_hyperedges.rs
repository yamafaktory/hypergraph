use crate::{errors::HypergraphError, HyperedgeIndex, Hypergraph, SharedTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    // Private method to get a vector of HyperedgeIndex from a vector of internal indexes.
    pub(crate) fn get_hyperedges(
        &self,
        hyperedges: Vec<usize>,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        hyperedges
            .iter()
            .map(|hyperedge_index| self.get_hyperedge(*hyperedge_index))
            .collect()
    }
}
