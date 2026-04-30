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
    // Private method to get the HyperedgeIndex matching an internal index.
    pub(crate) fn get_hyperedge(
        &self,
        hyperedge_index: usize,
    ) -> Result<HyperedgeIndex, HypergraphError<V, HE>> {
        match self.hyperedges_mapping.left.get(&hyperedge_index) {
            Some(index) => Ok(*index),
            None => Err(HypergraphError::InternalHyperedgeIndexNotFound(
                hyperedge_index,
            )),
        }
    }
}
