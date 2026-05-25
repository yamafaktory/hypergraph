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
    /// Merges two or more hyperedges into the first one and removes the rest.
    ///
    /// The vertex lists of all provided hyperedges are concatenated (in the
    /// order they appear in `hyperedges`) and assigned to `hyperedges[0]`. The
    /// remaining hyperedges are then deleted from the graph.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgesInvalidJoin`] if fewer than two
    /// indices are provided, or [`HypergraphError::HyperedgeIndexNotFound`] if
    /// any index does not exist.
    pub fn join_hyperedges(
        &mut self,
        hyperedges: &[HyperedgeIndex],
    ) -> Result<(), HypergraphError<V, HE>> {
        // If the provided hyperedges are less than two, skip the operation.
        if hyperedges.len() < 2 {
            return Err(HypergraphError::HyperedgesInvalidJoin);
        }

        // Try to collect all the vertices from the provided hyperedges.
        match hyperedges
            .iter()
            .map(|hyperedge_index| self.get_hyperedge_vertices(*hyperedge_index))
            .collect::<Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>>>()
        {
            Err(err) => Err(err),
            Ok(joined_vertices) => {
                // The goal is to move all the vertices from the provided
                // hyperedges to the first one.
                self.update_hyperedge_vertices(
                    hyperedges[0],
                    joined_vertices.into_iter().flatten().collect(),
                )?;

                // Get the tail.
                let tail = &hyperedges[1..];

                // Removes the other hyperedges.
                for hyperedge_index in tail {
                    self.remove_hyperedge(*hyperedge_index)?;
                }

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::test_support::build;

    #[test]
    fn joins_two_hyperedges() {
        let (mut g, [v0, v1, v2, _v3], [e0, e1, _e2]) = build();
        g.join_hyperedges(&[e0, e1]).unwrap();
        assert_eq!(g.count_hyperedges(), 2); // e0+e1 merged, e2 remains
        assert_eq!(g.get_hyperedge_vertices(e0).unwrap(), vec![v0, v1, v1, v2]);
    }

    #[test]
    fn too_few_hyperedges_returns_error() {
        let (mut g, _, [e0, _e1, _e2]) = build();
        assert!(g.join_hyperedges(&[e0]).is_err());
    }
}
