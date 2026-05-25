use crate::{
    HyperedgeTrait,
    Hypergraph,
    VertexTrait,
    core::types::{
        AIndexSet,
        ARandomState,
    },
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns a new hypergraph containing all original vertices and only those
    /// hyperedges whose raw vertex list length is `<= k`.
    ///
    /// Stable indices are preserved: the returned graph uses the same
    /// [`VertexIndex`](crate::VertexIndex) and
    /// [`HyperedgeIndex`](crate::HyperedgeIndex) values as the original.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError`] if any underlying primitive fails.
    pub fn get_k_skeleton(&self, k: usize) -> Result<Hypergraph<V, HE>, HypergraphError<V, HE>> {
        let mut new_graph = Hypergraph::new();

        // Insert all vertices preserving original indices.
        let mut vertex_list: Vec<_> = self.vertices.iter().collect();
        vertex_list.sort_by_key(|(idx, _)| **idx);
        let max_v = vertex_list.last().map_or(0, |(idx, _)| idx.0 + 1);
        new_graph.vertices_count = max_v;
        for (v_idx, (weight, _)) in &vertex_list {
            let he_set = AIndexSet::with_capacity_and_hasher(0, ARandomState::default());
            new_graph.vertices.insert(**v_idx, (*weight, he_set));
        }

        // Insert hyperedges with raw vertex list length <= k.
        let mut he_list: Vec<_> = self.hyperedges.iter().collect();
        he_list.sort_by_key(|(idx, _)| **idx);
        let mut max_he = 0usize;
        for (he_idx, (verts, weight)) in &he_list {
            if verts.len() <= k {
                // Update back-refs for included vertices.
                let unique: AIndexSet<_> = verts.iter().copied().collect();
                for v in &unique {
                    if let Some((_, he_set)) = new_graph.vertices.get_mut(v) {
                        he_set.insert(**he_idx);
                    }
                }
                new_graph
                    .hyperedges
                    .insert(**he_idx, (verts.clone(), *weight));
                if he_idx.0 + 1 > max_he {
                    max_he = he_idx.0 + 1;
                }
            }
        }
        new_graph.hyperedges_count = max_he;

        Ok(new_graph)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Hypergraph,
        core::test_support::{
            E,
            W,
            build,
        },
    };

    #[test]
    fn k_skeleton_includes_all_edges_of_right_size() {
        let (g, _, _) = build();
        // All edges have size 2, so k=2 keeps everything.
        let skel = g.get_k_skeleton(2).unwrap();
        assert_eq!(skel.count_vertices(), 4);
        assert_eq!(skel.count_hyperedges(), 3);
    }

    #[test]
    fn k_skeleton_filters_large_edges() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let v0 = g.add_vertex(W(0)).unwrap();
        let v1 = g.add_vertex(W(1)).unwrap();
        let v2 = g.add_vertex(W(2)).unwrap();
        g.add_hyperedge(vec![v0, v1], E(1)).unwrap();
        g.add_hyperedge(vec![v0, v1, v2], E(2)).unwrap();
        let skel = g.get_k_skeleton(2).unwrap();
        assert_eq!(skel.count_vertices(), 3);
        assert_eq!(skel.count_hyperedges(), 1);
    }

    #[test]
    fn k_skeleton_preserves_vertex_indices() {
        let (g, [v0, v1, v2, v3], _) = build();
        let skel = g.get_k_skeleton(2).unwrap();
        assert!(skel.get_vertex_weight(v0).is_ok());
        assert!(skel.get_vertex_weight(v1).is_ok());
        assert!(skel.get_vertex_weight(v2).is_ok());
        assert!(skel.get_vertex_weight(v3).is_ok());
    }
}
