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
    /// Adds a hyperedge connecting `vertices` with the given `weight`.
    /// Returns the stable index of the hyperedge.
    ///
    /// Duplicate weights are allowed — multiple hyperedges may carry the same
    /// weight value. The unique key is the `(vertices, weight)` combination:
    /// if an identical pair already exists the existing [`HyperedgeIndex`] is
    /// returned without creating a duplicate entry.
    pub fn add_hyperedge(
        &mut self,
        vertices: Vec<VertexIndex>,
        weight: HE,
    ) -> Result<HyperedgeIndex, HypergraphError<V, HE>> {
        if vertices.is_empty() {
            return Err(HypergraphError::HyperedgeCreationNoVertices(weight));
        }

        // Validate that all referenced vertices exist.
        for &v in &vertices {
            if !self.vertices.contains_key(&v) {
                return Err(HypergraphError::VertexIndexNotFound(v));
            }
        }

        // Idempotent insertion: return the existing index if identical entry found.
        if let Some((&existing, _)) = self
            .hyperedges
            .iter()
            .find(|(_, (v, w))| v == &vertices && w == &weight)
        {
            return Ok(existing);
        }

        let he_index = HyperedgeIndex(self.hyperedges_count);
        self.hyperedges_count += 1;

        // Collect unique vertex refs so each vertex HE-set is updated once.
        let unique_verts: AIndexSet<VertexIndex> = vertices.iter().copied().collect();

        self.hyperedges.insert(he_index, (vertices, weight));

        for v in unique_verts {
            if let Some((_, he_set)) = self.vertices.get_mut(&v) {
                he_set.insert(he_index);
            }
        }

        Ok(he_index)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        HyperedgeIndex,
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
        let v = g.add_vertex(W(0)).unwrap();
        let e0 = g.add_hyperedge(vec![v], E(1)).unwrap();
        let e1 = g.add_hyperedge(vec![v], E(2)).unwrap();
        assert_eq!(e0, HyperedgeIndex(0));
        assert_eq!(e1, HyperedgeIndex(1));
    }

    #[test]
    fn empty_vertices_returns_error() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.add_hyperedge(vec![], E(1)).is_err());
    }

    #[test]
    fn missing_vertex_returns_error() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.add_hyperedge(vec![VertexIndex(99)], E(1)).is_err());
    }

    #[test]
    fn identical_entry_is_idempotent() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let v = g.add_vertex(W(0)).unwrap();
        let e0 = g.add_hyperedge(vec![v], E(1)).unwrap();
        let e1 = g.add_hyperedge(vec![v], E(1)).unwrap();
        assert_eq!(e0, e1);
        assert_eq!(g.count_hyperedges(), 1);
    }
}
