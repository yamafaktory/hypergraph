use crate::{
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Removes a vertex by index.
    ///
    /// All hyperedges that contain only this vertex are removed. Hyperedges
    /// that contain other vertices are updated with the vertex filtered out.
    pub fn remove_vertex(
        &mut self,
        vertex_index: VertexIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        // Collect the hyperedge indices upfront before any mutation.
        let he_indices = self.get_vertex_hyperedges(vertex_index)?;

        for he_index in he_indices {
            let vertices = self
                .hyperedges
                .get(&he_index)
                .map(|(v, _)| v.clone())
                .ok_or(HypergraphError::HyperedgeIndexNotFound(he_index))?;

            // Determine if this vertex is the sole unique vertex in the hyperedge.
            let mut unique_verts = vertices.clone();
            unique_verts.sort_unstable();
            unique_verts.dedup();

            if unique_verts.len() == 1 {
                self.remove_hyperedge(he_index)?;
            } else {
                let updated: Vec<VertexIndex> = vertices
                    .into_iter()
                    .filter(|&v| v != vertex_index)
                    .collect();
                self.update_hyperedge_vertices(he_index, updated)?;
            }
        }

        self.vertices
            .swap_remove(&vertex_index)
            .ok_or(HypergraphError::VertexIndexNotFound(vertex_index))?;

        Ok(())
    }
}
