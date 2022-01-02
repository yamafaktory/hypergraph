use crate::{
    errors::HypergraphError, HyperedgeKey, HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait,
};

use itertools::Itertools;

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Removes a vertex by index.
    pub fn remove_vertex(
        &mut self,
        vertex_index: VertexIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        let internal_index = self.get_internal_vertex(vertex_index)?;

        // Get the hyperedges of the vertex.
        let hyperedges = self.get_internal_hyperedges(self.get_vertex_hyperedges(vertex_index)?)?;

        // Remove the vertex from the hyperedges which contain it.
        for hyperedge in hyperedges.into_iter() {
            let HyperedgeKey { vertices, .. } = self
                .hyperedges
                .get_index(hyperedge)
                .map(|hyperedge_key| hyperedge_key.to_owned())
                .ok_or(HypergraphError::InternalHyperedgeIndexNotFound(hyperedge))?;

            let hyperedge_index = self.get_hyperedge(hyperedge)?;

            // Get the unique vertices, i.e. check for self-loops.
            let unique_vertices = vertices.iter().sorted().dedup().collect_vec();

            // Remove the hyperedge if the vertex is the only one present.
            if unique_vertices.len() == 1 {
                self.remove_hyperedge(hyperedge_index)?;
            } else {
                // Otherwise update the hyperedge with the updated vertices.
                let updated_vertices = self.get_vertices(
                    vertices
                        .into_iter()
                        .filter(|vertex| *vertex != internal_index)
                        .collect_vec(),
                )?;

                self.update_hyperedge_vertices(hyperedge_index, updated_vertices)?;
            }
        }

        // Find the last index.
        let last_index = self.vertices.len() - 1;

        // Swap and remove by index.
        self.vertices.swap_remove_index(internal_index);

        // Update the mapping for the removed vertex.
        self.vertices_mapping.left.remove(&internal_index);
        self.vertices_mapping.right.remove(&vertex_index);

        // If the index to remove wasn't the last one, the last vertex has
        // been swapped in place of the removed one. See the remove_hyperedge
        // method for more details about the internals.
        if internal_index != last_index {
            // Get the index of the swapped vertex.
            let swapped_vertex_index = self.get_vertex(last_index)?;

            // Proceed with the aforementioned operations.
            self.vertices_mapping
                .right
                .insert(swapped_vertex_index, internal_index);
            self.vertices_mapping.left.remove(&last_index);
            self.vertices_mapping
                .left
                .insert(internal_index, swapped_vertex_index);

            let stale_hyperedges =
                self.get_internal_hyperedges(self.get_vertex_hyperedges(swapped_vertex_index)?)?;

            // Update the impacted hyperedges accordingly.
            for hyperedge in stale_hyperedges.into_iter() {
                let HyperedgeKey { vertices, weight } = self
                    .hyperedges
                    .get_index(hyperedge)
                    .map(|hyperedge_key| hyperedge_key.to_owned())
                    .ok_or(HypergraphError::InternalHyperedgeIndexNotFound(hyperedge))?;

                let updated_vertices = vertices
                    .into_iter()
                    .map(|vertex| {
                        // Remap the vertex if this is the swapped one.
                        if vertex == last_index {
                            internal_index
                        } else {
                            vertex
                        }
                    })
                    .collect_vec();

                // Insert the new entry with the updated vertices.
                // Since we are not altering the weight, we can safely perform
                // the operation without checking its output.
                self.hyperedges.insert(HyperedgeKey {
                    vertices: updated_vertices,
                    weight,
                });

                // Swap and remove by index.
                // Since we know that the hyperedge index is correct, we can
                // safely perform the operation without checking its output.
                self.hyperedges.swap_remove_index(hyperedge);
            }
        }

        // Return a unit.
        Ok(())
    }
}
