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
    /// Returns the number of vertices in the hypergraph.
    #[must_use]
    pub fn count_vertices(&self) -> usize {
        self.vertices.len()
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
    fn empty_graph_is_zero() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert_eq!(g.count_vertices(), 0);
    }

    #[test]
    fn after_additions() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        g.add_vertex(W(0)).unwrap();
        g.add_vertex(W(1)).unwrap();
        assert_eq!(g.count_vertices(), 2);
    }
}
