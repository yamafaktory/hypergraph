use crate::{
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    core::types::{
        AIndexSet,
        ARandomState,
    },
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Adds a vertex with a custom weight to the hypergraph.
    /// Returns the index of the vertex.
    ///
    /// Duplicate weights are allowed — vertex identity is the returned
    /// [`VertexIndex`], not the weight value.
    pub fn add_vertex(&mut self, weight: V) -> Result<VertexIndex, HypergraphError<V, HE>> {
        let index = VertexIndex(self.vertices_count);
        self.vertices_count += 1;
        self.vertices.insert(
            index,
            (
                weight,
                AIndexSet::with_capacity_and_hasher(0, ARandomState::default()),
            ),
        );
        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Hypergraph,
        VertexIndex,
        core::test_support::{
            E,
            W,
        },
    };

    #[test]
    fn returns_sequential_indices() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        assert_eq!(g.add_vertex(W(0)).unwrap(), VertexIndex(0));
        assert_eq!(g.add_vertex(W(1)).unwrap(), VertexIndex(1));
        assert_eq!(g.add_vertex(W(2)).unwrap(), VertexIndex(2));
    }

    #[test]
    fn duplicate_weights_allowed() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let a = g.add_vertex(W(7)).unwrap();
        let b = g.add_vertex(W(7)).unwrap();
        assert_ne!(a, b);
        assert_eq!(g.count_vertices(), 2);
    }
}
