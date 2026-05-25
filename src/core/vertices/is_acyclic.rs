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
    /// Returns `true` if the hypergraph contains no directed cycles.
    ///
    /// Implemented as a topological sort; the result is `false` when
    /// [`topological_sort`](Self::topological_sort) would return
    /// [`HypergraphError::HypergraphContainsCycle`](crate::errors::HypergraphError::HypergraphContainsCycle).
    #[must_use]
    pub fn is_acyclic(&self) -> bool {
        self.topological_sort().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Hypergraph,
        core::test_support::{
            E,
            W,
            build,
        },
    };

    #[test]
    fn dag_is_acyclic() {
        let (g, _, _) = build();
        assert!(g.is_acyclic());
    }

    #[test]
    fn cyclic_graph_is_not_acyclic() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let a = g.add_vertex(W(0)).unwrap();
        let b = g.add_vertex(W(1)).unwrap();
        g.add_hyperedge(vec![a, b], E(1)).unwrap();
        g.add_hyperedge(vec![b, a], E(1)).unwrap();
        assert!(!g.is_acyclic());
    }
}
