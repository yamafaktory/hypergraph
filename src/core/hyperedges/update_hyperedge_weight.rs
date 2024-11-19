use crate::{
    HyperedgeIndex,
    HyperedgeKey,
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
    pub fn update_hyperedge_weight(
        &mut self,
        hyperedge_index: HyperedgeIndex,
        weight: HE,
    ) -> Result<(), HypergraphError<V, HE>> {
        let internal_index = self.get_internal_hyperedge(hyperedge_index)?;

        let HyperedgeKey {
            vertices,
            weight: previous_weight,
        } = self.hyperedges.get_index(internal_index).ok_or(
            HypergraphError::InternalHyperedgeIndexNotFound(internal_index),
        )?;

        // Return an error if the new weight is the same as the previous one.
        if weight == *previous_weight {
            return Err(HypergraphError::HyperedgeWeightUnchanged {
                index: hyperedge_index,
                weight,
            });
        }

        // Return an error if the new weight is already assigned to another
        // hyperedge.
        // We can't use the contains method here since the key is a combination
        // of the weight and the vertices.
        if self.hyperedges.iter().any(
            |HyperedgeKey {
                 weight: current_weight,
                 ..
             }| { *current_weight == weight },
        ) {
            return Err(HypergraphError::HyperedgeWeightAlreadyAssigned(weight));
        }

        // IndexMap doesn't allow holes by design, see:
        // https://github.com/bluss/indexmap/issues/90#issuecomment-455381877
        //
        // As a consequence, we have two options. Either we use shift_remove
        // and it will result in an expensive regeneration of all the indexes
        // in the map/set or we use swap_remove methods and deal with the fact
        // that the last element will be swapped in place of the removed one
        // and will thus get a new index.
        //
        // In our case, since we are inserting an entry upfront, it circumvents
        // the aforementioned issue.
        //
        // First case: index alteration is avoided.
        //
        // Entry to remove
        //  |              1.Insert new entry
        //  |                     |
        //  v                     v
        // [a, b, c] -> [a, b, c, d] -> [d, b, c, _]
        //                               ^        ^
        //                               |        |
        //                               +--------+
        //                         2.Swap and remove
        //
        // -----------------------------------------
        //
        // Second case: no index alteration.
        //
        // Entry to remove
        //        |        1.Insert new entry
        //        |               |
        //        v               v
        // [a, b, c] -> [a, b, c, d] -> [a, b, d, _]
        //                                     ^  ^
        //                                     |  |
        //                                     +--+
        //                         2.Swap and remove
        //
        // Insert the new entry.
        // Since we have already checked that the new weight is not in the
        // map, we can safely perform the operation without checking its output.
        self.hyperedges
            .insert(HyperedgeKey::new(vertices.clone(), weight));

        // Swap and remove by index.
        // Since we know that the internal index is correct, we can safely
        // perform the operation without checking its output.
        self.hyperedges.swap_remove_index(internal_index);

        // Return a unit.
        Ok(())
    }
}
