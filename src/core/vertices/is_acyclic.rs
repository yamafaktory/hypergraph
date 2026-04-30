use crate::{HyperedgeTrait, Hypergraph, VertexTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns `true` if the hypergraph contains no directed cycles.
    ///
    /// Implemented as a topological sort; the result is `false` when
    /// [`topological_sort`](Self::topological_sort) would return
    /// [`HypergraphError::HypergraphContainsCycle`](crate::errors::HypergraphError::HypergraphContainsCycle).
    #[must_use]
    pub fn is_acyclic(&self) -> bool {
        self.topological_sort().is_ok()
    }
}
