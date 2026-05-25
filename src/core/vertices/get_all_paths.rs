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
    /// Returns all simple paths (no repeated vertices) from `from` to `to`.
    ///
    /// Each path is a `Vec<VertexIndex>` that includes both endpoints. When
    /// `from == to` the result is `vec![vec![from]]`. Paths are emitted in
    /// DFS discovery order and are not sorted.
    ///
    /// **Warning**: the number of simple paths can grow exponentially with
    /// graph size. Use on large or dense graphs with care.
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if either index does not
    /// exist.
    pub fn get_all_paths(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>> {
        if !self.vertices.contains_key(&from) {
            return Err(HypergraphError::VertexIndexNotFound(from));
        }
        if !self.vertices.contains_key(&to) {
            return Err(HypergraphError::VertexIndexNotFound(to));
        }

        if from == to {
            return Ok(vec![vec![from]]);
        }

        let mut all_paths: Vec<Vec<VertexIndex>> = Vec::new();
        let mut current_path: Vec<VertexIndex> = vec![from];
        let mut visited: AHashSet<VertexIndex> = AHashSet::from([from]);

        // Each frame: (current vertex, its neighbors, next-neighbor index).
        let mut stack: Vec<(VertexIndex, Vec<VertexIndex>, usize)> =
            vec![(from, self.get_adjacent_vertices_from(from)?, 0)];

        while let Some(frame) = stack.last_mut() {
            let (current, neighbors, idx) = frame;
            let current = *current;

            if *idx >= neighbors.len() {
                stack.pop();
                current_path.pop();
                visited.remove(&current);
                continue;
            }

            let next = neighbors[*idx];
            *idx += 1;

            if visited.contains(&next) {
                continue;
            }

            if next == to {
                let mut path = current_path.clone();
                path.push(to);
                all_paths.push(path);
                continue;
            }

            visited.insert(next);
            current_path.push(next);
            let next_neighbors = self.get_adjacent_vertices_from(next)?;
            stack.push((next, next_neighbors, 0));
        }

        Ok(all_paths)
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
    fn finds_path_between_vertices() {
        let (g, [v0, _v1, v2, _v3], _) = build();
        let paths = g.get_all_paths(v0, v2).unwrap();
        assert!(!paths.is_empty());
        assert_eq!(paths[0][0], v0);
        assert_eq!(*paths[0].last().unwrap(), v2);
    }

    #[test]
    fn same_vertex_returns_singleton() {
        let (g, [v0, _v1, _v2, _v3], _) = build();
        assert_eq!(g.get_all_paths(v0, v0).unwrap(), vec![vec![v0]]);
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.get_all_paths(VertexIndex(0), VertexIndex(1)).is_err());
    }
}
