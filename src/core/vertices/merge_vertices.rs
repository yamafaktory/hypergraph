use crate::{
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
    /// Merges all `vertices` into `vertices[0]` (the primary vertex).
    ///
    /// Every occurrence of a non-primary vertex in any hyperedge is replaced
    /// with the primary. Consecutive duplicate primary→primary occurrences are
    /// removed. Non-primary vertices are then removed from the graph and their
    /// back-reference sets are cleared.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::NotEnoughVerticesProvided`] if fewer than two
    /// vertices are supplied, or [`HypergraphError::VertexIndexNotFound`] if any
    /// index does not exist.
    pub fn merge_vertices(
        &mut self,
        vertices: &[VertexIndex],
    ) -> Result<(), HypergraphError<V, HE>> {
        if vertices.len() < 2 {
            return Err(HypergraphError::NotEnoughVerticesProvided);
        }

        let primary = vertices[0];
        let secondary: Vec<VertexIndex> = vertices[1..].to_vec();

        // Validate all vertices exist.
        if !self.vertices.contains_key(&primary) {
            return Err(HypergraphError::VertexIndexNotFound(primary));
        }
        for &s in &secondary {
            if !self.vertices.contains_key(&s) {
                return Err(HypergraphError::VertexIndexNotFound(s));
            }
        }

        // Collect all hyperedge indices (to avoid borrow issues).
        let he_indices: Vec<_> = self.hyperedges.keys().copied().collect();

        for hei in he_indices {
            let verts = self.hyperedges.get(&hei).unwrap().0.clone();
            if !verts.iter().any(|v| secondary.contains(v)) {
                continue;
            }

            // Replace secondaries with primary.
            let mut new_verts: Vec<VertexIndex> = verts
                .iter()
                .map(|&v| if secondary.contains(&v) { primary } else { v })
                .collect();

            // Remove consecutive primary→primary duplicates.
            new_verts.dedup();

            // Update back-refs: add hei to primary's back-ref if primary is now in the list.
            if new_verts.contains(&primary)
                && let Some((_, he_set)) = self.vertices.get_mut(&primary)
            {
                he_set.insert(hei);
            }

            // Remove hei from secondary back-refs.
            for &s in &secondary {
                if !new_verts.contains(&s)
                    && let Some((_, he_set)) = self.vertices.get_mut(&s)
                {
                    he_set.swap_remove(&hei);
                }
            }

            self.hyperedges.get_mut(&hei).unwrap().0 = new_verts;
        }

        // Remove secondary vertices from the graph.
        for &s in &secondary {
            self.vertices.swap_remove(&s);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        VertexIndex,
        core::test_support::build,
    };

    #[test]
    fn merge_vertices_basic() {
        let (mut g, [v0, v1, v2, v3], [_e0, e1, _e2]) = build();
        // Merge v2 into v1 (primary=v1, secondary=v2)
        g.merge_vertices(&[v1, v2]).unwrap();
        // e1=[v1,v2] becomes [v1] after dedup
        let verts = g.get_hyperedge_vertices(e1).unwrap();
        assert!(!verts.contains(&v2));
        // v2 should be gone from the graph
        assert!(g.get_vertex_weight(v2).is_err());
        let _ = (v0, v3);
    }

    #[test]
    fn merge_too_few_vertices() {
        let (mut g, [v0, _, _, _], _) = build();
        assert!(g.merge_vertices(&[v0]).is_err());
    }

    #[test]
    fn merge_vertices_empty() {
        let (mut g, _, _) = build();
        assert!(g.merge_vertices(&[]).is_err());
    }

    #[test]
    fn merge_vertices_invalid() {
        let (mut g, [v0, _, _, _], _) = build();
        assert!(g.merge_vertices(&[v0, VertexIndex(999)]).is_err());
    }
}
