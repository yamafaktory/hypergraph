use crate::{
    HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait,
    core::types::{AIndexSet, ARandomState},
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
        let internal_index = self.vertices.len();
        self.vertices.push((
            weight,
            AIndexSet::with_capacity_and_hasher(0, ARandomState::default()),
        ));
        Ok(self.add_vertex_index(internal_index))
    }
}
