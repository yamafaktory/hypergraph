use ahash::AHashMap;

use super::HypergraphQuery;
use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

pub(super) fn get_adjacent_vertices_from<V, HE, Q>(
    q: &Q,
    from: VertexIndex,
) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let he_indices = q.get_vertex_hyperedges(from)?;
    let mut neighbors: Vec<VertexIndex> = Vec::new();
    for he_idx in he_indices {
        let vertices = q.get_hyperedge_vertices(he_idx)?;
        for w in vertices.windows(2) {
            if w[0] == from && !neighbors.contains(&w[1]) {
                neighbors.push(w[1]);
            }
        }
    }
    neighbors.sort();
    Ok(neighbors)
}

pub(super) fn get_adjacent_vertices_to<V, HE, Q>(
    q: &Q,
    to: VertexIndex,
) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let he_indices = q.get_vertex_hyperedges(to)?;
    let mut predecessors: Vec<VertexIndex> = Vec::new();
    for he_idx in he_indices {
        let vertices = q.get_hyperedge_vertices(he_idx)?;
        for w in vertices.windows(2) {
            if w[1] == to && !predecessors.contains(&w[0]) {
                predecessors.push(w[0]);
            }
        }
    }
    predecessors.sort();
    Ok(predecessors)
}

#[allow(clippy::type_complexity)]
pub(super) fn get_full_adjacent_vertices_from<V, HE, Q>(
    q: &Q,
    from: VertexIndex,
) -> Result<Vec<(VertexIndex, Vec<HyperedgeIndex>)>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let he_indices = q.get_vertex_hyperedges(from)?;
    let mut map: AHashMap<VertexIndex, Vec<HyperedgeIndex>> = AHashMap::new();
    for he_idx in he_indices {
        let vertices = q.get_hyperedge_vertices(he_idx)?;
        for w in vertices.windows(2) {
            if w[0] == from {
                map.entry(w[1]).or_default().push(he_idx);
            }
        }
    }
    Ok(map.into_iter().collect())
}

#[allow(clippy::type_complexity)]
pub(super) fn get_full_adjacent_vertices_to<V, HE, Q>(
    q: &Q,
    to: VertexIndex,
) -> Result<Vec<(VertexIndex, Vec<HyperedgeIndex>)>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let he_indices = q.get_vertex_hyperedges(to)?;
    let mut map: AHashMap<VertexIndex, Vec<HyperedgeIndex>> = AHashMap::new();
    for he_idx in he_indices {
        let vertices = q.get_hyperedge_vertices(he_idx)?;
        for w in vertices.windows(2) {
            if w[1] == to {
                map.entry(w[0]).or_default().push(he_idx);
            }
        }
    }
    Ok(map.into_iter().collect())
}

pub(super) fn get_vertex_degree_in<V, HE, Q>(
    q: &Q,
    to: VertexIndex,
) -> Result<usize, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let he_indices = q.get_vertex_hyperedges(to)?;
    let mut count = 0;
    for he_idx in he_indices {
        let vertices = q.get_hyperedge_vertices(he_idx)?;
        for w in vertices.windows(2) {
            if w[1] == to {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub(super) fn get_vertex_degree_out<V, HE, Q>(
    q: &Q,
    from: VertexIndex,
) -> Result<usize, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let he_indices = q.get_vertex_hyperedges(from)?;
    let mut count = 0;
    for he_idx in he_indices {
        let vertices = q.get_hyperedge_vertices(he_idx)?;
        for w in vertices.windows(2) {
            if w[0] == from {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub(super) fn get_hyperedges_connecting<V, HE, Q>(
    q: &Q,
    from: VertexIndex,
    to: VertexIndex,
) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let he_indices = q.get_vertex_hyperedges(from)?;
    let mut result = Vec::new();
    for he_idx in he_indices {
        let vertices = q.get_hyperedge_vertices(he_idx)?;
        for w in vertices.windows(2) {
            if w[0] == from && w[1] == to {
                result.push(he_idx);
            }
        }
    }
    Ok(result)
}

pub(super) fn get_hyperedges_intersections<V, HE, Q>(
    q: &Q,
    hyperedges: &[HyperedgeIndex],
) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    if hyperedges.len() < 2 {
        return Err(HypergraphError::HyperedgesInvalidIntersections);
    }
    let vertex_sets: Vec<Vec<VertexIndex>> = hyperedges
        .iter()
        .map(|&he_idx| q.get_hyperedge_vertices(he_idx))
        .collect::<Result<Vec<_>, _>>()?;

    let mut result: Vec<VertexIndex> = vertex_sets[0]
        .iter()
        .filter(|v| vertex_sets[1..].iter().all(|s| s.contains(v)))
        .copied()
        .collect();
    result.sort();
    result.dedup();
    Ok(result)
}

pub(super) fn find_hyperedges_by_weight<V, HE, Q>(
    q: &Q,
    weight: HE,
) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let mut result = Vec::new();
    for idx in q.hyperedge_indices()? {
        if q.get_hyperedge_weight(idx)? == weight {
            result.push(idx);
        }
    }
    Ok(result)
}

pub(super) fn get_full_vertex_hyperedges<V, HE, Q>(
    q: &Q,
    v: VertexIndex,
) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    q.get_vertex_hyperedges(v)?
        .into_iter()
        .map(|he_idx| q.get_hyperedge_vertices(he_idx))
        .collect()
}

pub(super) fn contains_vertex<V, HE, Q>(q: &Q, weight: V) -> Result<bool, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    for idx in q.vertex_indices()? {
        if q.get_vertex_weight(idx)? == weight {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn get_vertex_neighborhood<V, HE, Q>(
    q: &Q,
    v: VertexIndex,
) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    // Validate the vertex exists.
    q.get_vertex_weight(v)?;

    let he_indices = q.get_vertex_hyperedges(v)?;
    let mut neighbors: Vec<VertexIndex> = Vec::new();
    for he_idx in he_indices {
        for co_member in q.get_hyperedge_vertices(he_idx)? {
            if co_member != v && !neighbors.contains(&co_member) {
                neighbors.push(co_member);
            }
        }
    }
    neighbors.sort();
    Ok(neighbors)
}

pub(super) fn get_vertex_index<V, HE, Q>(
    q: &Q,
    weight: V,
) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
    Q: HypergraphQuery<V, HE> + ?Sized,
{
    let mut result = Vec::new();
    for idx in q.vertex_indices()? {
        if q.get_vertex_weight(idx)? == weight {
            result.push(idx);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{
        Hypergraph,
        HypergraphQuery,
        VertexIndex,
        core::test_support::{
            E,
            W,
            build,
        },
    };

    #[test]
    fn get_adjacent_vertices_from_basic() {
        let (g, [v0, v1, _, _], _) = build();
        let adj = g.get_adjacent_vertices_from(v0).unwrap();
        assert_eq!(adj, vec![v1]);
    }

    #[test]
    fn get_adjacent_vertices_to_basic() {
        let (g, [v0, v1, _, _], _) = build();
        let adj = g.get_adjacent_vertices_to(v1).unwrap();
        assert_eq!(adj, vec![v0]);
    }

    #[test]
    fn get_full_adjacent_vertices_from_basic() {
        let (g, [v0, v1, _, _], _) = build();
        let full = g.get_full_adjacent_vertices_from(v0).unwrap();
        assert!(full.iter().any(|(v, _)| *v == v1));
    }

    #[test]
    fn get_full_adjacent_vertices_to_basic() {
        let (g, [v0, v1, _, _], _) = build();
        let full = g.get_full_adjacent_vertices_to(v1).unwrap();
        assert!(full.iter().any(|(v, _)| *v == v0));
    }

    #[test]
    fn get_vertex_degree_in_basic() {
        let (g, [_, v1, _, _], _) = build();
        assert_eq!(g.get_vertex_degree_in(v1).unwrap(), 1);
    }

    #[test]
    fn get_vertex_degree_out_basic() {
        let (g, [v0, _, _, _], _) = build();
        assert_eq!(g.get_vertex_degree_out(v0).unwrap(), 1);
    }

    #[test]
    fn get_hyperedges_connecting_basic() {
        let (g, [v0, v1, _, _], [e0, _, _]) = build();
        let connecting = g.get_hyperedges_connecting(v0, v1).unwrap();
        assert!(connecting.contains(&e0));
    }

    #[test]
    fn get_hyperedges_connecting_none() {
        let (g, [v0, _, v2, _], _) = build();
        let connecting = g.get_hyperedges_connecting(v0, v2).unwrap();
        assert!(connecting.is_empty());
    }

    #[test]
    fn get_hyperedges_intersections_basic() {
        let (g, [_, v1, _, _], [e0, e1, _]) = build();
        let inter = g.get_hyperedges_intersections(&[e0, e1]).unwrap();
        assert!(inter.contains(&v1));
    }

    #[test]
    fn get_hyperedges_intersections_too_few() {
        let (g, _, [e0, _, _]) = build();
        assert!(HypergraphQuery::get_hyperedges_intersections(&g, &[e0]).is_err());
    }

    #[test]
    fn find_hyperedges_by_weight_found() {
        let (g, _, _) = build();
        let found = HypergraphQuery::find_hyperedges_by_weight(&g, E(1)).unwrap();
        assert!(!found.is_empty());
    }

    #[test]
    fn find_hyperedges_by_weight_not_found() {
        let (g, _, _) = build();
        let found = HypergraphQuery::find_hyperedges_by_weight(&g, E(999)).unwrap();
        assert!(found.is_empty());
    }

    #[test]
    fn get_full_vertex_hyperedges_basic() {
        let (g, [_, v1, _, _], _) = build();
        let full = g.get_full_vertex_hyperedges(v1).unwrap();
        assert!(!full.is_empty());
    }

    #[test]
    fn contains_vertex_true() {
        let (g, _, _) = build();
        assert!(HypergraphQuery::contains_vertex(&g, W(0)).unwrap());
    }

    #[test]
    fn contains_vertex_false() {
        let (g, _, _) = build();
        assert!(!HypergraphQuery::contains_vertex(&g, W(99)).unwrap());
    }

    #[test]
    fn get_vertex_index_found() {
        let (g, [v0, _, _, _], _) = build();
        let indices = HypergraphQuery::get_vertex_index(&g, W(0)).unwrap();
        assert!(indices.contains(&v0));
    }

    #[test]
    fn get_vertex_index_not_found() {
        let (g, _, _) = build();
        let indices = HypergraphQuery::get_vertex_index(&g, W(99)).unwrap();
        assert!(indices.is_empty());
    }

    #[test]
    fn get_orphan_vertices_some() {
        let mut g: Hypergraph<W, E> = Hypergraph::new();
        let v0 = g.add_vertex(W(0)).unwrap();
        let v1 = g.add_vertex(W(1)).unwrap();
        g.add_hyperedge(vec![v0], E(1)).unwrap();
        assert_eq!(g.get_orphan_vertices().unwrap(), vec![v1]);
    }

    #[test]
    fn get_vertex_neighborhood_basic() {
        let (g, [v0, v1, v2, v3], _) = build();
        // v1 is in e0=[v0,v1], e1=[v1,v2], e2=[v1,v3]
        let mut nbrs = HypergraphQuery::get_vertex_neighborhood(&g, v1).unwrap();
        nbrs.sort();
        assert_eq!(nbrs, vec![v0, v2, v3]);
    }

    #[test]
    fn get_vertex_neighborhood_excludes_self() {
        let (g, [v0, _, _, _], _) = build();
        let nbrs = HypergraphQuery::get_vertex_neighborhood(&g, v0).unwrap();
        assert!(!nbrs.contains(&v0));
    }

    #[test]
    fn get_vertex_neighborhood_not_found() {
        let (g, _, _) = build();
        assert!(HypergraphQuery::get_vertex_neighborhood(&g, VertexIndex(999)).is_err());
    }
}
