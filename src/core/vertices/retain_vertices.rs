use crate::{HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait, errors::HypergraphError};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Retains only the vertices for which `predicate` returns `true`.
    ///
    /// Removed vertices also drop all hyperedges that contain them. The
    /// predicate receives the stable index and a reference to the weight of
    /// each vertex.
    pub fn retain_vertices<F>(&mut self, mut predicate: F) -> Result<(), HypergraphError<V, HE>>
    where
        F: FnMut(VertexIndex, &V) -> bool,
    {
        let to_remove: Vec<VertexIndex> = self
            .vertices_iter()
            .filter_map(|(idx, weight)| (!predicate(idx, weight)).then_some(idx))
            .collect();

        for idx in to_remove {
            self.remove_vertex(idx)?;
        }

        Ok(())
    }
}
