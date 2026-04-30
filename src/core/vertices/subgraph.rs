use ahash::{AHashMap, AHashSet};

use crate::{HyperedgeIndex, HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait, errors::HypergraphError};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns the induced subgraph for the given vertex set.
    ///
    /// The subgraph contains exactly the specified vertices (deduplicated and
    /// validated) and every hyperedge from the original graph whose vertices are
    /// all within that set. Vertex stable indexes are reassigned from `0` in
    /// sorted order; hyperedge stable indexes are similarly reassigned in
    /// original insertion order.
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] for any invalid index.
    pub fn subgraph(
        &self,
        vertices: &[VertexIndex],
    ) -> Result<Hypergraph<V, HE>, HypergraphError<V, HE>> {
        // Deduplicate and sort for a deterministic, stable output order.
        let mut sorted: Vec<VertexIndex> = vertices.to_vec();
        sorted.sort();
        sorted.dedup();

        let vertex_set: AHashSet<VertexIndex> = sorted.iter().copied().collect();

        let mut sub = Hypergraph::new();
        let mut old_to_new: AHashMap<VertexIndex, VertexIndex> = AHashMap::new();

        for &old_idx in &sorted {
            let weight = self.get_vertex_weight(old_idx)?;
            let new_idx = sub.add_vertex(*weight)?;
            old_to_new.insert(old_idx, new_idx);
        }

        // Include hyperedges whose entire vertex set falls within the subgraph.
        let mut hyperedges: Vec<(HyperedgeIndex, &HE)> = self.hyperedges_iter().collect();
        hyperedges.sort_by_key(|(idx, _)| *idx);

        for (he_idx, _) in hyperedges {
            let he_vertices = self.get_hyperedge_vertices(he_idx)?;
            if he_vertices.iter().all(|v| vertex_set.contains(v)) {
                let new_vertices: Vec<VertexIndex> =
                    he_vertices.iter().map(|v| old_to_new[v]).collect();
                let he_weight = self.get_hyperedge_weight(he_idx)?;
                sub.add_hyperedge(new_vertices, *he_weight)?;
            }
        }

        Ok(sub)
    }
}
