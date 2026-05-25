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
    /// Removes a vertex by index.
    ///
    /// All hyperedges that contain only this vertex are removed. Hyperedges
    /// that contain other vertices are updated with the vertex filtered out.
    pub fn remove_vertex(
        &mut self,
        vertex_index: VertexIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        // Collect the hyperedge indices upfront before any mutation.
        let he_indices = self.get_vertex_hyperedges(vertex_index)?;

        for he_index in he_indices {
            let vertices = self
                .hyperedges
                .get(&he_index)
                .map(|(v, _)| v.clone())
                .ok_or(HypergraphError::HyperedgeIndexNotFound(he_index))?;

            // Determine if this vertex is the sole unique vertex in the hyperedge.
            let mut unique_verts = vertices.clone();
            unique_verts.sort_unstable();
            unique_verts.dedup();

            if unique_verts.len() == 1 {
                self.remove_hyperedge(he_index)?;
            } else {
                let updated: Vec<VertexIndex> = vertices
                    .into_iter()
                    .filter(|&v| v != vertex_index)
                    .collect();
                self.update_hyperedge_vertices(he_index, updated)?;
            }
        }

        self.vertices
            .swap_remove(&vertex_index)
            .ok_or(HypergraphError::VertexIndexNotFound(vertex_index))?;

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
            build,
        },
    };

    #[test]
    fn removes_vertex_and_fixes_hyperedges() {
        let (mut g, [v0, v1, v2, v3], [e0, e1, e2]) = build();
        // v0 only appears in e0; removing it should drop e0 (sole unique member? no: e0 has v0,v1)
        // e0 = [v0, v1]: v0 removed → e0 becomes [v1], a unary
        g.remove_vertex(v0).unwrap();
        assert_eq!(g.count_vertices(), 3);
        // e1 and e2 are untouched (don't contain v0)
        assert_eq!(g.get_hyperedge_vertices(e1).unwrap(), vec![v1, v2]);
        assert_eq!(g.get_hyperedge_vertices(e2).unwrap(), vec![v1, v3]);
        let _ = (e0,); // silence unused warning
    }

    #[test]
    fn not_found_returns_error() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.remove_vertex(VertexIndex(99)).is_err());
    }
}
