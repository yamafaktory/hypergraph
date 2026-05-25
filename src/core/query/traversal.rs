use std::collections::VecDeque;

use ahash::AHashSet;

use super::HypergraphQuery;
use crate::{
    HyperedgeTrait,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

#[allow(clippy::cast_precision_loss)]
pub(super) fn random_walk<V, HE, Q>(
    q: &Q,
    start: VertexIndex,
    steps: usize,
    seed: u64,
) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    // Validate start vertex.
    q.get_vertex_weight(start)?;

    let mut rng_state: u64 = if seed == 0 { 1 } else { seed };
    let mut path: Vec<VertexIndex> = vec![start];
    let mut current = start;

    for _ in 0..steps {
        let hyperedges = q.get_vertex_hyperedges(current)?;
        if hyperedges.is_empty() {
            path.push(current);
            continue;
        }

        // Weighted sampling of hyperedge.
        let weights: Vec<f64> = hyperedges
            .iter()
            .map(|&e| {
                let w = q.get_hyperedge_weight(e)?;
                Ok(Into::<usize>::into(w) as f64)
            })
            .collect::<Result<Vec<_>, HypergraphError<V, HE>>>()?;

        let total: f64 = weights.iter().sum();
        let sample = xorshift64(&mut rng_state) as f64 / u64::MAX as f64 * total;

        let mut chosen_he = *hyperedges.last().unwrap();
        let mut cumulative = 0.0;
        for (i, &w) in weights.iter().enumerate() {
            cumulative += w;
            if sample < cumulative {
                chosen_he = hyperedges[i];
                break;
            }
        }

        // Get distinct vertices of chosen hyperedge excluding current.
        let verts = q.get_hyperedge_vertices(chosen_he)?;
        let distinct: Vec<VertexIndex> = {
            let mut visited: AHashSet<VertexIndex> = AHashSet::new();
            let mut uniq: Vec<VertexIndex> = Vec::new();
            for v in verts {
                if v != current && visited.insert(v) {
                    uniq.push(v);
                }
            }
            uniq
        };

        if distinct.is_empty() {
            path.push(current);
            continue;
        }

        #[allow(clippy::cast_possible_truncation)]
        let next_idx = xorshift64(&mut rng_state) as usize % distinct.len();
        current = distinct[next_idx];
        path.push(current);
    }

    Ok(path)
}

pub(super) fn get_bfs<V, HE, Q>(
    q: &Q,
    from: VertexIndex,
) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    q.get_vertex_weight(from)?;

    let mut visited: AHashSet<VertexIndex> = AHashSet::new();
    let mut queue: VecDeque<VertexIndex> = VecDeque::new();
    let mut result: Vec<VertexIndex> = Vec::new();

    visited.insert(from);
    queue.push_back(from);

    while let Some(current) = queue.pop_front() {
        result.push(current);
        for neighbor in q.get_adjacent_vertices_from(current)? {
            if visited.insert(neighbor) {
                queue.push_back(neighbor);
            }
        }
    }

    Ok(result)
}

pub(super) fn get_dfs<V, HE, Q>(
    q: &Q,
    from: VertexIndex,
) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    q.get_vertex_weight(from)?;

    let mut visited: AHashSet<VertexIndex> = AHashSet::new();
    let mut stack: Vec<VertexIndex> = vec![from];
    let mut result: Vec<VertexIndex> = Vec::new();

    while let Some(current) = stack.pop() {
        if visited.insert(current) {
            result.push(current);
            let neighbors = q.get_adjacent_vertices_from(current)?;
            for neighbor in neighbors.into_iter().rev() {
                if !visited.contains(&neighbor) {
                    stack.push(neighbor);
                }
            }
        }
    }

    Ok(result)
}

pub(super) fn is_reachable<V, HE, Q>(
    q: &Q,
    from: VertexIndex,
    to: VertexIndex,
) -> Result<bool, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    q.get_vertex_weight(from)?;
    q.get_vertex_weight(to)?;

    if from == to {
        return Ok(true);
    }

    let mut visited: AHashSet<VertexIndex> = AHashSet::new();
    let mut queue: VecDeque<VertexIndex> = VecDeque::new();

    visited.insert(from);
    queue.push_back(from);

    while let Some(current) = queue.pop_front() {
        for neighbor in q.get_adjacent_vertices_from(current)? {
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

pub(super) fn get_all_paths<V, HE, Q>(
    q: &Q,
    from: VertexIndex,
    to: VertexIndex,
) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    q.get_vertex_weight(from)?;
    q.get_vertex_weight(to)?;

    if from == to {
        return Ok(vec![vec![from]]);
    }

    let mut all_paths: Vec<Vec<VertexIndex>> = Vec::new();
    let mut current_path: Vec<VertexIndex> = vec![from];
    let mut visited: AHashSet<VertexIndex> = AHashSet::from([from]);
    let mut stack: Vec<(VertexIndex, Vec<VertexIndex>, usize)> =
        vec![(from, q.get_adjacent_vertices_from(from)?, 0)];

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
        let next_neighbors = q.get_adjacent_vertices_from(next)?;
        stack.push((next, next_neighbors, 0));
    }

    Ok(all_paths)
}

#[cfg(test)]
mod tests {
    use crate::core::test_support::build;

    #[test]
    fn get_bfs_basic() {
        let (g, [v0, v1, v2, v3], _) = build();
        let bfs = g.get_bfs(v0).unwrap();
        assert_eq!(bfs[0], v0);
        assert!(bfs.contains(&v1));
        assert!(bfs.contains(&v2));
        assert!(bfs.contains(&v3));
    }

    #[test]
    fn get_dfs_basic() {
        let (g, [v0, v1, v2, v3], _) = build();
        let dfs = g.get_dfs(v0).unwrap();
        assert_eq!(dfs[0], v0);
        assert!(dfs.contains(&v1));
        assert!(dfs.contains(&v2));
        assert!(dfs.contains(&v3));
    }

    #[test]
    fn is_reachable_true() {
        let (g, [v0, _, v2, _], _) = build();
        assert!(g.is_reachable(v0, v2).unwrap());
    }

    #[test]
    fn is_reachable_self() {
        let (g, [v0, _, _, _], _) = build();
        assert!(g.is_reachable(v0, v0).unwrap());
    }

    #[test]
    fn is_reachable_false() {
        let (g, [v0, _, v2, _], _) = build();
        assert!(!g.is_reachable(v2, v0).unwrap());
    }

    #[test]
    fn get_all_paths_basic() {
        let (g, [v0, _, v2, _], _) = build();
        let paths = g.get_all_paths(v0, v2).unwrap();
        assert!(!paths.is_empty());
        assert!(paths.iter().all(|p| p[0] == v0 && *p.last().unwrap() == v2));
    }

    #[test]
    fn get_all_paths_same_vertex() {
        let (g, [v0, _, _, _], _) = build();
        let paths = g.get_all_paths(v0, v0).unwrap();
        assert_eq!(paths, vec![vec![v0]]);
    }

    #[test]
    fn get_bfs_error_unknown() {
        use crate::VertexIndex;
        let (g, _, _) = build();
        let bad = VertexIndex::from(999usize);
        assert!(g.get_bfs(bad).is_err());
    }

    #[test]
    fn get_dfs_error_unknown() {
        use crate::VertexIndex;
        let (g, _, _) = build();
        let bad = VertexIndex::from(999usize);
        assert!(g.get_dfs(bad).is_err());
    }

    #[test]
    fn random_walk_length() {
        use crate::HypergraphQuery;
        let (g, [v0, _, _, _], _) = build();
        let path = HypergraphQuery::random_walk(&g, v0, 5, 42).unwrap();
        assert_eq!(path.len(), 6); // steps + 1
        assert_eq!(path[0], v0);
    }

    #[test]
    fn random_walk_not_found() {
        use crate::{
            HypergraphQuery,
            VertexIndex,
        };
        let (g, _, _) = build();
        assert!(HypergraphQuery::random_walk(&g, VertexIndex(999), 1, 1).is_err());
    }

    #[test]
    fn random_walk_seed_zero() {
        use crate::HypergraphQuery;
        let (g, [v0, _, _, _], _) = build();
        // seed 0 treated as 1 — should not panic
        let path = HypergraphQuery::random_walk(&g, v0, 3, 0).unwrap();
        assert_eq!(path.len(), 4);
    }
}
