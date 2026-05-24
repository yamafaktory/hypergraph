use std::collections::VecDeque;

use ahash::AHashSet;

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
    /// Returns `true` if `to` is reachable from `from` via directed hyperedges.
    ///
    /// A vertex is always considered reachable from itself.
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if either index does not exist.
    pub fn is_reachable(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<bool, HypergraphError<V, HE>> {
        if !self.vertices.contains_key(&from) {
            return Err(HypergraphError::VertexIndexNotFound(from));
        }
        if !self.vertices.contains_key(&to) {
            return Err(HypergraphError::VertexIndexNotFound(to));
        }

        if from == to {
            return Ok(true);
        }

        let mut visited: AHashSet<VertexIndex> = AHashSet::new();
        let mut queue: VecDeque<VertexIndex> = VecDeque::new();

        visited.insert(from);
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.get_adjacent_vertices_from(current)? {
                if neighbor == to {
                    return Ok(true);
                }
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }

        Ok(false)
    }
}
