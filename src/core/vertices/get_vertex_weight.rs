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
    /// Returns a reference to the weight of the vertex at `vertex_index`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `vertex_index` does
    /// not exist.
    pub fn get_vertex_weight(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<&V, HypergraphError<V, HE>> {
        self.vertices
            .get(&vertex_index)
            .map(|(weight, _)| weight)
            .ok_or(HypergraphError::VertexIndexNotFound(vertex_index))
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
    fn returns_weight() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let idx = g.add_vertex(W(5)).unwrap();
        assert_eq!(g.get_vertex_weight(idx).unwrap(), &W(5));
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.get_vertex_weight(VertexIndex(99)).is_err());
    }
}
