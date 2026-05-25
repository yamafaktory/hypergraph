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
    /// Inserts `vertex_index` at `position` in `hyperedge_index`'s vertex list.
    ///
    /// `position == 0` is equivalent to prepend; `position == len` is equivalent
    /// to append. Positions greater than the current length are an error.
    ///
    /// Back-references are updated: if `vertex_index` was not already in the
    /// hyperedge's vertex list, `hyperedge_index` is added to that vertex's
    /// back-reference set.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist, [`HypergraphError::VertexIndexNotFound`] if `vertex_index`
    /// does not exist, or
    /// [`HypergraphError::HyperedgeInsertIndexOutOfBounds`] if `position` exceeds
    /// the current vertex list length.
    pub fn insert_vertex_into_hyperedge(
        &mut self,
        hyperedge_index: HyperedgeIndex,
        vertex_index: VertexIndex,
        position: usize,
    ) -> Result<(), HypergraphError<V, HE>> {
        if !self.hyperedges.contains_key(&hyperedge_index) {
            return Err(HypergraphError::HyperedgeIndexNotFound(hyperedge_index));
        }
        if !self.vertices.contains_key(&vertex_index) {
            return Err(HypergraphError::VertexIndexNotFound(vertex_index));
        }

        let (verts, _) = self.hyperedges.get(&hyperedge_index).unwrap();
        let len = verts.len();
        if position > len {
            return Err(HypergraphError::HyperedgeInsertIndexOutOfBounds {
                index: hyperedge_index,
                position,
            });
        }
        let already_present = verts.contains(&vertex_index);

        self.hyperedges
            .get_mut(&hyperedge_index)
            .unwrap()
            .0
            .insert(position, vertex_index);

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
    fn insert_at_zero_is_prepend() {
        let (mut g, [v0, v1, v2, _v3], [e0, _, _]) = build();
        g.insert_vertex_into_hyperedge(e0, v2, 0).unwrap();
        let verts = g.get_hyperedge_vertices(e0).unwrap();
        assert_eq!(verts[0], v2);
        assert_eq!(verts[1], v0);
        assert_eq!(verts[2], v1);
    }

    #[test]
    fn insert_at_len_is_append() {
        let (mut g, [v0, v1, v2, _v3], [e0, _, _]) = build();
        g.insert_vertex_into_hyperedge(e0, v2, 2).unwrap();
        let verts = g.get_hyperedge_vertices(e0).unwrap();
        assert_eq!(verts, vec![v0, v1, v2]);
    }

    #[test]
    fn insert_middle() {
        let (mut g, [v0, v1, v2, _v3], [e0, _, _]) = build();
        g.insert_vertex_into_hyperedge(e0, v2, 1).unwrap();
        let verts = g.get_hyperedge_vertices(e0).unwrap();
        assert_eq!(verts, vec![v0, v2, v1]);
    }

    #[test]
    fn insert_out_of_bounds() {
        let (mut g, [_v0, _v1, v2, _v3], [e0, _, _]) = build();
        assert!(g.insert_vertex_into_hyperedge(e0, v2, 99).is_err());
    }

    #[test]
    fn insert_bad_hyperedge() {
        let (mut g, [v0, _, _, _], _) = build();
        assert!(
            g.insert_vertex_into_hyperedge(HyperedgeIndex(999), v0, 0)
                .is_err()
        );
    }

    #[test]
    fn insert_bad_vertex() {
        let (mut g, _, [e0, _, _]) = build();
        assert!(
            g.insert_vertex_into_hyperedge(e0, VertexIndex(999), 0)
                .is_err()
        );
    }
}
