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
    /// Removes the first occurrence of `vertex_index` from `hyperedge_index`'s
    /// vertex list.
    ///
    /// If `vertex_index` no longer appears anywhere in the vertex list after
    /// removal, `hyperedge_index` is removed from that vertex's back-reference
    /// set.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist, [`HypergraphError::VertexIndexNotFound`] if `vertex_index`
    /// does not exist, or
    /// [`HypergraphError::HyperedgeVerticesIndexesNotFound`] if `vertex_index` is
    /// not in the hyperedge's vertex list.
    pub fn delete_vertex_from_hyperedge(
        &mut self,
        hyperedge_index: HyperedgeIndex,
        vertex_index: VertexIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        if !self.hyperedges.contains_key(&hyperedge_index) {
            return Err(HypergraphError::HyperedgeIndexNotFound(hyperedge_index));
        }
        if !self.vertices.contains_key(&vertex_index) {
            return Err(HypergraphError::VertexIndexNotFound(vertex_index));
        }

        let (verts, _) = self.hyperedges.get(&hyperedge_index).unwrap();
        let pos = verts.iter().position(|&v| v == vertex_index).ok_or(
            HypergraphError::HyperedgeVerticesIndexesNotFound {
                index: hyperedge_index,
                vertices: vec![vertex_index],
            },
        )?;

        self.hyperedges
            .get_mut(&hyperedge_index)
            .unwrap()
            .0
            .remove(pos);

        // Check if vertex still appears in the list.
        let still_present = self
            .hyperedges
            .get(&hyperedge_index)
            .unwrap()
            .0
            .contains(&vertex_index);

        if !still_present && let Some((_, he_set)) = self.vertices.get_mut(&vertex_index) {
            he_set.swap_remove(&hyperedge_index);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        HyperedgeIndex,
        VertexIndex,
        core::test_support::{
            E,
            build,
        },
    };

    #[test]
    fn delete_vertex_basic() {
        let (mut g, [v0, v1, v2, v3], [e0, _, _]) = build();
        // e0=[v0,v1], delete v0
        g.delete_vertex_from_hyperedge(e0, v0).unwrap();
        let verts = g.get_hyperedge_vertices(e0).unwrap();
        assert_eq!(verts, vec![v1]);
        // v0 should no longer have e0 as a back-ref
        assert!(!g.get_vertex_hyperedges(v0).unwrap().contains(&e0));
        let _ = (v2, v3);
    }

    #[test]
    fn delete_vertex_still_present_keeps_backref() {
        let (mut g, [v0, v1, v2, v3], _) = build();
        // Create an edge with v0 twice
        let e_new = g.add_hyperedge(vec![v0, v0, v1], E(99)).unwrap();
        g.delete_vertex_from_hyperedge(e_new, v0).unwrap();
        // v0 still in the list once, so back-ref should remain
        assert!(g.get_vertex_hyperedges(v0).unwrap().contains(&e_new));
        let _ = (v2, v3);
    }

    #[test]
    fn delete_vertex_not_in_hyperedge() {
        let (mut g, [_v0, _v1, v2, _v3], [e0, _, _]) = build();
        // v2 is not in e0=[v0,v1]
        assert!(g.delete_vertex_from_hyperedge(e0, v2).is_err());
    }

    #[test]
    fn delete_vertex_bad_hyperedge() {
        let (mut g, [v0, _, _, _], _) = build();
        assert!(
            g.delete_vertex_from_hyperedge(HyperedgeIndex(999), v0)
                .is_err()
        );
    }

    #[test]
    fn delete_vertex_bad_vertex() {
        let (mut g, _, [e0, _, _]) = build();
        assert!(
            g.delete_vertex_from_hyperedge(e0, VertexIndex(999))
                .is_err()
        );
    }
}
