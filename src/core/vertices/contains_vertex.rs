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
    /// Returns `true` if at least one vertex with the given weight exists.
    #[must_use]
    pub fn contains_vertex(&self, weight: V) -> bool {
        self.vertices.values().any(|(w, _)| *w == weight)
    }
}
