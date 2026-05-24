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
    /// Returns a reference to the weight of the hyperedge at `hyperedge_index`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist.
    pub fn get_hyperedge_weight(
        &self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<&HE, HypergraphError<V, HE>> {
        self.hyperedges
            .get(&hyperedge_index)
            .map(|(_, weight)| weight)
            .ok_or(HypergraphError::HyperedgeIndexNotFound(hyperedge_index))
    }
}
