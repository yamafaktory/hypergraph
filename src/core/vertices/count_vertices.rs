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
    pub fn count_vertices(&self) -> usize {
        self.vertices.len()
    }
}
