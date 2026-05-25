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
    fn reachable_via_path() {
        let (g, [v0, _v1, v2, _v3], _) = build();
        assert!(g.is_reachable(v0, v2).unwrap());
    }

    #[test]
    fn self_is_reachable() {
        let (g, [v0, _v1, _v2, _v3], _) = build();
        assert!(g.is_reachable(v0, v0).unwrap());
    }

    #[test]
    fn unreachable_returns_false() {
        let (g, [v0, _v1, _v2, v3], _) = build();
        // v3 has no outgoing edges
        assert!(!g.is_reachable(v3, v0).unwrap());
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.is_reachable(VertexIndex(0), VertexIndex(1)).is_err());
    }
}
