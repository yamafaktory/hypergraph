use crate::{HyperedgeIndex, HyperedgeTrait, Hypergraph, VertexTrait, errors::HypergraphError};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Gets the weight of a hyperedge from its index.
    pub fn get_hyperedge_weight(
        &self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<&HE, HypergraphError<V, HE>> {
        let internal_index = self.get_internal_hyperedge(hyperedge_index)?;

        let hyperedge_key = self
            .hyperedges
            .get_index(internal_index)
            .ok_or(HypergraphError::InternalVertexIndexNotFound(internal_index))?;

        Ok(&**hyperedge_key)
    }
}
