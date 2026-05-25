use std::{
    collections::BinaryHeap,
    iter::successors,
};

use ahash::AHashMap;

use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    core::shared::Visitor,
    errors::HypergraphError,
};

type DijkstraResult<V, HE> =
    Result<(usize, Vec<(VertexIndex, Option<HyperedgeIndex>)>), HypergraphError<V, HE>>;

#[allow(clippy::type_complexity)]
impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    fn dijkstra_impl(&self, from: VertexIndex, to: VertexIndex) -> DijkstraResult<V, HE> {
        if !self.vertices.contains_key(&from) {
            return Err(HypergraphError::VertexIndexNotFound(from));
        }
        if !self.vertices.contains_key(&to) {
            return Err(HypergraphError::VertexIndexNotFound(to));
        }

        let mut distances: AHashMap<VertexIndex, usize> = AHashMap::new();
        let mut predecessors: AHashMap<VertexIndex, (VertexIndex, Option<HyperedgeIndex>)> =
            AHashMap::new();
        let mut to_traverse = BinaryHeap::new();

        distances.insert(from, 0);
        to_traverse.push(Visitor::new(0, from));

        while let Some(Visitor { distance, index }) = to_traverse.pop() {
            if index == to {
                let path = successors(Some(to), |&current| {
                    (current != from).then(|| predecessors[&current].0)
                })
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(|v| {
                    let he = predecessors.get(&v).and_then(|&(_, he)| he);
                    Ok((v, he))
                })
                .collect::<Result<Vec<_>, HypergraphError<V, HE>>>()?;

                return Ok((distance, path));
            }

            if distance > distances[&index] {
                continue;
            }

            let neighbors = self.get_full_adjacent_vertices_from(index)?;

            for (vertex_index, hyperedge_indexes) in neighbors {
                let mut min_cost = usize::MAX;
                let mut best_hyperedge: Option<HyperedgeIndex> = None;

                for hyperedge_index in hyperedge_indexes {
                    let cost: usize = self
                        .get_hyperedge_weight(hyperedge_index)?
                        .to_owned()
                        .into();

                    if cost < min_cost {
                        min_cost = cost;
                        best_hyperedge = Some(hyperedge_index);
                    }
                }

                let next_distance = distance + min_cost;
                let is_shorter = distances
                    .get(&vertex_index)
                    .is_none_or(|&current| next_distance < current);

                if is_shorter {
                    distances.insert(vertex_index, next_distance);
                    predecessors.insert(vertex_index, (index, best_hyperedge));
                    to_traverse.push(Visitor::new(next_distance, vertex_index));
                }
            }
        }

        Ok((0, vec![]))
    }

    /// Gets the cheapest path between two vertices as a vector of
    /// `(VertexIndex, Option<HyperedgeIndex>)` tuples.
    ///
    /// The first element always carries `None` as no hyperedge has been
    /// traversed to reach the starting vertex.
    /// The implementation is based on:
    /// <https://doc.rust-lang.org/std/collections/binary_heap/#examples>
    pub fn get_dijkstra_connections(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<Vec<(VertexIndex, Option<HyperedgeIndex>)>, HypergraphError<V, HE>> {
        self.dijkstra_impl(from, to).map(|(_, path)| path)
    }

    /// Gets the cheapest path between two vertices together with the total cost.
    ///
    /// Returns `(total_cost, path)` where `path` is the same format as
    /// [`get_dijkstra_connections`](Self::get_dijkstra_connections).
    /// When no path exists, returns `(0, [])`.
    pub fn get_dijkstra_connections_with_cost(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<(usize, Vec<(VertexIndex, Option<HyperedgeIndex>)>), HypergraphError<V, HE>> {
        self.dijkstra_impl(from, to)
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
    fn finds_shortest_path() {
        let (g, [v0, _v1, v2, _v3], [e0, e1, _e2]) = build();
        assert_eq!(
            g.get_dijkstra_connections(v0, v2).unwrap(),
            vec![(v0, None), (_v1, Some(e0)), (v2, Some(e1))]
        );
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(
            g.get_dijkstra_connections(VertexIndex(0), VertexIndex(1))
                .is_err()
        );
    }
}
