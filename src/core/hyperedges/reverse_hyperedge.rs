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
    /// Reverses the vertex order of the hyperedge at `hyperedge_index`.
    ///
    /// This inverts the direction of the hyperedge without changing which
    /// vertices it connects. The weight is unchanged. Palindromic vertex
    /// lists (where reversing yields the same sequence) are treated as a
    /// no-op and return `Ok(())`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist.
    pub fn reverse_hyperedge(
        &mut self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        let vertices = self.get_hyperedge_vertices(hyperedge_index)?;
        let reversed: Vec<_> = vertices.iter().copied().rev().collect();

        if reversed == vertices {
            return Ok(());
        }

        self.update_hyperedge_vertices(hyperedge_index, reversed)
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
    fn reverses_vertex_list() {
        let (mut g, [v0, v1, _v2, _v3], [e0, _e1, _e2]) = build();
        g.reverse_hyperedge(e0).unwrap();
        assert_eq!(g.get_hyperedge_vertices(e0).unwrap(), vec![v1, v0]);
    }

    #[test]
    fn palindrome_is_noop() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let a = g.add_vertex(W(0)).unwrap();
        let b = g.add_vertex(W(1)).unwrap();
        let e = g.add_hyperedge(vec![a, b, a], E(1)).unwrap();
        g.reverse_hyperedge(e).unwrap();
        assert_eq!(g.get_hyperedge_vertices(e).unwrap(), vec![a, b, a]);
    }

    #[test]
    fn not_found_returns_error() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.reverse_hyperedge(HyperedgeIndex(99)).is_err());
    }
}
