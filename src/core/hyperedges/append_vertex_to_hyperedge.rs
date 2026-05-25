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
    /// Appends `vertex_index` to the end of `hyperedge_index`'s vertex list.
    ///
    /// Back-references are updated: if `vertex_index` was not already in the
    /// hyperedge's vertex list, `hyperedge_index` is added to that vertex's
    /// back-reference set.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist, or [`HypergraphError::VertexIndexNotFound`] if
    /// `vertex_index` does not exist.
    pub fn append_vertex_to_hyperedge(
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
        let already_present = verts.contains(&vertex_index);

        self.hyperedges
            .get_mut(&hyperedge_index)
            .unwrap()
            .0
            .push(vertex_index);

        if !already_present && let Some((_, he_set)) = self.vertices.get_mut(&vertex_index) {
            he_set.insert(hyperedge_index);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        HyperedgeIndex,
        VertexIndex,
        core::test_support::build,
    };

    #[test]
    fn append_vertex_basic() {
        let (mut g, [v0, v1, v2, v3], [e0, _, _]) = build();
        // e0=[v0,v1], append v2
        g.append_vertex_to_hyperedge(e0, v2).unwrap();
        assert_eq!(g.get_hyperedge_vertices(e0).unwrap(), vec![v0, v1, v2]);
        // v2's back-ref now includes e0
        assert!(g.get_vertex_hyperedges(v2).unwrap().contains(&e0));
        let _ = v3;
    }

    #[test]
    fn append_existing_vertex_no_duplicate_backref() {
        let (mut g, [v0, v1, _v2, _v3], [e0, _, _]) = build();
        // v1 already in e0
        g.append_vertex_to_hyperedge(e0, v1).unwrap();
        let hyperedges_of_v1 = g.get_vertex_hyperedges(v1).unwrap();
        // e0 should appear only once in v1's back-refs
        assert_eq!(hyperedges_of_v1.iter().filter(|&&h| h == e0).count(), 1);
        let _ = v0;
    }

    #[test]
    fn append_vertex_bad_hyperedge() {
        let (mut g, [v0, _, _, _], _) = build();
        assert!(
            g.append_vertex_to_hyperedge(HyperedgeIndex(999), v0)
                .is_err()
        );
    }

    #[test]
    fn append_vertex_bad_vertex() {
        let (mut g, _, [e0, _, _]) = build();
        assert!(g.append_vertex_to_hyperedge(e0, VertexIndex(999)).is_err());
    }
}
