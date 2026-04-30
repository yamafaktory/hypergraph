use itertools::Itertools;
use rayon::prelude::*;

use crate::{
    HyperedgeIndex, HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait,
    core::utils::are_slices_equal, errors::HypergraphError,
};

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
        // We use `par_sort_unstable` here which means that the order of equal
        // elements is not preserved but this is fine since we dedupe them
        // afterwards.
        let mut deduped_vertices = vertices;

        deduped_vertices.par_sort_unstable();
        deduped_vertices.dedup();

        // Check that the target is included in the deduped vertices.
        if !deduped_vertices
            .par_iter()
            .any(|&current_index| current_index == target)
        {
            return Err(HypergraphError::HyperedgeInvalidContraction {
                index: hyperedge_index,
                target,
                vertices: deduped_vertices,
            });
        }

        // Get the vertices not found in the hyperedge.
        let vertices_not_found = deduped_vertices
            .par_iter()
            .fold_with(vec![], |mut acc: Vec<VertexIndex>, &index| {
                if !hyperedge_vertices
                    .par_iter()
                    .any(|&current_index| current_index == index)
                {
                    acc.push(index);
                }

                acc
            })
            .flatten()
            .collect::<Vec<VertexIndex>>();

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
        for &vertex in &deduped_vertices {
            // Safely get the hyperedges of the current vertex.
            let mut vertex_hyperedges = self.get_vertex_hyperedges(vertex)?;

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
                        .par_iter()
                        .any(|&current_index| current_index == *vertex)
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
