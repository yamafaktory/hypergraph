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
    /// Returns the vertices reachable from `from` in depth-first order.
    ///
    /// The starting vertex is always the first element of the result.
    /// Only vertices reachable via directed hyperedges are included.
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `from` does not exist.
    pub fn get_dfs(&self, from: VertexIndex) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        if !self.vertices.contains_key(&from) {
            return Err(HypergraphError::VertexIndexNotFound(from));
        }

        let mut visited: AHashSet<VertexIndex> = AHashSet::new();
        let mut stack: Vec<VertexIndex> = vec![from];
        let mut result: Vec<VertexIndex> = Vec::new();

        while let Some(current) = stack.pop() {
            if visited.insert(current) {
                result.push(current);

                let neighbors = self.get_adjacent_vertices_from(current)?;
                for neighbor in neighbors.into_iter().rev() {
                    if !visited.contains(&neighbor) {
                        stack.push(neighbor);
                    }
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
        let result = g.get_dfs(v0).unwrap();
        assert_eq!(result[0], v0);
        assert_eq!(result[1], v1);
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.get_dfs(VertexIndex(99)).is_err());
    }
}
