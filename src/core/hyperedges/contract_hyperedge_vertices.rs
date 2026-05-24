use itertools::Itertools;

use crate::{
    HyperedgeIndex,
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
    /// Contracts `vertices` into `target` within `hyperedge_index`.
    ///
    /// Every occurrence of any vertex in `vertices` across all hyperedges that
    /// touch those vertices is replaced by `target`. Consecutive duplicates
    /// introduced by the substitution are collapsed. Returns the updated vertex
    /// list of `hyperedge_index` after the contraction.
    ///
    /// See <https://en.wikipedia.org/wiki/Edge_contraction> for the definition.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist, [`HypergraphError::HyperedgeInvalidContraction`] if
    /// `target` is not in `vertices`, or
    /// [`HypergraphError::HyperedgeVerticesIndexesNotFound`] if any of
    /// `vertices` are not part of the hyperedge.
    pub fn contract_hyperedge_vertices(
        &mut self,
        hyperedge_index: HyperedgeIndex,
        vertices: Vec<VertexIndex>,
        target: VertexIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        let hyperedge_vertices = self.get_hyperedge_vertices(hyperedge_index)?;

        let mut deduped_vertices = vertices;
        deduped_vertices.sort_unstable();
        deduped_vertices.dedup();

        if !deduped_vertices.contains(&target) {
            return Err(HypergraphError::HyperedgeInvalidContraction {
                index: hyperedge_index,
                target,
                vertices: deduped_vertices,
            });
        }

        let vertices_not_found = deduped_vertices
            .iter()
            .filter(|index| !hyperedge_vertices.contains(index))
            .copied()
            .collect::<Vec<VertexIndex>>();

        if !vertices_not_found.is_empty() {
            return Err(HypergraphError::HyperedgeVerticesIndexesNotFound {
                index: hyperedge_index,
                vertices: vertices_not_found,
            });
        }

        // Collect all hyperedges touching any of the contracting vertices.
        let mut all_hyperedges: Vec<HyperedgeIndex> = Vec::new();
        for &vertex in &deduped_vertices {
            let mut vertex_hyperedges = self.get_vertex_hyperedges(vertex)?;
            all_hyperedges.append(&mut vertex_hyperedges);
        }

        for &hyperedge in all_hyperedges.iter().sorted().dedup() {
            let he_vertices = self.get_hyperedge_vertices(hyperedge)?;

            let contraction = he_vertices
                .iter()
                .map(|vertex| {
                    if deduped_vertices.contains(vertex) {
                        target
                    } else {
                        *vertex
                    }
                })
                .dedup()
                .collect::<Vec<_>>();

            if contraction != he_vertices {
                self.update_hyperedge_vertices(hyperedge, contraction)?;
            }
        }

        self.get_hyperedge_vertices(hyperedge_index)
    }
}
