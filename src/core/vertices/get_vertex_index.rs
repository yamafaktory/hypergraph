use crate::{
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns the stable indexes of all vertices whose weight equals `weight`.
    ///
    /// Because vertex weights are not required to be unique, multiple indexes
    /// may be returned. Returns an empty `Vec` if no match is found.
    ///
    /// This is the reverse of [`get_vertex_weight`](Self::get_vertex_weight).
    #[must_use]
    pub fn get_vertex_index(&self, weight: V) -> Vec<VertexIndex> {
        self.vertices
            .iter()
            .filter_map(|(&idx, (w, _))| (*w == weight).then_some(idx))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Hypergraph,
        core::test_support::{
            E,
            W,
        },
    };

    #[test]
    fn returns_indices_for_weight() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let i = g.add_vertex(W(5)).unwrap();
        assert_eq!(g.get_vertex_index(W(5)), vec![i]);
    }

    #[test]
    fn returns_empty_for_missing() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.get_vertex_index(W(99)).is_empty());
    }

    #[test]
    fn returns_multiple_for_duplicate_weights() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let a = g.add_vertex(W(1)).unwrap();
        let b = g.add_vertex(W(1)).unwrap();
        let mut got = g.get_vertex_index(W(1));
        got.sort();
        assert_eq!(got, vec![a, b]);
    }
}
