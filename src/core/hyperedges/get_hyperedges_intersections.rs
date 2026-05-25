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
    /// Returns the vertices present in every hyperedge in `hyperedges`.
    ///
    /// The result is sorted by [`VertexIndex`] and deduplicated. An empty `Vec`
    /// is returned when the hyperedges share no common vertices.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgesInvalidIntersections`] if fewer than
    /// two hyperedge indices are provided, or
    /// [`HypergraphError::HyperedgeIndexNotFound`] if any index does not exist.
    pub fn get_hyperedges_intersections(
        &self,
        hyperedges: &[HyperedgeIndex],
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        let n = hyperedges.len();

        if n < 2 {
            return Err(HypergraphError::HyperedgesInvalidIntersections);
        }

        // Build a unique vertex set per hyperedge.
        let vertex_sets = hyperedges
            .iter()
            .map(|&he_index| {
                self.hyperedges
                    .get(&he_index)
                    .map(|(v, _)| v.iter().copied().collect::<AIndexSet<VertexIndex>>())
                    .ok_or(HypergraphError::HyperedgeIndexNotFound(he_index))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Intersection: vertices present in every hyperedge's set.
        let mut result: Vec<VertexIndex> = vertex_sets[0]
            .iter()
            .filter(|v| vertex_sets[1..].iter().all(|s| s.contains(*v)))
            .copied()
            .collect();

        result.sort();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::test_support::build;

    #[test]
    fn returns_common_vertices() {
        let (g, [_v0, v1, _v2, _v3], [e0, e1, e2]) = build();
        let mut got = g.get_hyperedges_intersections(&[e0, e1, e2]).unwrap();
        got.sort();
        assert_eq!(got, vec![v1]);
    }

    #[test]
    fn too_few_hyperedges_returns_error() {
        let (g, _, [e0, _e1, _e2]) = build();
        assert!(g.get_hyperedges_intersections(&[e0]).is_err());
    }
}
