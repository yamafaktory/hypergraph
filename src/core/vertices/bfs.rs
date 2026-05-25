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
    /// Returns the vertices reachable from `from` in breadth-first order.
    ///
    /// The starting vertex is always the first element of the result.
    /// Only vertices reachable via directed hyperedges are included.
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `from` does not exist.
    pub fn get_bfs(&self, from: VertexIndex) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        if !self.vertices.contains_key(&from) {
            return Err(HypergraphError::VertexIndexNotFound(from));
        }

        let mut visited: AHashSet<VertexIndex> = AHashSet::new();
        let mut queue: VecDeque<VertexIndex> = VecDeque::new();
        let mut result: Vec<VertexIndex> = Vec::new();

        visited.insert(from);
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            result.push(current);

            for neighbor in self.get_adjacent_vertices_from(current)? {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }

        Ok(result)
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
    fn starts_at_from() {
        let (g, [v0, v1, _v2, _v3], _) = build();
        let result = g.get_bfs(v0).unwrap();
        assert_eq!(result[0], v0);
        assert_eq!(result[1], v1);
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.get_bfs(VertexIndex(99)).is_err());
    }
}
