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
