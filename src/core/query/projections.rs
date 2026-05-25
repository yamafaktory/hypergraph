use ahash::{
    AHashMap,
    AHashSet,
};

use super::HypergraphQuery;
use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

pub(super) fn expand_to_graph<V, HE, Q>(
    q: &Q,
) -> Result<Vec<(VertexIndex, VertexIndex)>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let mut pairs: AHashSet<(VertexIndex, VertexIndex)> = AHashSet::default();
    for hei in q.hyperedge_indices()? {
        let verts = q.get_hyperedge_vertices(hei)?;
        for window in verts.windows(2) {
            pairs.insert((window[0], window[1]));
        }
    }
    let mut result: Vec<(VertexIndex, VertexIndex)> = pairs.into_iter().collect();
    result.sort_unstable();
    Ok(result)
}

pub(super) fn expand_to_star<V, HE, Q>(
    q: &Q,
) -> Result<Vec<(VertexIndex, HyperedgeIndex)>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let mut pairs: AHashSet<(VertexIndex, HyperedgeIndex)> = AHashSet::default();
    for hei in q.hyperedge_indices()? {
        for vi in q.get_hyperedge_vertices(hei)? {
            pairs.insert((vi, hei));
        }
    }
    let mut result: Vec<(VertexIndex, HyperedgeIndex)> = pairs.into_iter().collect();
    result.sort_unstable();
    Ok(result)
}

pub(super) fn get_core<V, HE, Q>(
    q: &Q,
    min_vertex_degree: usize,
    min_edge_size: usize,
) -> Result<(Vec<VertexIndex>, Vec<HyperedgeIndex>), HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let mut active_v: AHashSet<VertexIndex> = q.vertex_indices()?.into_iter().collect();
    let mut active_he: AHashSet<HyperedgeIndex> = q.hyperedge_indices()?.into_iter().collect();

    loop {
        let mut changed = false;

        let remove_he: Vec<HyperedgeIndex> = {
            let mut v = Vec::new();
            for hei in active_he.iter().copied() {
                let count = q
                    .get_hyperedge_vertices(hei)?
                    .iter()
                    .filter(|vi| active_v.contains(vi))
                    .count();
                if count < min_edge_size {
                    v.push(hei);
                }
            }
            v
        };
        for hei in remove_he {
            active_he.remove(&hei);
            changed = true;
        }

        let remove_v: Vec<VertexIndex> = {
            let mut v = Vec::new();
            for vi in active_v.iter().copied() {
                let degree = q
                    .get_vertex_hyperedges(vi)?
                    .iter()
                    .filter(|he| active_he.contains(he))
                    .count();
                if degree < min_vertex_degree {
                    v.push(vi);
                }
            }
            v
        };
        for vi in remove_v {
            active_v.remove(&vi);
            changed = true;
        }

        if !changed {
            break;
        }
    }

    let mut vertices: Vec<VertexIndex> = active_v.into_iter().collect();
    let mut hyperedges: Vec<HyperedgeIndex> = active_he.into_iter().collect();
    vertices.sort_unstable();
    hyperedges.sort_unstable();
    Ok((vertices, hyperedges))
}

#[allow(clippy::cast_precision_loss)]
pub(super) fn compute_page_rank<V, HE, Q>(
    q: &Q,
    damping: f64,
    iterations: usize,
) -> Result<AHashMap<VertexIndex, f64>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let v_indices = q.vertex_indices()?;
    let n = v_indices.len();
    if n == 0 {
        return Ok(AHashMap::default());
    }

    let initial = 1.0 / n as f64;
    let mut rank: AHashMap<VertexIndex, f64> = v_indices.iter().map(|&v| (v, initial)).collect();

    let out_neighbors: AHashMap<VertexIndex, Vec<VertexIndex>> = {
        let mut m: AHashMap<VertexIndex, Vec<VertexIndex>> = AHashMap::default();
        for &v in &v_indices {
            let raw = q.get_adjacent_vertices_from(v)?;
            let mut seen: AHashSet<VertexIndex> = AHashSet::default();
            let deduped: Vec<VertexIndex> = raw.into_iter().filter(|&vi| seen.insert(vi)).collect();
            m.insert(v, deduped);
        }
        m
    };

    for _ in 0..iterations {
        let dangling_sum: f64 = v_indices
            .iter()
            .filter(|&&v| out_neighbors[&v].is_empty())
            .map(|&v| rank[&v])
            .sum();
        let base = (1.0 - damping) / n as f64 + damping * dangling_sum / n as f64;

        let mut new_rank: AHashMap<VertexIndex, f64> =
            v_indices.iter().map(|&v| (v, base)).collect();

        for &u in &v_indices {
            let neighbors = &out_neighbors[&u];
            if neighbors.is_empty() {
                continue;
            }
            let contrib = damping * rank[&u] / neighbors.len() as f64;
            for &v in neighbors {
                *new_rank.get_mut(&v).unwrap() += contrib;
            }
        }

        rank = new_rank;
    }

    Ok(rank)
}

#[cfg(test)]
mod tests {
    use crate::{
        HypergraphQuery,
        core::test_support::build,
    };

    #[test]
    fn expand_to_graph_consecutive_pairs() {
        let (g, [v0, v1, v2, v3], _) = build();
        let pairs = g.expand_to_graph().unwrap();
        assert_eq!(pairs.len(), 3);
        assert!(pairs.contains(&(v0, v1)));
        assert!(pairs.contains(&(v1, v2)));
        assert!(pairs.contains(&(v1, v3)));
    }

    #[test]
    fn expand_to_star_membership() {
        let (g, [v0, v1, v2, v3], [e0, e1, e2]) = build();
        let pairs = g.expand_to_star().unwrap();
        assert_eq!(pairs.len(), 6);
        assert!(pairs.contains(&(v0, e0)));
        assert!(pairs.contains(&(v1, e0)));
        assert!(pairs.contains(&(v1, e1)));
        assert!(pairs.contains(&(v2, e1)));
        assert!(pairs.contains(&(v1, e2)));
        assert!(pairs.contains(&(v3, e2)));
    }

    #[test]
    fn get_core_all_survive() {
        let (g, [v0, v1, v2, v3], [e0, e1, e2]) = build();
        let (vs, hes) = g.get_core(1, 1).unwrap();
        assert_eq!(vs, vec![v0, v1, v2, v3]);
        assert_eq!(hes, vec![e0, e1, e2]);
    }

    #[test]
    fn get_core_peels_to_empty() {
        let (g, _, _) = build();
        let (vs, hes) = g.get_core(2, 2).unwrap();
        assert!(vs.is_empty());
        assert!(hes.is_empty());
    }

    #[test]
    fn compute_page_rank_sums_to_one() {
        let (g, _, _) = build();
        let ranks = g.compute_page_rank(0.85, 100).unwrap();
        assert_eq!(ranks.len(), 4);
        let total: f64 = ranks.values().sum();
        assert!((total - 1.0).abs() < 0.01);
    }

    #[test]
    fn compute_page_rank_central_vertex_highest() {
        let (g, [_, v1, _, _], _) = build();
        let ranks = g.compute_page_rank(0.85, 100).unwrap();
        let v1_rank = ranks[&v1];
        assert!(
            ranks.values().all(|&r| r <= v1_rank + 1e-10),
            "v1 should have the highest rank"
        );
    }
}
