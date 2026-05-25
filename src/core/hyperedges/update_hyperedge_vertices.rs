use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    core::types::AIndexSet,
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Replaces the vertex list of the hyperedge at `hyperedge_index` with `vertices`.
    ///
    /// Back-references on newly added vertices and removed vertices are updated
    /// automatically. The weight of the hyperedge is unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeUpdateNoVertices`] if `vertices` is
    /// empty, [`HypergraphError::VertexIndexNotFound`] if any vertex does not
    /// exist, [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist, or [`HypergraphError::HyperedgeVerticesUnchanged`] if the
    /// new list is identical to the current one.
    pub fn update_hyperedge_vertices(
        &mut self,
        hyperedge_index: HyperedgeIndex,
        vertices: Vec<VertexIndex>,
    ) -> Result<(), HypergraphError<V, HE>> {
        if vertices.is_empty() {
            return Err(HypergraphError::HyperedgeUpdateNoVertices(hyperedge_index));
        }

        // Validate that all new vertices exist.
        for &v in &vertices {
            if !self.vertices.contains_key(&v) {
                return Err(HypergraphError::VertexIndexNotFound(v));
            }
        }

        let previous_vertices = self
            .hyperedges
            .get(&hyperedge_index)
            .map(|(v, _)| v.clone())
            .ok_or(HypergraphError::HyperedgeIndexNotFound(hyperedge_index))?;

        if vertices == previous_vertices {
            return Err(HypergraphError::HyperedgeVerticesUnchanged(hyperedge_index));
        }

        let prev_unique: AIndexSet<VertexIndex> = previous_vertices.iter().copied().collect();
        let new_unique: AIndexSet<VertexIndex> = vertices.iter().copied().collect();

        // Add hyperedge ref to newly included vertices.
        for &v in &new_unique {
            if !prev_unique.contains(&v)
                && let Some((_, he_set)) = self.vertices.get_mut(&v)
            {
                he_set.insert(hyperedge_index);
            }
        }

        // Remove hyperedge ref from vertices no longer included.
        for &v in &prev_unique {
            if !new_unique.contains(&v)
                && let Some((_, he_set)) = self.vertices.get_mut(&v)
            {
                he_set.swap_remove(&hyperedge_index);
            }
        }

        if let Some((v, _)) = self.hyperedges.get_mut(&hyperedge_index) {
            *v = vertices;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        HyperedgeIndex,
        Hypergraph,
        core::test_support::{
            E,
            W,
            build,
        },
    };

    #[test]
    fn updates_vertex_list() {
        let (mut g, [v0, _v1, v2, _v3], [e0, _e1, _e2]) = build();
        g.update_hyperedge_vertices(e0, vec![v0, v2]).unwrap();
        assert_eq!(g.get_hyperedge_vertices(e0).unwrap(), vec![v0, v2]);
    }

    #[test]
    fn empty_vertices_returns_error() {
        let (mut g, _, [e0, _e1, _e2]) = build();
        assert!(g.update_hyperedge_vertices(e0, vec![]).is_err());
    }

    #[test]
    fn unchanged_vertices_returns_error() {
        let (mut g, [v0, v1, _v2, _v3], [e0, _e1, _e2]) = build();
        assert!(g.update_hyperedge_vertices(e0, vec![v0, v1]).is_err());
    }

    #[test]
    fn not_found_returns_error() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let v = g.add_vertex(W(0)).unwrap();
        assert!(
            g.update_hyperedge_vertices(HyperedgeIndex(99), vec![v])
                .is_err()
        );
    }
}
