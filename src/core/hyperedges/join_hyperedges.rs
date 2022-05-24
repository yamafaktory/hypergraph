use rayon::prelude::*;

use crate::{errors::HypergraphError, HyperedgeIndex, HyperedgeTrait, Hypergraph, VertexTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Joins two or more hyperedges from the hypergraph into one single entity.
    pub fn join_hyperedges(
        &mut self,
        hyperedges: Vec<HyperedgeIndex>,
    ) -> Result<(), HypergraphError<V, HE>> {
        // If the provided hyperedges are empty, skip the operation.
        if hyperedges.len() < 2 {
            // return Err(HypergraphError::HyperedgeCreationNoVertices(weight));
        }

        let joined_vertices = hyperedges
            .par_iter()
            .flat_map_iter(|hyperedge_index| self.get_hyperedge_vertices(*hyperedge_index));

        // The goal is to move all the vertices from the provided hyperedges to
        // the first hyperedge.
        self.update_hyperedge_vertices(hyperedges[0]);

        Ok(())
    }
}
