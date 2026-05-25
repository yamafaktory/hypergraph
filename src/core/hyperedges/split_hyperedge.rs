use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    Hypergraph,
    VertexTrait,
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Splits the hyperedge at `hyperedge_index` into two at position `at`.
    ///
    /// The original edge retains `vertices[0..at]` and the new edge gets
    /// `vertices[at..]` with weight `weight`. Returns the [`HyperedgeIndex`]
    /// of the newly created edge.
    ///
    /// Split positions `0` or `>= len` are invalid.
    ///
    /// Back-references are updated automatically.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist, or
    /// [`HypergraphError::HyperedgeSplitIndexOutOfBounds`] if `at == 0` or
    /// `at >= len`.
    pub fn split_hyperedge(
        &mut self,
        hyperedge_index: HyperedgeIndex,
        at: usize,
        weight: HE,
    ) -> Result<HyperedgeIndex, HypergraphError<V, HE>> {
        let full_verts = self
            .hyperedges
            .get(&hyperedge_index)
            .map(|(v, _)| v.clone())
            .ok_or(HypergraphError::HyperedgeIndexNotFound(hyperedge_index))?;

        let len = full_verts.len();
        if at == 0 || at >= len {
            return Err(HypergraphError::HyperedgeSplitIndexOutOfBounds {
                index: hyperedge_index,
                at,
            });
        }

        let tail: Vec<_> = full_verts[at..].to_vec();

        // Create new edge for the tail — add_hyperedge handles new edge's back-refs.
        let new_idx = self.add_hyperedge(tail.clone(), weight)?;

        // Truncate the original to the head.
        self.hyperedges
            .get_mut(&hyperedge_index)
            .unwrap()
            .0
            .truncate(at);

        // For vertices in the tail that no longer appear in the head,
        // remove the back-ref to the original hyperedge.
        let head = &full_verts[..at];
        for v in &tail {
            if !head.contains(v)
                && let Some((_, he_set)) = self.vertices.get_mut(v)
            {
                he_set.swap_remove(&hyperedge_index);
            }
        }

        Ok(new_idx)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        HyperedgeIndex,
        core::test_support::{
            E,
            build,
        },
    };

    #[test]
    fn split_hyperedge_basic() {
        let (mut g, [v0, v1, v2, _v3], _) = build();
        let e_new = g.add_hyperedge(vec![v0, v1, v2], E(10)).unwrap();
        let new_idx = g.split_hyperedge(e_new, 1, E(20)).unwrap();
        // Original should have [v0]
        assert_eq!(g.get_hyperedge_vertices(e_new).unwrap(), vec![v0]);
        // New edge should have [v1,v2]
        assert_eq!(g.get_hyperedge_vertices(new_idx).unwrap(), vec![v1, v2]);
        // v0 back-ref still has e_new
        assert!(g.get_vertex_hyperedges(v0).unwrap().contains(&e_new));
        // v1 no longer in e_new (removed from head), now only in new_idx
        assert!(!g.get_vertex_hyperedges(v1).unwrap().contains(&e_new));
        assert!(g.get_vertex_hyperedges(v1).unwrap().contains(&new_idx));
    }

    #[test]
    fn split_at_zero_is_error() {
        let (mut g, _, [e0, _, _]) = build();
        assert!(g.split_hyperedge(e0, 0, E(99)).is_err());
    }

    #[test]
    fn split_at_len_is_error() {
        let (mut g, _, [e0, _, _]) = build();
        // e0 has 2 vertices, split at 2 >= len is out of bounds
        assert!(g.split_hyperedge(e0, 2, E(99)).is_err());
    }

    #[test]
    fn split_bad_hyperedge() {
        let (mut g, _, _) = build();
        assert!(g.split_hyperedge(HyperedgeIndex(999), 1, E(1)).is_err());
    }
}
