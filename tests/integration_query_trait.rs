//! Integration tests for `HypergraphQuery` — exercises every trait method via
//! explicit trait dispatch so the implementations in `hypergraph_impl` and
//! `persistent_impl` are both covered (the latter through feature-gating).

#![allow(clippy::many_single_char_names)]

mod common;

use common::{
    Hyperedge,
    Vertex,
};
use hypergraph::{
    HyperedgeIndex,
    Hypergraph,
    HypergraphQuery,
    VertexIndex,
};

/// Builds a small acyclic directed hypergraph for most tests:
///
/// ```text
///   a --[e0,cost=1]--> b --[e1,cost=2]--> c
///                      |
///                      [e2,cost=3]
///                      |
///                      v
///                      d
/// ```
///
/// Vertex indices are in insertion order (0=a, 1=b, 2=c, 3=d).
/// Hyperedge indices: e0=0, e1=1, e2=2.
fn build_acyclic() -> (
    Hypergraph<Vertex<'static>, Hyperedge<'static>>,
    [VertexIndex; 4],
    [HyperedgeIndex; 3],
) {
    let mut g = Hypergraph::<Vertex, Hyperedge>::new();
    let a = g.add_vertex(Vertex::new("a")).unwrap();
    let b = g.add_vertex(Vertex::new("b")).unwrap();
    let c = g.add_vertex(Vertex::new("c")).unwrap();
    let d = g.add_vertex(Vertex::new("d")).unwrap();
    let e0 = g
        .add_hyperedge(vec![a, b], Hyperedge::new("e0", 1))
        .unwrap();
    let e1 = g
        .add_hyperedge(vec![b, c], Hyperedge::new("e1", 2))
        .unwrap();
    let e2 = g
        .add_hyperedge(vec![b, d], Hyperedge::new("e2", 3))
        .unwrap();
    (g, [a, b, c, d], [e0, e1, e2])
}

/// Builds a cyclic graph: a→b→c→a, with d isolated (but present).
fn build_cyclic() -> (
    Hypergraph<Vertex<'static>, Hyperedge<'static>>,
    [VertexIndex; 4],
    [HyperedgeIndex; 3],
) {
    let mut g = Hypergraph::<Vertex, Hyperedge>::new();
    let a = g.add_vertex(Vertex::new("a")).unwrap();
    let b = g.add_vertex(Vertex::new("b")).unwrap();
    let c = g.add_vertex(Vertex::new("c")).unwrap();
    let d = g.add_vertex(Vertex::new("d")).unwrap();
    let e0 = g
        .add_hyperedge(vec![a, b], Hyperedge::new("e0", 1))
        .unwrap();
    let e1 = g
        .add_hyperedge(vec![b, c], Hyperedge::new("e1", 1))
        .unwrap();
    let e2 = g
        .add_hyperedge(vec![c, a], Hyperedge::new("e2", 1))
        .unwrap();
    (g, [a, b, c, d], [e0, e1, e2])
}

#[test]
fn query_count_vertices() {
    let (g, _, _) = build_acyclic();
    assert_eq!(HypergraphQuery::count_vertices(&g), 4);
}

#[test]
fn query_count_hyperedges() {
    let (g, _, _) = build_acyclic();
    assert_eq!(HypergraphQuery::count_hyperedges(&g), 3);
}

#[test]
fn query_is_empty_false() {
    let (g, _, _) = build_acyclic();
    assert!(!HypergraphQuery::is_empty(&g));
}

#[test]
fn query_is_empty_true() {
    let g = Hypergraph::<Vertex, Hyperedge>::new();
    assert!(HypergraphQuery::is_empty(&g));
}

#[test]
fn query_vertex_indices() {
    let (g, [a, b, c, d], _) = build_acyclic();
    let mut got = HypergraphQuery::vertex_indices(&g).unwrap();
    got.sort();
    assert_eq!(got, vec![a, b, c, d]);
}

#[test]
fn query_hyperedge_indices() {
    let (g, _, [e0, e1, e2]) = build_acyclic();
    let mut got = HypergraphQuery::hyperedge_indices(&g).unwrap();
    got.sort();
    assert_eq!(got, vec![e0, e1, e2]);
}

#[test]
fn query_get_vertex_weight() {
    let (g, [a, _, _, _], _) = build_acyclic();
    assert_eq!(
        HypergraphQuery::get_vertex_weight(&g, a).unwrap(),
        Vertex::new("a")
    );
}

#[test]
fn query_get_vertex_weight_not_found() {
    let (g, _, _) = build_acyclic();
    let missing = VertexIndex(999);
    assert!(HypergraphQuery::get_vertex_weight(&g, missing).is_err());
}

#[test]
fn query_get_hyperedge_weight() {
    let (g, _, [e0, _, _]) = build_acyclic();
    assert_eq!(
        HypergraphQuery::get_hyperedge_weight(&g, e0).unwrap(),
        Hyperedge::new("e0", 1)
    );
}

#[test]
fn query_get_hyperedge_weight_not_found() {
    let (g, _, _) = build_acyclic();
    let missing = HyperedgeIndex(999);
    assert!(HypergraphQuery::get_hyperedge_weight(&g, missing).is_err());
}

#[test]
fn query_get_vertex_hyperedges() {
    let (g, [_, b, _, _], [e0, e1, e2]) = build_acyclic();
    let mut got = HypergraphQuery::get_vertex_hyperedges(&g, b).unwrap();
    got.sort();
    assert_eq!(got, vec![e0, e1, e2]);
}

#[test]
fn query_get_hyperedge_vertices() {
    let (g, [a, b, _, _], [e0, _, _]) = build_acyclic();
    assert_eq!(
        HypergraphQuery::get_hyperedge_vertices(&g, e0).unwrap(),
        vec![a, b]
    );
}

#[test]
fn query_get_adjacent_vertices_from() {
    let (g, [_, b, c, d], _) = build_acyclic();
    let mut got = HypergraphQuery::get_adjacent_vertices_from(&g, b).unwrap();
    got.sort();
    assert_eq!(got, vec![c, d]);
}

#[test]
fn query_get_adjacent_vertices_to() {
    let (g, [a, b, _, _], _) = build_acyclic();
    let got = HypergraphQuery::get_adjacent_vertices_to(&g, b).unwrap();
    assert_eq!(got, vec![a]);
}

#[test]
fn query_get_full_adjacent_vertices_from() {
    let (g, [a, b, _, _], [e0, _, _]) = build_acyclic();
    let got = HypergraphQuery::get_full_adjacent_vertices_from(&g, a).unwrap();
    assert_eq!(got.len(), 1);
    let (neighbor, hes) = &got[0];
    assert_eq!(*neighbor, b);
    assert_eq!(hes, &[e0]);
}

#[test]
fn query_get_full_adjacent_vertices_to() {
    let (g, [a, b, _, _], [e0, _, _]) = build_acyclic();
    let got = HypergraphQuery::get_full_adjacent_vertices_to(&g, b).unwrap();
    assert_eq!(got.len(), 1);
    let (predecessor, hes) = &got[0];
    assert_eq!(*predecessor, a);
    assert_eq!(hes, &[e0]);
}

#[test]
fn query_get_vertex_degree_in() {
    let (g, [_, b, _, _], _) = build_acyclic();
    assert_eq!(HypergraphQuery::get_vertex_degree_in(&g, b).unwrap(), 1);
}

#[test]
fn query_get_vertex_degree_out() {
    let (g, [_, b, _, _], _) = build_acyclic();
    assert_eq!(HypergraphQuery::get_vertex_degree_out(&g, b).unwrap(), 2);
}

#[test]
fn query_get_hyperedges_connecting() {
    let (g, [a, b, _, _], [e0, _, _]) = build_acyclic();
    assert_eq!(
        HypergraphQuery::get_hyperedges_connecting(&g, a, b).unwrap(),
        vec![e0]
    );
}

#[test]
fn query_get_hyperedges_connecting_none() {
    let (g, [a, _, c, _], _) = build_acyclic();
    assert_eq!(
        HypergraphQuery::get_hyperedges_connecting(&g, a, c).unwrap(),
        vec![]
    );
}

#[test]
fn query_get_hyperedges_intersections() {
    let (g, [_, b, _, _], [e0, e1, e2]) = build_acyclic();
    let mut got = HypergraphQuery::get_hyperedges_intersections(&g, &[e0, e1, e2]).unwrap();
    got.sort();
    assert_eq!(got, vec![b]);
}

#[test]
fn query_get_hyperedges_intersections_too_few() {
    let (g, _, [e0, _, _]) = build_acyclic();
    assert!(HypergraphQuery::get_hyperedges_intersections(&g, &[e0]).is_err());
}

#[test]
fn query_find_hyperedges_by_weight() {
    let (g, _, [e0, _, _]) = build_acyclic();
    assert_eq!(
        HypergraphQuery::find_hyperedges_by_weight(&g, Hyperedge::new("e0", 1)).unwrap(),
        vec![e0]
    );
}

#[test]
fn query_get_full_vertex_hyperedges() {
    let (g, [a, b, _, _], _) = build_acyclic();
    let got = HypergraphQuery::get_full_vertex_hyperedges(&g, a).unwrap();
    assert_eq!(got, vec![vec![a, b]]);
}

#[test]
fn query_contains_vertex_true() {
    let (g, _, _) = build_acyclic();
    assert!(HypergraphQuery::contains_vertex(&g, Vertex::new("a")).unwrap());
}

#[test]
fn query_contains_vertex_false() {
    let (g, _, _) = build_acyclic();
    assert!(!HypergraphQuery::contains_vertex(&g, Vertex::new("z")).unwrap());
}

#[test]
fn query_get_vertex_index() {
    let (g, [a, _, _, _], _) = build_acyclic();
    assert_eq!(
        HypergraphQuery::get_vertex_index(&g, Vertex::new("a")).unwrap(),
        vec![a]
    );
}

#[test]
fn query_get_bfs() {
    let (g, [a, b, c, d], _) = build_acyclic();
    let got = HypergraphQuery::get_bfs(&g, a).unwrap();
    assert_eq!(got[0], a);
    assert_eq!(got[1], b);
    let rest: Vec<_> = got[2..].to_vec();
    assert!(rest.contains(&c));
    assert!(rest.contains(&d));
}

#[test]
fn query_get_dfs() {
    let (g, [a, b, _, _], _) = build_acyclic();
    let got = HypergraphQuery::get_dfs(&g, a).unwrap();
    assert_eq!(got[0], a);
    assert_eq!(got[1], b);
}

#[test]
fn query_is_reachable_true() {
    let (g, [a, _, c, _], _) = build_acyclic();
    assert!(HypergraphQuery::is_reachable(&g, a, c).unwrap());
}

#[test]
fn query_is_reachable_self() {
    let (g, [a, _, _, _], _) = build_acyclic();
    assert!(HypergraphQuery::is_reachable(&g, a, a).unwrap());
}

#[test]
fn query_is_reachable_false() {
    let (g, [a, _, _, d], _) = build_acyclic();
    // d has no outgoing edges — a is not reachable from d
    assert!(!HypergraphQuery::is_reachable(&g, d, a).unwrap());
}

#[test]
fn query_is_acyclic_true() {
    let (g, _, _) = build_acyclic();
    assert!(HypergraphQuery::is_acyclic(&g));
}

#[test]
fn query_is_acyclic_false() {
    let (g, _, _) = build_cyclic();
    assert!(!HypergraphQuery::is_acyclic(&g));
}

#[test]
fn query_get_all_paths() {
    let (g, [a, _, c, _], _) = build_acyclic();
    let got = HypergraphQuery::get_all_paths(&g, a, c).unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0][0], a);
    assert_eq!(*got[0].last().unwrap(), c);
}

#[test]
fn query_get_all_paths_same_vertex() {
    let (g, [a, _, _, _], _) = build_acyclic();
    assert_eq!(
        HypergraphQuery::get_all_paths(&g, a, a).unwrap(),
        vec![vec![a]]
    );
}

#[test]
fn query_topological_sort_acyclic() {
    let (g, [a, b, _, _], _) = build_acyclic();
    let order = HypergraphQuery::topological_sort(&g).unwrap();
    let pos_a = order.iter().position(|&v| v == a).unwrap();
    let pos_b = order.iter().position(|&v| v == b).unwrap();
    assert!(pos_a < pos_b, "a must come before b in topological order");
}

#[test]
fn query_topological_sort_cyclic_errors() {
    let (g, _, _) = build_cyclic();
    assert!(HypergraphQuery::topological_sort(&g).is_err());
}

#[test]
fn query_strongly_connected_components() {
    let (g, [a, b, c, d], _) = build_cyclic();
    let sccs = HypergraphQuery::strongly_connected_components(&g).unwrap();
    let big_scc: Vec<_> = {
        let mut v = vec![a, b, c];
        v.sort();
        v
    };
    let small_scc = vec![d];
    assert!(sccs.contains(&big_scc));
    assert!(sccs.contains(&small_scc));
}

#[test]
fn query_connected_components() {
    // Build a graph with two disconnected components.
    let mut g = Hypergraph::<Vertex, Hyperedge>::new();
    let a = g.add_vertex(Vertex::new("a")).unwrap();
    let b = g.add_vertex(Vertex::new("b")).unwrap();
    let c = g.add_vertex(Vertex::new("c")).unwrap();
    let d = g.add_vertex(Vertex::new("d")).unwrap();
    g.add_hyperedge(vec![a, b], Hyperedge::new("e0", 1))
        .unwrap();
    g.add_hyperedge(vec![c, d], Hyperedge::new("e1", 1))
        .unwrap();

    let components = HypergraphQuery::connected_components(&g).unwrap();
    assert_eq!(components.len(), 2);
    let first: Vec<_> = {
        let mut v = vec![a, b];
        v.sort();
        v
    };
    let second: Vec<_> = {
        let mut v = vec![c, d];
        v.sort();
        v
    };
    assert!(components.contains(&first));
    assert!(components.contains(&second));
}

#[test]
fn query_get_dijkstra_connections() {
    let (g, [a, b, c, _], [e0, e1, _]) = build_acyclic();
    let path = HypergraphQuery::get_dijkstra_connections(&g, a, c).unwrap();
    assert_eq!(path, vec![(a, None), (b, Some(e0)), (c, Some(e1))]);
}

#[test]
fn query_get_dijkstra_connections_with_cost() {
    let (g, [a, _, c, _], _) = build_acyclic();
    let (cost, path) = HypergraphQuery::get_dijkstra_connections_with_cost(&g, a, c).unwrap();
    assert_eq!(cost, 3); // e0(1) + e1(2)
    assert_eq!(path[0].0, a);
    assert_eq!(path.last().unwrap().0, c);
}

#[test]
fn query_get_dijkstra_from() {
    let (g, [a, b, c, d], _) = build_acyclic();
    let distances = HypergraphQuery::get_dijkstra_from(&g, a).unwrap();
    assert_eq!(distances[&a], 0);
    assert_eq!(distances[&b], 1);
    assert_eq!(distances[&c], 3); // a→b(1) + b→c(2)
    assert_eq!(distances[&d], 4); // a→b(1) + b→d(3)
}

#[test]
fn query_get_orphan_vertices_none() {
    let (g, _, _) = build_acyclic();
    assert!(HypergraphQuery::get_orphan_vertices(&g).unwrap().is_empty());
}

#[test]
fn query_get_orphan_vertices_some() {
    let mut g = Hypergraph::<Vertex, Hyperedge>::new();
    let a = g.add_vertex(Vertex::new("a")).unwrap();
    let b = g.add_vertex(Vertex::new("b")).unwrap();
    g.add_hyperedge(vec![a], Hyperedge::new("e0", 1)).unwrap();
    let orphans = HypergraphQuery::get_orphan_vertices(&g).unwrap();
    assert_eq!(orphans, vec![b]);
}

#[test]
fn query_get_orphan_hyperedges_none() {
    let (g, _, _) = build_acyclic();
    assert!(
        HypergraphQuery::get_orphan_hyperedges(&g)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn query_get_endpoints() {
    let (g, [a, _, c, d], _) = build_acyclic();
    let (sources, sinks) = HypergraphQuery::get_endpoints(&g).unwrap();
    assert_eq!(sources, vec![a]);
    let mut sinks_sorted = sinks;
    sinks_sorted.sort();
    assert_eq!(sinks_sorted, vec![c, d]);
}

#[test]
fn query_get_inclusions_none() {
    let (g, _, _) = build_acyclic();
    assert!(HypergraphQuery::get_inclusions(&g).unwrap().is_empty());
}

#[test]
fn query_get_inclusions_some() {
    let mut g = Hypergraph::<Vertex, Hyperedge>::new();
    let a = g.add_vertex(Vertex::new("a")).unwrap();
    let b = g.add_vertex(Vertex::new("b")).unwrap();
    let c = g.add_vertex(Vertex::new("c")).unwrap();
    let big = g
        .add_hyperedge(vec![a, b, c], Hyperedge::new("big", 1))
        .unwrap();
    let small = g
        .add_hyperedge(vec![a, b], Hyperedge::new("small", 1))
        .unwrap();
    let pairs = HypergraphQuery::get_inclusions(&g).unwrap();
    assert!(pairs.contains(&(small, big)));
    assert!(!pairs.contains(&(big, small)));
}

#[test]
fn query_is_k_uniform_true() {
    let (g, _, _) = build_acyclic();
    assert!(HypergraphQuery::is_k_uniform(&g, 2).unwrap());
}

#[test]
fn query_is_k_uniform_false() {
    let (g, _, _) = build_acyclic();
    assert!(!HypergraphQuery::is_k_uniform(&g, 3).unwrap());
}

#[test]
fn query_find_cut_vertices_present() {
    let (g, [_, b, _, _], _) = build_acyclic();
    let cuts = HypergraphQuery::find_cut_vertices(&g).unwrap();
    assert!(cuts.contains(&b));
}

#[test]
fn query_find_cut_vertices_none() {
    let (g, _, _) = build_cyclic();
    // Triangle a→b→c→a: no articulation points.
    // Isolated vertex d is also not a cut vertex.
    let cuts = HypergraphQuery::find_cut_vertices(&g).unwrap();
    assert!(cuts.is_empty());
}

#[test]
fn query_get_core_all() {
    let (g, [a, b, c, d], [e0, e1, e2]) = build_acyclic();
    let (vs, hes) = HypergraphQuery::get_core(&g, 1, 1).unwrap();
    assert_eq!(vs, vec![a, b, c, d]);
    assert_eq!(hes, vec![e0, e1, e2]);
}

#[test]
fn query_get_core_empty() {
    let (g, _, _) = build_acyclic();
    // min_edge_size=2: after a,c,d are removed (degree 1<2) the edges shrink
    // to size 1, then b loses all edges and is removed too.
    let (vs, hes) = HypergraphQuery::get_core(&g, 2, 2).unwrap();
    assert!(vs.is_empty());
    assert!(hes.is_empty());
}

#[test]
fn query_expand_to_graph() {
    let (g, [a, b, c, d], _) = build_acyclic();
    let pairs = HypergraphQuery::expand_to_graph(&g).unwrap();
    assert!(pairs.contains(&(a, b)));
    assert!(pairs.contains(&(b, c)));
    assert!(pairs.contains(&(b, d)));
    assert_eq!(pairs.len(), 3);
}

#[test]
fn query_expand_to_star() {
    let (g, [a, b, c, d], [e0, e1, e2]) = build_acyclic();
    let pairs = HypergraphQuery::expand_to_star(&g).unwrap();
    assert!(pairs.contains(&(a, e0)));
    assert!(pairs.contains(&(b, e0)));
    assert!(pairs.contains(&(b, e1)));
    assert!(pairs.contains(&(c, e1)));
    assert!(pairs.contains(&(b, e2)));
    assert!(pairs.contains(&(d, e2)));
    assert_eq!(pairs.len(), 6);
}

#[test]
fn query_compute_page_rank() {
    let (g, [a, b, c, d], _) = build_acyclic();
    let ranks = HypergraphQuery::compute_page_rank(&g, 0.85, 100).unwrap();
    assert!(ranks.contains_key(&a));
    assert!(ranks.contains_key(&b));
    assert!(ranks.contains_key(&c));
    assert!(ranks.contains_key(&d));
    assert!(ranks[&b] > ranks[&a], "b is more central than a");
    let total: f64 = ranks.values().sum();
    assert!((total - 1.0).abs() < 0.01, "ranks should sum to ~1.0");
}
