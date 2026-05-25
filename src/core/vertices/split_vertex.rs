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
    /// Creates a new vertex with `weight` and replaces all occurrences of
    /// `vertex_index` with the new vertex in each of the specified `hyperedges`.
    ///
    /// If a specified hyperedge does not contain `vertex_index`, it is skipped.
    ///
    /// Returns the [`VertexIndex`] of the newly created vertex.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgesInvalidSplit`] if `hyperedges` is
    /// empty, [`HypergraphError::VertexIndexNotFound`] if `vertex_index` does
    /// not exist, or [`HypergraphError::HyperedgeIndexNotFound`] if any index
    /// in `hyperedges` does not exist.
    pub fn split_vertex(
        &mut self,
        vertex_index: VertexIndex,
        hyperedges: &[HyperedgeIndex],
        weight: V,
    ) -> Result<VertexIndex, HypergraphError<V, HE>> {
        if hyperedges.is_empty() {
            return Err(HypergraphError::HyperedgesInvalidSplit);
        }

        // Validate vertex exists.
        if !self.vertices.contains_key(&vertex_index) {
            return Err(HypergraphError::VertexIndexNotFound(vertex_index));
        }

        // Validate all specified hyperedges.
        for &hei in hyperedges {
            if !self.hyperedges.contains_key(&hei) {
                return Err(HypergraphError::HyperedgeIndexNotFound(hei));
            }
        }

        // Create the new vertex.
        let new_v = self.add_vertex(weight)?;

        for &hei in hyperedges {
            let verts = self.hyperedges.get(&hei).unwrap().0.clone();
            if !verts.contains(&vertex_index) {
                continue;
            }

            // Replace all occurrences of vertex_index with new_v.
            let new_verts: Vec<VertexIndex> = verts
                .iter()
                .map(|&v| if v == vertex_index { new_v } else { v })
                .collect();

            // Update back-refs: add new_v's back-ref to hei.
            if let Some((_, he_set)) = self.vertices.get_mut(&new_v) {
                he_set.insert(hei);
            }

            // Remove old vertex back-ref if it no longer appears.
            let still_has_old = new_verts.contains(&vertex_index);
            if !still_has_old && let Some((_, he_set)) = self.vertices.get_mut(&vertex_index) {
                he_set.swap_remove(&hei);
            }

            self.hyperedges.get_mut(&hei).unwrap().0 = new_verts;
        }

        Ok(new_v)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        HyperedgeIndex,
        VertexIndex,
        core::test_support::{
            W,
            build,
        },
    };

    #[test]
    fn split_vertex_basic() {
        let (mut g, [_v0, v1, _v2, _v3], [e0, e1, _e2]) = build();
        // Split v1 into a new vertex for e0 only
        let new_v = g.split_vertex(v1, &[e0], W(99)).unwrap();
        // e0 should now have new_v instead of v1
        let e0_verts = g.get_hyperedge_vertices(e0).unwrap();
        assert!(e0_verts.contains(&new_v));
        assert!(!e0_verts.contains(&v1));
        // e1 should still have v1
        let e1_verts = g.get_hyperedge_vertices(e1).unwrap();
        assert!(e1_verts.contains(&v1));
        // v1 should still be in e1 back-refs, not in e0
        assert!(!g.get_vertex_hyperedges(v1).unwrap().contains(&e0));
        assert!(g.get_vertex_hyperedges(v1).unwrap().contains(&e1));
        // new_v should be in e0 back-refs
        assert!(g.get_vertex_hyperedges(new_v).unwrap().contains(&e0));
    }

    #[test]
    fn split_vertex_empty_hyperedges_error() {
        let (mut g, [v0, _, _, _], _) = build();
        assert!(g.split_vertex(v0, &[], W(99)).is_err());
    }

    #[test]
    fn split_vertex_bad_vertex() {
        let (mut g, _, [e0, _, _]) = build();
        assert!(g.split_vertex(VertexIndex(999), &[e0], W(99)).is_err());
    }

    #[test]
    fn split_vertex_bad_hyperedge() {
        let (mut g, [v0, _, _, _], _) = build();
        assert!(g.split_vertex(v0, &[HyperedgeIndex(999)], W(99)).is_err());
    }

    #[test]
    fn split_vertex_skips_non_containing_edge() {
        let (mut g, [v0, _v1, _v2, _v3], [_e0, e1, e2]) = build();
        // v0 is not in e1 or e2, so split should still succeed but do nothing.
        let new_v = g.split_vertex(v0, &[e1, e2], W(99)).unwrap();
        // e1 and e2 should be unchanged
        assert!(!g.get_hyperedge_vertices(e1).unwrap().contains(&new_v));
        assert!(!g.get_hyperedge_vertices(e2).unwrap().contains(&new_v));
    }
}
