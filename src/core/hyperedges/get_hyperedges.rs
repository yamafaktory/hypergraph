use rayon::prelude::*;

use crate::{errors::HypergraphError, HyperedgeIndex, HyperedgeTrait, Hypergraph, VertexTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    // Private method to get a vector of HyperedgeIndex from a vector of internal indexes.
    pub(crate) fn get_hyperedges(
        &self,
        hyperedges: &[usize],
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        hyperedges
            .par_iter()
            .map(|hyperedge_index| self.get_hyperedge(*hyperedge_index))
            .collect()
    }
}
