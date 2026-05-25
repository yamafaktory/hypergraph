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
    /// Retains only the vertices for which `predicate` returns `true`.
    ///
    /// Removed vertices also drop all hyperedges that contain them. The
    /// predicate receives the stable index and a reference to the weight of
    /// each vertex.
    pub fn retain_vertices<F>(&mut self, mut predicate: F) -> Result<(), HypergraphError<V, HE>>
    where
        F: FnMut(VertexIndex, &V) -> bool,
    {
        let to_remove: Vec<VertexIndex> = self
            .vertices_iter()
            .filter_map(|(idx, weight)| (!predicate(idx, weight)).then_some(idx))
            .collect();

        for idx in to_remove {
            self.remove_vertex(idx)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::test_support::build;

    #[test]
    fn retains_matching_removes_rest() {
        let (mut g, [v0, v1, v2, v3], _) = build();
        // keep only v1 and v2
        g.retain_vertices(|idx, _| idx == v1 || idx == v2).unwrap();
        assert_eq!(g.count_vertices(), 2);
        let _ = (v0, v3); // silence unused warnings
    }

    #[test]
    fn keep_all_leaves_graph_intact() {
        let (mut g, _, _) = build();
        let before = g.count_vertices();
        g.retain_vertices(|_, _| true).unwrap();
        assert_eq!(g.count_vertices(), before);
    }
}
