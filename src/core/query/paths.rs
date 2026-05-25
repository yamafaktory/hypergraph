use std::collections::BinaryHeap;

use ahash::AHashMap;

use super::HypergraphQuery;
use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    VertexIndex,
    VertexTrait,
    core::shared::Visitor,
    errors::HypergraphError,
};

pub(super) type DijkstraPath<V, HE> =
    Result<(usize, Vec<(VertexIndex, Option<HyperedgeIndex>)>), HypergraphError<V, HE>>;

pub(super) fn dijkstra_pair<V, HE, Q>(
    graph: &Q,
    from: VertexIndex,
    to: VertexIndex,
) -> DijkstraPath<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    graph.get_vertex_weight(from)?;
    graph.get_vertex_weight(to)?;

    let mut distances: AHashMap<VertexIndex, usize> = AHashMap::new();
    let mut predecessors: AHashMap<VertexIndex, (VertexIndex, Option<HyperedgeIndex>)> =
        AHashMap::new();
    let mut heap = BinaryHeap::new();

    distances.insert(from, 0);
    heap.push(Visitor::new(0, from));

    while let Some(Visitor { distance, index }) = heap.pop() {
        if index == to {
            let mut path = Vec::new();
            let mut cur = to;
            while cur != from {
                let (prev, he) = predecessors[&cur];
                path.push((cur, he));
                cur = prev;
            }
            path.push((from, None));
            path.reverse();
            return Ok((distance, path));
        }

        if distance > distances[&index] {
            continue;
        }

        for (neighbor, he_indices) in graph.get_full_adjacent_vertices_from(index)? {
            let mut min_cost = usize::MAX;
            let mut best_he: Option<HyperedgeIndex> = None;
            for he_idx in he_indices {
                let cost: usize = graph.get_hyperedge_weight(he_idx)?.into();
                if cost < min_cost {
                    min_cost = cost;
                    best_he = Some(he_idx);
                }
            }
            let next = distance + min_cost;
            if distances.get(&neighbor).is_none_or(|&d| next < d) {
                distances.insert(neighbor, next);
                predecessors.insert(neighbor, (index, best_he));
                heap.push(Visitor::new(next, neighbor));
            }
        }
    }

    Ok((0, vec![]))
}

pub(super) fn dijkstra_from<V, HE, Q>(
    graph: &Q,
    from: VertexIndex,
) -> Result<AHashMap<VertexIndex, usize>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    graph.get_vertex_weight(from)?;

    let mut distances: AHashMap<VertexIndex, usize> = AHashMap::new();
    let mut heap = BinaryHeap::new();

    distances.insert(from, 0);
    heap.push(Visitor::new(0, from));

    while let Some(Visitor { distance, index }) = heap.pop() {
        if distance > distances[&index] {
            continue;
        }

        for (neighbor, he_indices) in graph.get_full_adjacent_vertices_from(index)? {
            let mut min_cost = usize::MAX;
            for he_idx in he_indices {
                let cost: usize = graph.get_hyperedge_weight(he_idx)?.into();
                if cost < min_cost {
                    min_cost = cost;
                }
            }
            let next = distance + min_cost;
            if distances.get(&neighbor).is_none_or(|&d| next < d) {
                distances.insert(neighbor, next);
                heap.push(Visitor::new(next, neighbor));
            }
        }
    }

    Ok(distances)
}

#[allow(clippy::type_complexity)]
pub(super) fn get_dijkstra_connections<V, HE, Q>(
    q: &Q,
    from: VertexIndex,
    to: VertexIndex,
) -> Result<Vec<(VertexIndex, Option<HyperedgeIndex>)>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    dijkstra_pair(q, from, to).map(|(_, path)| path)
}

#[allow(clippy::type_complexity)]
pub(super) fn get_dijkstra_connections_with_cost<V, HE, Q>(
    q: &Q,
    from: VertexIndex,
    to: VertexIndex,
) -> Result<(usize, Vec<(VertexIndex, Option<HyperedgeIndex>)>), HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    dijkstra_pair(q, from, to)
}

pub(super) fn get_dijkstra_from<V, HE, Q>(
    q: &Q,
    from: VertexIndex,
) -> Result<AHashMap<VertexIndex, usize>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    dijkstra_from(q, from)
}

#[cfg(test)]
mod tests {
    use crate::core::test_support::build;

    #[test]
    fn get_dijkstra_connections_basic() {
        let (g, [v0, v1, v2, _], _) = build();
        let path = g.get_dijkstra_connections(v0, v2).unwrap();
        assert!(!path.is_empty());
        assert_eq!(path[0].0, v0);
        assert_eq!(path.last().unwrap().0, v2);
        let _ = v1;
    }

    #[test]
    fn get_dijkstra_connections_with_cost_basic() {
        let (g, [v0, _, v2, _], _) = build();
        let (cost, path) = g.get_dijkstra_connections_with_cost(v0, v2).unwrap();
        assert!(!path.is_empty());
        assert!(cost > 0);
    }

    #[test]
    fn get_dijkstra_from_basic() {
        let (g, [v0, _, _, _], _) = build();
        let distances = g.get_dijkstra_from(v0).unwrap();
        assert!(distances.contains_key(&v0));
        assert_eq!(distances[&v0], 0);
    }

    #[test]
    fn get_dijkstra_connections_no_path() {
        let (g, [v0, _, v2, _], _) = build();
        let path = g.get_dijkstra_connections(v2, v0).unwrap();
        assert!(path.is_empty());
    }
}
