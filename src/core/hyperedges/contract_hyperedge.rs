use crate::{errors::HypergraphError, HyperedgeIndex, Hypergraph, SharedTrait, VertexIndex};

use itertools::Itertools;

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Contracts the vertices of a hyperedge into one single vertex.
    /// Based on <https://en.wikipedia.org/wiki/Edge_contraction>
    pub fn contract_hyperedge_vertices(
        &mut self,
        hyperedge_index: HyperedgeIndex,
        vertices: Vec<VertexIndex>,
        target: VertexIndex,
    ) -> Result<HE, HypergraphError<V, HE>> {
        // Get all the vertices of the hyperedge.
        let hyperedge_vertices = self.get_hyperedge_vertices(hyperedge_index)?;

        // Get the deduped vertices.
        let deduped_vertices = vertices.into_iter().sorted().dedup().collect_vec();

        // Check that the target is included in the deduped vertices.
        if !deduped_vertices
            .iter()
            .any(|&current_index| current_index == target)
        {
            return Err(HypergraphError::HyperedgeInvalidContraction {
                index: hyperedge_index,
                target,
                vertices: deduped_vertices,
            });
        }

        // Get the vertices not found in the hyperedge.
        let vertices_not_found =
            deduped_vertices
                .iter()
                .fold(vec![], |mut acc: Vec<VertexIndex>, index| {
                    if !hyperedge_vertices
                        .iter()
                        .any(|current_index| current_index == index)
                    {
                        acc.push(index.to_owned())
                    }

                    acc
                });

        // Check that all the vertices - target included - are a subset of
        // the hyperedge vertices.
        if !vertices_not_found.is_empty() {
            return Err(HypergraphError::HyperedgeVerticesIndexesNotFound {
                index: hyperedge_index,
                vertices: vertices_not_found,
            });
        }

        // todo
        todo!()
    }
}
