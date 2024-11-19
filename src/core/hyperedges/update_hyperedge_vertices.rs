use rayon::prelude::*;

use crate::{
    HyperedgeIndex,
    HyperedgeKey,
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    core::utils::are_slices_equal,
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Updates the vertices of a hyperedge by index.
    pub fn update_hyperedge_vertices(
        &mut self,
        hyperedge_index: HyperedgeIndex,
        vertices: Vec<VertexIndex>,
    ) -> Result<(), HypergraphError<V, HE>> {
        // If the provided vertices are empty, skip the update.
        if vertices.is_empty() {
            return Err(HypergraphError::HyperedgeUpdateNoVertices(hyperedge_index));
        }

        let internal_index = self.get_internal_hyperedge(hyperedge_index)?;

        let internal_vertices = self.get_internal_vertices(vertices)?;

        let HyperedgeKey {
            vertices: previous_vertices,
            weight,
        } = self.hyperedges.get_index(internal_index).cloned().ok_or(
            HypergraphError::InternalHyperedgeIndexNotFound(internal_index),
        )?;

        // If the new vertices are the same as the old ones, skip the update.
        if are_slices_equal(&internal_vertices, &previous_vertices) {
            return Err(HypergraphError::HyperedgeVerticesUnchanged(hyperedge_index));
        }

        // Find the vertices which have been added.
        let mut added = internal_vertices
            .par_iter()
            .fold_with(vec![], |mut acc: Vec<usize>, index| {
                if !previous_vertices
                    .par_iter()
                    .any(|current_index| current_index == index)
                {
                    acc.push(*index);
                }

                acc
            })
            .flatten()
            .collect::<Vec<usize>>();

        added.par_sort_unstable();
        added.dedup();

        // Find the vertices which have been removed.
        let mut removed = previous_vertices
            .into_par_iter()
            .filter_map(|index| {
                if internal_vertices
                    .par_iter()
                    .any(|current_index| index == *current_index)
                {
                    None
                } else {
                    Some(index)
                }
            })
            .collect::<Vec<usize>>();

        removed.par_sort_unstable();
        removed.dedup();

        // Update the added vertices.
        for index in added {
            match self.vertices.get_index_mut(index) {
                Some((_, index_set)) => {
                    index_set.insert(internal_index);
                }
                None => return Err(HypergraphError::InternalVertexIndexNotFound(index)),
            }
        }

        // Update the removed vertices.
        for index in removed {
            match self.vertices.get_index_mut(index) {
                Some((_, index_set)) => {
                    // This has an impact on the internal indexing for the set.
                    // However since this is not exposed to the user - i.e. no
                    // mapping is involved - we can safely perform the operation.
                    index_set.swap_remove_index(internal_index);
                }
                None => return Err(HypergraphError::InternalVertexIndexNotFound(index)),
            }
        }

        // Insert the new entry.
        // Since we are not altering the weight, we can safely perform the
        // operation without checking its output.
        self.hyperedges.insert(HyperedgeKey {
            vertices: internal_vertices,
            weight,
        });

        // Swap and remove by index.
        // Since we know that the internal index is correct, we can safely
        // perform the operation without checking its output.
        self.hyperedges.swap_remove_index(internal_index);

        // Return a unit.
        Ok(())
    }
}
