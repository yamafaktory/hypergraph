use crate::{HyperedgeTrait, Hypergraph, VertexTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns `true` if a vertex with the given weight exists in the hypergraph.
    pub fn contains_vertex(&self, weight: V) -> bool {
        self.vertices.contains_key(&weight)
    }
}
