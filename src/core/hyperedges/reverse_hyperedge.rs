use crate::{errors::HypergraphError, HyperedgeIndex, Hypergraph, SharedTrait};

use itertools::Itertools;

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    // Reverses a hyperedge.
    pub fn reverse_hyperedge(
        &mut self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        // Get the vertices of the hyperedge.
        let vertices = self.get_hyperedge_vertices(hyperedge_index)?;

        // Update the hyperedge with the reversed vertices.
        self.update_hyperedge_vertices(hyperedge_index, vertices.into_iter().rev().collect_vec())
    }
}
