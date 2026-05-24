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
    /// Updates the weight of a hyperedge by index.
    ///
    /// The new weight need not be unique — multiple hyperedges may carry the
    /// same weight value. Returns [`HypergraphError::HyperedgeWeightUnchanged`]
    /// if `weight` equals the current weight (no-op guard).
    pub fn update_hyperedge_weight(
        &mut self,
        hyperedge_index: HyperedgeIndex,
        weight: HE,
    ) -> Result<(), HypergraphError<V, HE>> {
        let (_, current_weight) = self
            .hyperedges
            .get(&hyperedge_index)
            .ok_or(HypergraphError::HyperedgeIndexNotFound(hyperedge_index))?;

        if weight == *current_weight {
            return Err(HypergraphError::HyperedgeWeightUnchanged {
                index: hyperedge_index,
                weight,
            });
        }

        if let Some((_, w)) = self.hyperedges.get_mut(&hyperedge_index) {
            *w = weight;
        }

        Ok(())
    }
}
