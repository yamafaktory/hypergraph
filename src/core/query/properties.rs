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

/// A single row of the nestedness profile for a hyperedge size class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NestednessEntry {
    /// Distinct-vertex count defining this size class.
    pub size: usize,
    /// Hyperedges of this size that are a strict subset of some other hyperedge.
    pub included: usize,
    /// Total hyperedges of this size.
    pub total: usize,
}

pub(super) fn get_orphan_vertices<V, HE, Q>(
    q: &Q,
) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let mut result = Vec::new();
    for idx in q.vertex_indices()? {
        if q.get_vertex_hyperedges(idx)?.is_empty() {
            result.push(idx);
        }
    }
    Ok(result)
}

pub(super) fn get_orphan_hyperedges<V, HE, Q>(
    q: &Q,
) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let mut result = Vec::new();
    for idx in q.hyperedge_indices()? {
        if q.get_hyperedge_vertices(idx)?.is_empty() {
            result.push(idx);
        }
    }
    Ok(result)
}

pub(super) fn get_endpoints<V, HE, Q>(
    q: &Q,
) -> Result<(Vec<VertexIndex>, Vec<VertexIndex>), HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let mut sources = Vec::new();
    let mut sinks = Vec::new();
    for idx in q.vertex_indices()? {
        if q.get_vertex_degree_in(idx)? == 0 {
            sources.push(idx);
        }
        if q.get_vertex_degree_out(idx)? == 0 {
            sinks.push(idx);
        }
    }
    Ok((sources, sinks))
}

pub(super) fn get_inclusions<V, HE, Q>(
    q: &Q,
) -> Result<Vec<(HyperedgeIndex, HyperedgeIndex)>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let he_indices = q.hyperedge_indices()?;
    let mut result = Vec::new();
    for &a in &he_indices {
        let verts_a: AHashSet<VertexIndex> = q.get_hyperedge_vertices(a)?.into_iter().collect();
        for &b in &he_indices {
            if a == b {
                continue;
            }
            let verts_b: AHashSet<VertexIndex> = q.get_hyperedge_vertices(b)?.into_iter().collect();
            if verts_a.len() < verts_b.len() && verts_a.iter().all(|v| verts_b.contains(v)) {
                result.push((a, b));
            }
        }
    }
    Ok(result)
}

pub(super) fn is_k_uniform<V, HE, Q>(q: &Q, k: usize) -> Result<bool, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    for idx in q.hyperedge_indices()? {
        if q.get_hyperedge_vertices(idx)?.len() != k {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn find_cut_vertices<V, HE, Q>(q: &Q) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let v_indices = q.vertex_indices()?;
    let he_indices = q.hyperedge_indices()?;

    let mut adj: AHashMap<VertexIndex, AHashSet<VertexIndex>> = AHashMap::default();
    for &vi in &v_indices {
        adj.entry(vi).or_default();
    }
    for &hei in &he_indices {
        let verts = q.get_hyperedge_vertices(hei)?;
        for i in 0..verts.len() {
            for j in (i + 1)..verts.len() {
                adj.entry(verts[i]).or_default().insert(verts[j]);
                adj.entry(verts[j]).or_default().insert(verts[i]);
            }
        }
    }

    let mut disc: AHashMap<VertexIndex, usize> = AHashMap::default();
    let mut low: AHashMap<VertexIndex, usize> = AHashMap::default();
    let mut is_cut: AHashSet<VertexIndex> = AHashSet::default();
    let mut timer = 0usize;

    for &start in &v_indices {
        if disc.contains_key(&start) {
            continue;
        }
        disc.insert(start, timer);
        low.insert(start, timer);
        timer += 1;

        let mut init_nbrs: Vec<VertexIndex> = adj[&start].iter().copied().collect();
        init_nbrs.sort_unstable();
        // Stack frame: (vertex, parent, sorted_neighbor_list, current_index)
        let mut stack: Vec<(VertexIndex, Option<VertexIndex>, Vec<VertexIndex>, usize)> =
            vec![(start, None, init_nbrs, 0)];
        let mut root_children = 0usize;

        loop {
            if stack.is_empty() {
                break;
            }
            let (u, par_u, cur_idx, nbrs_len) = {
                let top = stack.last().unwrap();
                (top.0, top.1, top.3, top.2.len())
            };

            if cur_idx < nbrs_len {
                let v = stack.last().unwrap().2[cur_idx];
                stack.last_mut().unwrap().3 += 1;

                if !disc.contains_key(&v) {
                    if par_u.is_none() {
                        root_children += 1;
                    }
                    disc.insert(v, timer);
                    low.insert(v, timer);
                    timer += 1;
                    let mut child_nbrs: Vec<VertexIndex> = adj[&v].iter().copied().collect();
                    child_nbrs.sort_unstable();
                    stack.push((v, Some(u), child_nbrs, 0));
                } else if Some(v) != par_u {
                    let dv = disc[&v];
                    let lu = low.get_mut(&u).unwrap();
                    if dv < *lu {
                        *lu = dv;
                    }
                }
            } else {
                stack.pop();
                if let Some(pf) = stack.last() {
                    let p = pf.0;
                    let lu = low[&u];
                    {
                        let lp = low.get_mut(&p).unwrap();
                        if lu < *lp {
                            *lp = lu;
                        }
                    }
                    if p != start && lu >= disc[&p] {
                        is_cut.insert(p);
                    }
                }
            }
        }

        if root_children >= 2 {
            is_cut.insert(start);
        }
    }

    let mut result: Vec<VertexIndex> = is_cut.into_iter().collect();
    result.sort_unstable();
    Ok(result)
}

pub(super) fn get_nestedness_profile<V, HE, Q>(
    q: &Q,
) -> Result<Vec<NestednessEntry>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let he_indices = q.hyperedge_indices()?;
    let mut size_of: AHashMap<HyperedgeIndex, usize> = AHashMap::new();
    let mut total_by_size: AHashMap<usize, usize> = AHashMap::new();

    for &hei in &he_indices {
        let verts = q.get_hyperedge_vertices(hei)?;
        let distinct: AHashSet<VertexIndex> = verts.into_iter().collect();
        let sz = distinct.len();
        size_of.insert(hei, sz);
        *total_by_size.entry(sz).or_insert(0) += 1;
    }

    let subsets = q.get_inclusions()?;
    let mut included_by_size: AHashMap<usize, usize> = AHashMap::new();
    let mut counted: AHashSet<HyperedgeIndex> = AHashSet::new();
    for (subset, _superset) in subsets {
        if counted.insert(subset) {
            let sz = size_of[&subset];
            *included_by_size.entry(sz).or_insert(0) += 1;
        }
    }

    let mut entries: Vec<NestednessEntry> = total_by_size
        .into_iter()
        .map(|(size, total)| NestednessEntry {
            size,
            included: *included_by_size.get(&size).unwrap_or(&0),
            total,
        })
        .collect();
    entries.sort_by_key(|e| e.size);
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use crate::{
        Hypergraph,
        HypergraphQuery,
        core::test_support::{
            E,
            W,
            build,
        },
    };

    #[test]
    fn get_orphan_vertices_none() {
        let (g, _, _) = build();
        assert!(g.get_orphan_vertices().unwrap().is_empty());
    }

    #[test]
    fn get_orphan_hyperedges_none() {
        let (g, _, _) = build();
        assert!(g.get_orphan_hyperedges().unwrap().is_empty());
    }

    #[test]
    fn get_endpoints_sources_and_sinks() {
        let (g, [v0, _, v2, v3], _) = build();
        let (sources, mut sinks) = g.get_endpoints().unwrap();
        assert_eq!(sources, vec![v0]);
        sinks.sort();
        assert_eq!(sinks, vec![v2, v3]);
    }

    #[test]
    fn get_inclusions_empty() {
        let (g, _, _) = build();
        assert!(g.get_inclusions().unwrap().is_empty());
    }

    #[test]
    fn get_inclusions_found() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let v0 = g.add_vertex(W(0)).unwrap();
        let v1 = g.add_vertex(W(1)).unwrap();
        let v2 = g.add_vertex(W(2)).unwrap();
        let big = g.add_hyperedge(vec![v0, v1, v2], E(1)).unwrap();
        let small = g.add_hyperedge(vec![v0, v1], E(2)).unwrap();
        let pairs = g.get_inclusions().unwrap();
        assert!(pairs.contains(&(small, big)));
        assert!(!pairs.contains(&(big, small)));
    }

    #[test]
    fn is_k_uniform_match() {
        let (g, _, _) = build();
        assert!(g.is_k_uniform(2).unwrap());
    }

    #[test]
    fn is_k_uniform_mismatch() {
        let (g, _, _) = build();
        assert!(!g.is_k_uniform(3).unwrap());
    }

    #[test]
    fn find_cut_vertices_star_center() {
        let (g, [_, v1, _, _], _) = build();
        assert!(g.find_cut_vertices().unwrap().contains(&v1));
    }

    #[test]
    fn find_cut_vertices_cycle_empty() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let v0 = g.add_vertex(W(0)).unwrap();
        let v1 = g.add_vertex(W(1)).unwrap();
        let v2 = g.add_vertex(W(2)).unwrap();
        g.add_hyperedge(vec![v0, v1], E(1)).unwrap();
        g.add_hyperedge(vec![v1, v2], E(1)).unwrap();
        g.add_hyperedge(vec![v2, v0], E(1)).unwrap();
        assert!(g.find_cut_vertices().unwrap().is_empty());
    }

    #[test]
    fn get_nestedness_profile_no_inclusions() {
        let (g, _, _) = build();
        let profile = HypergraphQuery::get_nestedness_profile(&g).unwrap();
        // All edges have size 2, none are subsets of each other.
        assert_eq!(profile.len(), 1);
        assert_eq!(profile[0].size, 2);
        assert_eq!(profile[0].total, 3);
        assert_eq!(profile[0].included, 0);
    }

    #[test]
    fn get_nestedness_profile_with_inclusion() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let v0 = g.add_vertex(W(0)).unwrap();
        let v1 = g.add_vertex(W(1)).unwrap();
        let v2 = g.add_vertex(W(2)).unwrap();
        g.add_hyperedge(vec![v0, v1, v2], E(1)).unwrap();
        g.add_hyperedge(vec![v0, v1], E(2)).unwrap();
        let profile = HypergraphQuery::get_nestedness_profile(&g).unwrap();
        // size=2 edge is included; size=3 edge is superset.
        let size2 = profile.iter().find(|e| e.size == 2).unwrap();
        assert_eq!(size2.included, 1);
        assert_eq!(size2.total, 1);
        let size3 = profile.iter().find(|e| e.size == 3).unwrap();
        assert_eq!(size3.included, 0);
        assert_eq!(size3.total, 1);
    }
}
