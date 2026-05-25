use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    Hypergraph,
    VertexTrait,
    core::types::AIndexSet,
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Removes the hyperedge at `hyperedge_index` from the graph.
    ///
    /// Also removes the corresponding back-reference from every vertex that was
    /// part of the hyperedge. Vertex weights and all other hyperedges are
    /// unaffected.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist.
    pub fn remove_hyperedge(
        &mut self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        let (vertices, _) = self
            .hyperedges
            .swap_remove(&hyperedge_index)
            .ok_or(HypergraphError::HyperedgeIndexNotFound(hyperedge_index))?;

        // Remove this hyperedge ref from each unique vertex.
        let unique_verts: AIndexSet<_> = vertices.into_iter().collect();
        for v in unique_verts {
            if let Some((_, he_set)) = self.vertices.get_mut(&v) {
                he_set.swap_remove(&hyperedge_index);
            }
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
    fn removes_hyperedge() {
        let (mut g, _, [e0, _e1, _e2]) = build();
        g.remove_hyperedge(e0).unwrap();
        assert_eq!(g.count_hyperedges(), 2);
    }

    #[test]
    fn not_found_returns_error() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.remove_hyperedge(HyperedgeIndex(99)).is_err());
    }
}
