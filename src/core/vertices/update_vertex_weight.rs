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
    /// Updates the weight of a vertex by index.
    ///
    /// The new weight need not be unique — multiple vertices may carry the same
    /// weight value. Returns [`HypergraphError::VertexWeightUnchanged`] if
    /// `weight` equals the current weight (no-op guard).
    pub fn update_vertex_weight(
        &mut self,
        vertex_index: VertexIndex,
        weight: V,
    ) -> Result<(), HypergraphError<V, HE>> {
        let (current_weight, _) = self
            .vertices
            .get(&vertex_index)
            .ok_or(HypergraphError::VertexIndexNotFound(vertex_index))?;

        if weight == *current_weight {
            return Err(HypergraphError::VertexWeightUnchanged {
                index: vertex_index,
                weight,
            });
        }

        if let Some((w, _)) = self.vertices.get_mut(&vertex_index) {
            *w = weight;
        }

        Ok(())
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
    fn updates_weight() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let idx = g.add_vertex(W(1)).unwrap();
        g.update_vertex_weight(idx, W(2)).unwrap();
        assert_eq!(g.get_vertex_weight(idx).unwrap(), &W(2));
    }

    #[test]
    fn unchanged_weight_returns_error() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let idx = g.add_vertex(W(1)).unwrap();
        assert!(g.update_vertex_weight(idx, W(1)).is_err());
    }

    #[test]
    fn not_found_returns_error() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.update_vertex_weight(VertexIndex(99), W(1)).is_err());
    }
}
