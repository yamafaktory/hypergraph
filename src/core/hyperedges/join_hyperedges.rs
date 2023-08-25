use rayon::prelude::*;

use crate::{
    errors::HypergraphError, HyperedgeIndex, HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Joins two or more hyperedges from the hypergraph into one single entity.
    /// All the vertices are moved to the first hyperedge in the provided order.
    pub fn join_hyperedges(
        &mut self,
        hyperedges: &[HyperedgeIndex],
    ) -> Result<(), HypergraphError<V, HE>> {
        // If the provided hyperedges are less than two, skip the operation.
        if hyperedges.len() < 2 {
            return Err(HypergraphError::HyperedgesInvalidJoin);
        }

        // Try to collect all the vertices from the provided hyperedges.
        match hyperedges
            .par_iter()
            .map(|hyperedge_index| self.get_hyperedge_vertices(*hyperedge_index))
            .collect::<Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>>>()
        {
            Err(err) => Err(err),
            Ok(joined_vertices) => {
                // The goal is to move all the vertices from the provided
                // hyperedges to the first one.
                self.update_hyperedge_vertices(
                    hyperedges[0],
                    joined_vertices.into_par_iter().flatten().collect(),
                )?;

                // Get the tail.
                let tail = &hyperedges[1..];

                // Removes the other hyperedges.
                for hyperedge_index in tail {
                    self.remove_hyperedge(*hyperedge_index)?;
                }

                Ok(())
            }
        }
    }
}
