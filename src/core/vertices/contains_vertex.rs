use crate::{
    HyperedgeTrait,
    Hypergraph,
    VertexTrait,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns `true` if at least one vertex with the given weight exists.
    #[must_use]
    pub fn contains_vertex(&self, weight: V) -> bool {
        self.vertices.values().any(|(w, _)| *w == weight)
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
    fn finds_existing() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        g.add_vertex(W(3)).unwrap();
        assert!(g.contains_vertex(W(3)));
    }

    #[test]
    fn returns_false_for_missing() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(!g.contains_vertex(W(99)));
    }
}
