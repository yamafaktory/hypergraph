use crate::{
    core::utils::are_slices_equal, errors::HypergraphError, HyperedgeIndex, HyperedgeTrait,
    Hypergraph, VertexIndex, VertexTrait,
};

use itertools::Itertools;

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Contracts a set of the vertices of a hyperedge into one single vertex.
    /// Returns the updated vertices.
    /// Based on <https://en.wikipedia.org/wiki/Edge_contraction>
    pub fn contract_hyperedge_vertices(
        &mut self,
        hyperedge_index: HyperedgeIndex,
        vertices: Vec<VertexIndex>,
        target: VertexIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        // Get all the vertices of the hyperedge.
        let hyperedge_vertices = self.get_hyperedge_vertices(hyperedge_index)?;

        // Get the deduped vertices.
        let deduped_vertices = vertices.iter().sorted().dedup().collect_vec();

        // Check that the target is included in the deduped vertices.
        if !deduped_vertices
            .iter()
            .any(|&current_index| current_index == &target)
        {
            return Err(HypergraphError::HyperedgeInvalidContraction {
                index: hyperedge_index,
                target,
                vertices: deduped_vertices.into_iter().cloned().collect(),
            });
        }

        // Get the vertices not found in the hyperedge.
        let vertices_not_found =
            deduped_vertices
                .iter()
                .fold(vec![], |mut acc: Vec<VertexIndex>, &index| {
                    if !hyperedge_vertices
                        .iter()
                        .any(|&current_index| current_index == *index)
                    {
                        acc.push(index.to_owned())
                    }

                    acc
                });

        // Check that all the vertices - target included - are a subset of
        // the current hyperedge's vertices.
        if !vertices_not_found.is_empty() {
            return Err(HypergraphError::HyperedgeVerticesIndexesNotFound {
                index: hyperedge_index,
                vertices: vertices_not_found,
            });
        }

        // Store all the hyperedges which are going to change.
        let mut all_hyperedges: Vec<HyperedgeIndex> = vec![];

        // Iterate over all the deduped vertices.
        for &vertex in deduped_vertices.iter() {
            // Safely get the hyperedges of the current vertex.
            let mut vertex_hyperedges = self.get_vertex_hyperedges(*vertex)?;

            // Concatenate them to the global ones.
            all_hyperedges.append(&mut vertex_hyperedges);
        }

        // Iterate over all the deduped hyperedges.
        for &hyperedge in all_hyperedges.iter().sorted().dedup() {
            let hyperedge_vertices = self.get_hyperedge_vertices(hyperedge)?;

            // Contract the vertices of the hyperedge.
            let contraction = hyperedge_vertices
                .iter()
                // First remap each vertex to itself or to the target.
                .map(|vertex| {
                    if deduped_vertices
                        .iter()
                        .any(|&current_index| current_index == vertex)
                    {
                        target
                    } else {
                        *vertex
                    }
                })
                // Then dedupe the resulting vector.
                .dedup()
                .collect_vec();

            // Only update the hyperedge if necessary.
            if !are_slices_equal(
                &self.get_internal_vertices(&contraction)?,
                &self.get_internal_vertices(hyperedge_vertices)?,
            ) {
                // Safely update the current hyperedge with the contraction.
                self.update_hyperedge_vertices(hyperedge, contraction)?;
            }
        }

        // Return the contraction.
        self.get_hyperedge_vertices(hyperedge_index)
    }
}
