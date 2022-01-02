use crate::{
    bi_hash_map::BiHashMap, errors::HypergraphError, HyperedgeTrait, Hypergraph, VertexTrait,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Clears all the hyperedges from the hypergraph.
    pub fn clear_hyperedges(&mut self) -> Result<(), HypergraphError<V, HE>> {
        // Clear the set while keeping its capacity.
        self.hyperedges.clear();

        // Reset the hyperedges mapping.
        self.hyperedges_mapping = BiHashMap::default();

        // Reset the hyperedges counter.
        self.hyperedges_count = 0;

        // Update the vertices accordingly.
        self.vertices
            .iter_mut()
            // Clear the sets while keeping their capacities.
            .for_each(|(_, hyperedges)| hyperedges.clear());

        Ok(())
    }
}
