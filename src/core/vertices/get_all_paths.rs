use ahash::AHashSet;

use crate::{HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait, errors::HypergraphError};

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
        self.get_internal_vertex(from)?;
        self.get_internal_vertex(to)?;

        if from == to {
            return Ok(vec![vec![from]]);
        }

        let mut all_paths: Vec<Vec<VertexIndex>> = Vec::new();
        let mut current_path: Vec<VertexIndex> = vec![from];
        let mut visited: AHashSet<VertexIndex> = AHashSet::from([from]);

        // Each frame: (current vertex, its neighbors, next-neighbor index).
        // Pushing a frame means we have already added `current` to `current_path`
        // and `visited`. Popping a frame undoes both.
        let mut stack: Vec<(VertexIndex, Vec<VertexIndex>, usize)> =
            vec![(from, self.get_adjacent_vertices_from(from)?, 0)];

        while let Some(frame) = stack.last_mut() {
            let (current, neighbors, idx) = frame;
            let current = *current;

            if *idx >= neighbors.len() {
                // All neighbors of `current` explored — backtrack.
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
                // Do not push `to` onto the stack: paths beyond the destination
                // are not simple paths to `to`.
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
