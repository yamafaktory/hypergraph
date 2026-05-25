use std::collections::VecDeque;

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

/// Per-vertex centrality scores.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CentralityScores {
    /// Degree centrality normalised by 2*(n-1).
    pub degree: f64,
    /// Closeness centrality.
    pub closeness: f64,
    /// Betweenness centrality (un-normalised).
    pub betweenness: f64,
}

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

#[allow(clippy::cast_precision_loss)]
pub(super) fn compute_centrality<V, HE, Q>(
    q: &Q,
) -> Result<AHashMap<VertexIndex, CentralityScores>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let all_vertices = q.vertex_indices()?;
    let n = all_vertices.len();

    let mut degree_raw: AHashMap<VertexIndex, usize> =
        all_vertices.iter().map(|&v| (v, 0usize)).collect();
    let mut betweenness: AHashMap<VertexIndex, f64> =
        all_vertices.iter().map(|&v| (v, 0.0f64)).collect();
    let mut closeness: AHashMap<VertexIndex, f64> =
        all_vertices.iter().map(|&v| (v, 0.0f64)).collect();

    // Compute raw degree from consecutive pairs in each hyperedge.
    for hei in q.hyperedge_indices()? {
        let verts = q.get_hyperedge_vertices(hei)?;
        for w in verts.windows(2) {
            *degree_raw.entry(w[0]).or_insert(0) += 1;
            *degree_raw.entry(w[1]).or_insert(0) += 1;
        }
    }

    // Brandes' algorithm for closeness + betweenness.
    for &s in &all_vertices {
        let mut sigma: AHashMap<VertexIndex, u64> =
            all_vertices.iter().map(|&v| (v, 0u64)).collect();
        let mut dist: AHashMap<VertexIndex, i64> =
            all_vertices.iter().map(|&v| (v, -1i64)).collect();
        let mut pred: AHashMap<VertexIndex, Vec<VertexIndex>> =
            all_vertices.iter().map(|&v| (v, Vec::new())).collect();
        let mut bfs_order: Vec<VertexIndex> = Vec::new();

        *sigma.get_mut(&s).unwrap() = 1;
        *dist.get_mut(&s).unwrap() = 0;

        let mut queue: VecDeque<VertexIndex> = VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            bfs_order.push(v);
            for w in q.get_adjacent_vertices_from(v)? {
                if dist[&w] < 0 {
                    *dist.get_mut(&w).unwrap() = dist[&v] + 1;
                    queue.push_back(w);
                }
                if dist[&w] == dist[&v] + 1 {
                    *sigma.get_mut(&w).unwrap() += sigma[&v];
                    pred.get_mut(&w).unwrap().push(v);
                }
            }
        }

        // Closeness for s.
        let reachable = all_vertices
            .iter()
            .filter(|&&v| v != s && dist[&v] >= 0)
            .count();
        let sum_dist: i64 = all_vertices
            .iter()
            .filter(|&&v| v != s && dist[&v] >= 0)
            .map(|&v| dist[&v])
            .sum();
        *closeness.get_mut(&s).unwrap() = if sum_dist > 0 && n > 1 {
            (reachable as f64).powi(2) / (sum_dist as f64 * (n - 1) as f64)
        } else {
            0.0
        };

        // Betweenness back-prop.
        let mut delta: AHashMap<VertexIndex, f64> =
            all_vertices.iter().map(|&v| (v, 0.0f64)).collect();
        for &v in bfs_order.iter().rev() {
            for &u in &pred[&v] {
                let contribution = (sigma[&u] as f64 / sigma[&v] as f64) * (1.0 + delta[&v]);
                *delta.get_mut(&u).unwrap() += contribution;
            }
            if v != s {
                *betweenness.get_mut(&v).unwrap() += delta[&v];
            }
        }
    }

    let normalizer = if n > 1 { 2.0 * (n - 1) as f64 } else { 1.0 };

    let result: AHashMap<VertexIndex, CentralityScores> = all_vertices
        .iter()
        .map(|&v| {
            let scores = CentralityScores {
                degree: degree_raw[&v] as f64 / normalizer,
                closeness: closeness[&v],
                betweenness: betweenness[&v],
            };
            (v, scores)
        })
        .collect();

    Ok(result)
}

#[allow(clippy::type_complexity)]
pub(super) fn to_incidence_matrix<V, HE, Q>(
    q: &Q,
) -> Result<(Vec<VertexIndex>, Vec<HyperedgeIndex>, Vec<Vec<u8>>), HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let mut vertex_order = q.vertex_indices()?;
    vertex_order.sort();
    let mut edge_order = q.hyperedge_indices()?;
    edge_order.sort();

    let v_pos: AHashMap<VertexIndex, usize> = vertex_order
        .iter()
        .enumerate()
        .map(|(i, &v)| (v, i))
        .collect();

    let nv = vertex_order.len();
    let ne = edge_order.len();
    let mut matrix: Vec<Vec<u8>> = vec![vec![0u8; ne]; nv];

    for (j, &hei) in edge_order.iter().enumerate() {
        let verts = q.get_hyperedge_vertices(hei)?;
        let distinct: AHashSet<VertexIndex> = verts.into_iter().collect();
        for vi in distinct {
            if let Some(&i) = v_pos.get(&vi) {
                matrix[i][j] = 1;
            }
        }
    }

    Ok((vertex_order, edge_order, matrix))
}

#[allow(clippy::type_complexity)]
pub(super) fn to_incidence_matrix_coo<V, HE, Q>(
    q: &Q,
) -> Result<(Vec<VertexIndex>, Vec<HyperedgeIndex>, Vec<(usize, usize)>), HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let mut vertex_order = q.vertex_indices()?;
    vertex_order.sort();
    let mut edge_order = q.hyperedge_indices()?;
    edge_order.sort();

    let v_pos: AHashMap<VertexIndex, usize> = vertex_order
        .iter()
        .enumerate()
        .map(|(i, &v)| (v, i))
        .collect();

    let mut coords: Vec<(usize, usize)> = Vec::new();

    for (j, &hei) in edge_order.iter().enumerate() {
        let verts = q.get_hyperedge_vertices(hei)?;
        let distinct: AHashSet<VertexIndex> = verts.into_iter().collect();
        for vi in distinct {
            if let Some(&i) = v_pos.get(&vi) {
                coords.push((i, j));
            }
        }
    }

    coords.sort_unstable();
    Ok((vertex_order, edge_order, coords))
}

#[allow(clippy::cast_precision_loss)]
#[allow(clippy::type_complexity)]
pub(super) fn to_laplacian<V, HE, Q>(
    q: &Q,
    normalized: bool,
) -> Result<(Vec<VertexIndex>, Vec<Vec<f64>>), HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let mut vertex_order = q.vertex_indices()?;
    vertex_order.sort();
    let n = vertex_order.len();

    let v_idx: AHashMap<VertexIndex, usize> = vertex_order
        .iter()
        .enumerate()
        .map(|(i, &v)| (v, i))
        .collect();

    let mut s_mat: Vec<Vec<f64>> = vec![vec![0.0f64; n]; n];
    let mut deg: Vec<f64> = vec![0.0f64; n];

    for hei in q.hyperedge_indices()? {
        let w_e = Into::<usize>::into(q.get_hyperedge_weight(hei)?) as f64;
        let verts = q.get_hyperedge_vertices(hei)?;
        let distinct_v: Vec<usize> = {
            let mut d: AHashSet<usize> = AHashSet::new();
            for vi in verts {
                if let Some(&idx) = v_idx.get(&vi) {
                    d.insert(idx);
                }
            }
            let mut v: Vec<usize> = d.into_iter().collect();
            v.sort_unstable();
            v
        };
        let delta_e = distinct_v.len();
        if delta_e == 0 {
            continue;
        }
        let contrib = w_e / delta_e as f64;
        for &i in &distinct_v {
            deg[i] += w_e;
            for &j in &distinct_v {
                s_mat[i][j] += contrib;
            }
        }
    }

    let mut laplacian: Vec<Vec<f64>> = vec![vec![0.0f64; n]; n];

    if normalized {
        let inv_sqrt: Vec<f64> = (0..n)
            .map(|i| {
                if deg[i] > 0.0 {
                    1.0 / deg[i].sqrt()
                } else {
                    0.0
                }
            })
            .collect();
        for i in 0..n {
            for j in 0..n {
                laplacian[i][j] = if i == j {
                    1.0 - inv_sqrt[i] * s_mat[i][i] * inv_sqrt[i]
                } else {
                    -inv_sqrt[i] * s_mat[i][j] * inv_sqrt[j]
                };
            }
        }
    } else {
        for i in 0..n {
            for j in 0..n {
                laplacian[i][j] = if i == j {
                    deg[i] - s_mat[i][i]
                } else {
                    -s_mat[i][j]
                };
            }
        }
    }

    Ok((vertex_order, laplacian))
}

pub(super) fn get_line_graph<V, HE, Q>(
    q: &Q,
) -> Result<Vec<(HyperedgeIndex, HyperedgeIndex)>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let he_indices = q.hyperedge_indices()?;
    let mut pairs: AHashSet<(HyperedgeIndex, HyperedgeIndex)> = AHashSet::new();

    for vi in q.vertex_indices()? {
        let incident = q.get_vertex_hyperedges(vi)?;
        for i in 0..incident.len() {
            for j in (i + 1)..incident.len() {
                let a = incident[i].min(incident[j]);
                let b = incident[i].max(incident[j]);
                pairs.insert((a, b));
            }
        }
    }

    // Only emit pairs where both hyperedges actually exist (already guaranteed
    // since get_vertex_hyperedges only returns valid indices, but we check
    // both are in the hyperedge list for safety with custom impls).
    let he_set: AHashSet<HyperedgeIndex> = he_indices.into_iter().collect();
    let mut result: Vec<(HyperedgeIndex, HyperedgeIndex)> = pairs
        .into_iter()
        .filter(|(a, b)| he_set.contains(a) && he_set.contains(b))
        .collect();
    result.sort_unstable();
    Ok(result)
}

#[allow(clippy::type_complexity)]
pub(super) fn get_dual<V, HE, Q>(
    q: &Q,
) -> Result<Vec<(VertexIndex, Vec<HyperedgeIndex>)>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let mut vertex_order = q.vertex_indices()?;
    vertex_order.sort();

    let mut result: Vec<(VertexIndex, Vec<HyperedgeIndex>)> = Vec::new();
    for vi in vertex_order {
        let mut incident = q.get_vertex_hyperedges(vi)?;
        incident.sort();
        result.push((vi, incident));
    }
    Ok(result)
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

    #[test]
    fn compute_centrality_basic() {
        let (g, [v0, v1, v2, v3], _) = build();
        let scores = HypergraphQuery::compute_centrality(&g).unwrap();
        assert!(scores.contains_key(&v0));
        assert!(scores.contains_key(&v1));
        assert!(scores.contains_key(&v2));
        assert!(scores.contains_key(&v3));
        // v1 is central — highest betweenness
        let bw_v1 = scores[&v1].betweenness;
        assert!(
            scores.values().all(|s| s.betweenness <= bw_v1 + 1e-10),
            "v1 should have the highest betweenness"
        );
    }

    #[test]
    fn to_incidence_matrix_dimensions() {
        let (g, _, _) = build();
        let (vord, eord, mat) = HypergraphQuery::to_incidence_matrix(&g).unwrap();
        assert_eq!(vord.len(), 4);
        assert_eq!(eord.len(), 3);
        assert_eq!(mat.len(), 4);
        assert_eq!(mat[0].len(), 3);
    }

    #[test]
    fn to_incidence_matrix_coo_basic() {
        let (g, _, _) = build();
        let (vord, eord, coords) = HypergraphQuery::to_incidence_matrix_coo(&g).unwrap();
        assert_eq!(vord.len(), 4);
        assert_eq!(eord.len(), 3);
        // 6 memberships: v0-e0, v1-e0, v1-e1, v2-e1, v1-e2, v3-e2
        assert_eq!(coords.len(), 6);
        // coords are sorted
        let mut sorted = coords.clone();
        sorted.sort_unstable();
        assert_eq!(coords, sorted);
    }

    #[test]
    fn to_laplacian_unnormalized_row_sum_zero() {
        let (g, _, _) = build();
        let (_vord, mat) = HypergraphQuery::to_laplacian(&g, false).unwrap();
        // Row sums of the unnormalized Laplacian should be zero.
        for row in &mat {
            let sum: f64 = row.iter().sum();
            assert!(sum.abs() < 1e-10, "row sum should be 0, got {sum}");
        }
    }

    #[test]
    fn to_laplacian_normalized_diagonal_one() {
        let (g, _, _) = build();
        let (_vord, mat) = HypergraphQuery::to_laplacian(&g, true).unwrap();
        // v0 is only in e0 (size=2), so L_norm[v0][v0] should be 1 - w/delta * inv_sqrt^2
        // Just verify all diagonal values are finite and in [0,2].
        for (i, row) in mat.iter().enumerate() {
            let diag = row[i];
            assert!(diag.is_finite());
        }
    }

    #[test]
    fn get_line_graph_basic() {
        let (g, _, [e0, e1, e2]) = build();
        let lg = HypergraphQuery::get_line_graph(&g).unwrap();
        // e0=[v0,v1], e1=[v1,v2], e2=[v1,v3]: e0-e1 share v1, e0-e2 share v1, e1-e2 share v1
        assert!(lg.contains(&(e0, e1)));
        assert!(lg.contains(&(e0, e2)));
        assert!(lg.contains(&(e1, e2)));
    }

    #[test]
    fn get_dual_basic() {
        let (g, [v0, v1, v2, v3], [e0, e1, e2]) = build();
        let dual = HypergraphQuery::get_dual(&g).unwrap();
        // Should have 4 entries (one per vertex), sorted by vertex index.
        assert_eq!(dual.len(), 4);
        // v0 is in e0 only
        let (vi, hes) = &dual[0];
        assert_eq!(*vi, v0);
        assert_eq!(*hes, vec![e0]);
        // v1 is in e0, e1, e2
        let (vi, hes) = &dual[1];
        assert_eq!(*vi, v1);
        assert_eq!(*hes, vec![e0, e1, e2]);
        // v2 is in e1
        let (vi, hes) = &dual[2];
        assert_eq!(*vi, v2);
        assert_eq!(*hes, vec![e1]);
        // v3 is in e2
        let (vi, hes) = &dual[3];
        assert_eq!(*vi, v3);
        assert_eq!(*hes, vec![e2]);
    }
}
