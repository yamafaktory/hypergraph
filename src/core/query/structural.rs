use std::collections::VecDeque;

use ahash::{
    AHashMap,
    AHashSet,
};

use super::HypergraphQuery;
use crate::{
    HyperedgeTrait,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

pub(super) fn is_acyclic<V, HE, Q>(q: &Q) -> bool
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    q.topological_sort().is_ok()
}

pub(super) fn topological_sort<V, HE, Q>(q: &Q) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    use std::{
        cmp::Reverse,
        collections::BinaryHeap,
    };

    let all_vertices = q.vertex_indices()?;
    let vertex_count = all_vertices.len();

    let mut in_degree: AHashMap<VertexIndex, usize> =
        all_vertices.iter().map(|&v| (v, 0)).collect();

    for &v in &all_vertices {
        for neighbor in q.get_adjacent_vertices_from(v)? {
            *in_degree.entry(neighbor).or_insert(0) += 1;
        }
    }

    let mut heap: BinaryHeap<Reverse<VertexIndex>> = in_degree
        .iter()
        .filter_map(|(&v, &deg)| (deg == 0).then_some(Reverse(v)))
        .collect();

    let mut result: Vec<VertexIndex> = Vec::with_capacity(vertex_count);

    while let Some(Reverse(current)) = heap.pop() {
        result.push(current);
        for neighbor in q.get_adjacent_vertices_from(current)? {
            let deg = in_degree.entry(neighbor).or_insert(0);
            *deg -= 1;
            if *deg == 0 {
                heap.push(Reverse(neighbor));
            }
        }
    }

    if result.len() == vertex_count {
        Ok(result)
    } else {
        Err(HypergraphError::HypergraphContainsCycle)
    }
}

pub(super) fn strongly_connected_components<V, HE, Q>(
    q: &Q,
) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let mut all_vertices = q.vertex_indices()?;
    all_vertices.sort();

    let mut visited: AHashSet<VertexIndex> = AHashSet::new();
    let mut finish_order: Vec<VertexIndex> = Vec::new();

    for &start in &all_vertices {
        if visited.contains(&start) {
            continue;
        }
        let mut stack: Vec<(VertexIndex, bool)> = vec![(start, false)];
        while let Some((v, exiting)) = stack.pop() {
            if exiting {
                finish_order.push(v);
                continue;
            }
            if !visited.insert(v) {
                continue;
            }
            stack.push((v, true));
            for neighbor in q.get_adjacent_vertices_from(v)? {
                if !visited.contains(&neighbor) {
                    stack.push((neighbor, false));
                }
            }
        }
    }

    let mut visited2: AHashSet<VertexIndex> = AHashSet::new();
    let mut sccs: Vec<Vec<VertexIndex>> = Vec::new();

    for &start in finish_order.iter().rev() {
        if visited2.contains(&start) {
            continue;
        }
        let mut scc: Vec<VertexIndex> = Vec::new();
        let mut stack: Vec<VertexIndex> = vec![start];
        visited2.insert(start);
        while let Some(v) = stack.pop() {
            scc.push(v);
            for predecessor in q.get_adjacent_vertices_to(v)? {
                if visited2.insert(predecessor) {
                    stack.push(predecessor);
                }
            }
        }
        scc.sort();
        sccs.push(scc);
    }

    Ok(sccs)
}

pub(super) fn connected_components<V, HE, Q>(
    q: &Q,
) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let mut all_vertices = q.vertex_indices()?;
    all_vertices.sort();

    let mut visited: AHashSet<VertexIndex> = AHashSet::new();
    let mut components: Vec<Vec<VertexIndex>> = Vec::new();

    for start in all_vertices {
        if visited.contains(&start) {
            continue;
        }
        let mut component: Vec<VertexIndex> = Vec::new();
        let mut queue: VecDeque<VertexIndex> = VecDeque::new();
        visited.insert(start);
        queue.push_back(start);
        while let Some(current) = queue.pop_front() {
            component.push(current);
            for neighbor in q.get_adjacent_vertices_from(current)? {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
            for neighbor in q.get_adjacent_vertices_to(current)? {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }
        component.sort();
        components.push(component);
    }

    Ok(components)
}

#[cfg(test)]
mod tests {
    use crate::{
        Hypergraph,
        core::test_support::{
            E,
            W,
            build,
        },
    };

    #[test]
    fn is_acyclic_true() {
        let (g, _, _) = build();
        assert!(g.is_acyclic());
    }

    #[test]
    fn is_acyclic_false() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let v0 = g.add_vertex(W(0)).unwrap();
        let v1 = g.add_vertex(W(1)).unwrap();
        g.add_hyperedge(vec![v0, v1], E(1)).unwrap();
        g.add_hyperedge(vec![v1, v0], E(1)).unwrap();
        assert!(!g.is_acyclic());
    }

    #[test]
    fn topological_sort_basic() {
        let (g, _, _) = build();
        let order = g.topological_sort().unwrap();
        assert!(!order.is_empty());
    }

    #[test]
    fn topological_sort_cycle_error() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let v0 = g.add_vertex(W(0)).unwrap();
        let v1 = g.add_vertex(W(1)).unwrap();
        g.add_hyperedge(vec![v0, v1], E(1)).unwrap();
        g.add_hyperedge(vec![v1, v0], E(1)).unwrap();
        assert!(g.topological_sort().is_err());
    }

    #[test]
    fn strongly_connected_components_basic() {
        let (g, _, _) = build();
        let sccs = g.strongly_connected_components().unwrap();
        assert!(!sccs.is_empty());
    }

    #[test]
    fn strongly_connected_components_cycle() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let v0 = g.add_vertex(W(0)).unwrap();
        let v1 = g.add_vertex(W(1)).unwrap();
        g.add_hyperedge(vec![v0, v1], E(1)).unwrap();
        g.add_hyperedge(vec![v1, v0], E(1)).unwrap();
        let sccs = g.strongly_connected_components().unwrap();
        let big_scc = sccs.iter().find(|s| s.len() == 2).unwrap();
        assert!(big_scc.contains(&v0) && big_scc.contains(&v1));
    }

    #[test]
    fn connected_components_basic() {
        let (g, _, _) = build();
        let comps = g.connected_components().unwrap();
        assert_eq!(comps.len(), 1);
    }

    #[test]
    fn connected_components_disconnected() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let v0 = g.add_vertex(W(0)).unwrap();
        let v1 = g.add_vertex(W(1)).unwrap();
        let v2 = g.add_vertex(W(2)).unwrap();
        g.add_hyperedge(vec![v0, v1], E(1)).unwrap();
        let _ = v2;
        let comps = g.connected_components().unwrap();
        assert_eq!(comps.len(), 2);
    }
}
