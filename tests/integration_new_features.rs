//! Integration tests for features added in the modernization pass:
//! is_empty, Clone, vertices_iter, hyperedges_iter, IntoIterator for &Hypergraph,
//! BFS, DFS, is_reachable, topological_sort, get_dijkstra_connections_with_cost,
//! contains_vertex, get_vertex_index, is_acyclic, Display, connected_components,
//! get_dijkstra_from.

#![deny(unsafe_code, nonstandard_style)]
#![allow(missing_docs)]

mod common;

use common::{Hyperedge, Vertex};
use hypergraph::{HyperedgeIndex, Hypergraph, VertexIndex};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_graph<'a>() -> (
    Hypergraph<Vertex<'a>, Hyperedge<'a>>,
    VertexIndex,
    VertexIndex,
    VertexIndex,
    VertexIndex,
    HyperedgeIndex,
    HyperedgeIndex,
    HyperedgeIndex,
) {
    let mut g = Hypergraph::<Vertex, Hyperedge>::new();

    let a = g.add_vertex(Vertex::new("a")).unwrap();
    let b = g.add_vertex(Vertex::new("b")).unwrap();
    let c = g.add_vertex(Vertex::new("c")).unwrap();
    let d = g.add_vertex(Vertex::new("d")).unwrap();

    // a → b (cost 1), b → c (cost 2), a → c (cost 5)
    let ab = g
        .add_hyperedge(vec![a, b], Hyperedge::new("a-b", 1))
        .unwrap();
    let bc = g
        .add_hyperedge(vec![b, c], Hyperedge::new("b-c", 2))
        .unwrap();
    let ac = g
        .add_hyperedge(vec![a, c], Hyperedge::new("a-c", 5))
        .unwrap();

    (g, a, b, c, d, ab, bc, ac)
}

// ---------------------------------------------------------------------------
// is_empty
// ---------------------------------------------------------------------------

#[test]
fn is_empty_on_new_graph() {
    let g = Hypergraph::<Vertex, Hyperedge>::new();
    assert!(g.is_empty());
}

#[test]
fn is_empty_after_adding_vertex() {
    let mut g = Hypergraph::<Vertex, Hyperedge>::new();
    g.add_vertex(Vertex::new("x")).unwrap();
    assert!(!g.is_empty());
}

#[test]
fn is_empty_after_clear() {
    let (mut g, ..) = build_graph();
    g.clear();
    assert!(g.is_empty());
}

// ---------------------------------------------------------------------------
// Clone
// ---------------------------------------------------------------------------

#[test]
fn clone_produces_independent_copy() {
    let (g, a, _, _, _, _, _, _) = build_graph();
    let mut g2 = g.clone();

    assert_eq!(g.count_vertices(), g2.count_vertices());
    assert_eq!(g.count_hyperedges(), g2.count_hyperedges());

    // Mutating the clone must not affect the original.
    g2.remove_vertex(a).unwrap();
    assert_eq!(g.count_vertices(), 4);
    assert_eq!(g2.count_vertices(), 3);
}

// ---------------------------------------------------------------------------
// vertices_iter / hyperedges_iter
// ---------------------------------------------------------------------------

#[test]
fn vertices_iter_yields_all() {
    let (g, a, b, c, d, ..) = build_graph();
    let mut collected: Vec<VertexIndex> = g.vertices_iter().map(|(idx, _)| idx).collect();
    collected.sort();
    assert_eq!(collected, vec![a, b, c, d]);
}

#[test]
fn hyperedges_iter_yields_all() {
    let (g, _, _, _, _, ab, bc, ac) = build_graph();
    let mut collected: Vec<HyperedgeIndex> = g.hyperedges_iter().map(|(idx, _)| idx).collect();
    collected.sort();
    assert_eq!(collected, vec![ab, bc, ac]);
}

// ---------------------------------------------------------------------------
// IntoIterator for &Hypergraph
// ---------------------------------------------------------------------------

#[test]
fn ref_into_iterator() {
    let (g, ..) = build_graph();
    let count = (&g).into_iter().count();
    assert_eq!(count, g.count_hyperedges());
}

// ---------------------------------------------------------------------------
// BFS
// ---------------------------------------------------------------------------

#[test]
fn bfs_from_source() {
    let (g, a, b, c, _, ..) = build_graph();
    let bfs = g.get_bfs(a).unwrap();
    // a must come first; b and c must both appear.
    assert_eq!(bfs[0], a);
    assert!(bfs.contains(&b));
    assert!(bfs.contains(&c));
}

#[test]
fn bfs_isolated_vertex() {
    let (g, _, _, _, d, ..) = build_graph();
    // d has no outgoing edges in the test graph.
    assert_eq!(g.get_bfs(d).unwrap(), vec![d]);
}

#[test]
fn bfs_invalid_vertex() {
    let (g, ..) = build_graph();
    assert!(g.get_bfs(VertexIndex(999)).is_err());
}

// ---------------------------------------------------------------------------
// DFS
// ---------------------------------------------------------------------------

#[test]
fn dfs_from_source() {
    let (g, a, b, c, _, ..) = build_graph();
    let dfs = g.get_dfs(a).unwrap();
    assert_eq!(dfs[0], a);
    assert!(dfs.contains(&b));
    assert!(dfs.contains(&c));
}

#[test]
fn dfs_isolated_vertex() {
    let (g, _, _, _, d, ..) = build_graph();
    assert_eq!(g.get_dfs(d).unwrap(), vec![d]);
}

#[test]
fn dfs_invalid_vertex() {
    let (g, ..) = build_graph();
    assert!(g.get_dfs(VertexIndex(999)).is_err());
}

// ---------------------------------------------------------------------------
// is_reachable
// ---------------------------------------------------------------------------

#[test]
fn is_reachable_self() {
    let (g, a, ..) = build_graph();
    assert_eq!(g.is_reachable(a, a).unwrap(), true);
}

#[test]
fn is_reachable_direct_edge() {
    let (g, a, b, ..) = build_graph();
    assert_eq!(g.is_reachable(a, b).unwrap(), true);
}

#[test]
fn is_reachable_transitive() {
    let (g, a, _, c, ..) = build_graph();
    assert_eq!(g.is_reachable(a, c).unwrap(), true);
}

#[test]
fn is_reachable_false() {
    let (g, a, _, _, d, ..) = build_graph();
    // d has no incoming edges, only a has outgoing ones.
    assert_eq!(g.is_reachable(d, a).unwrap(), false);
}

#[test]
fn is_reachable_invalid_vertex() {
    let (g, a, ..) = build_graph();
    assert!(g.is_reachable(a, VertexIndex(999)).is_err());
    assert!(g.is_reachable(VertexIndex(999), a).is_err());
}

// ---------------------------------------------------------------------------
// topological_sort
// ---------------------------------------------------------------------------

#[test]
fn topological_sort_dag() {
    let (g, a, b, c, d, ..) = build_graph();
    let order = g.topological_sort().unwrap();

    // In a valid topological order a must appear before b and c.
    let pos = |v: VertexIndex| order.iter().position(|&x| x == v).unwrap();

    assert!(pos(a) < pos(b), "a must precede b");
    assert!(pos(a) < pos(c), "a must precede c");
    assert!(pos(b) < pos(c), "b must precede c");
    // d is isolated — just verify it appears somewhere.
    assert!(order.contains(&d));
    assert_eq!(order.len(), 4);
}

#[test]
fn topological_sort_cyclic() {
    let mut g = Hypergraph::<Vertex, Hyperedge>::new();
    let a = g.add_vertex(Vertex::new("a")).unwrap();
    let b = g.add_vertex(Vertex::new("b")).unwrap();
    g.add_hyperedge(vec![a, b], Hyperedge::new("a-b", 1)).unwrap();
    g.add_hyperedge(vec![b, a], Hyperedge::new("b-a", 1)).unwrap();

    assert!(g.topological_sort().is_err());
}

// ---------------------------------------------------------------------------
// get_dijkstra_connections_with_cost
// ---------------------------------------------------------------------------

#[test]
fn dijkstra_with_cost_finds_shortest() {
    let (g, a, b, c, _, ab, bc, _ac) = build_graph();
    // Shortest a→c is via a→b→c with cost 3, not direct a→c with cost 5.
    let (cost, path) = g.get_dijkstra_connections_with_cost(a, c).unwrap();
    assert_eq!(cost, 3);
    assert_eq!(
        path,
        vec![(a, None), (b, Some(ab)), (c, Some(bc))]
    );
}

#[test]
fn dijkstra_with_cost_matches_without_cost() {
    let (g, a, _, c, ..) = build_graph();
    let (_, path_with) = g.get_dijkstra_connections_with_cost(a, c).unwrap();
    let path_without = g.get_dijkstra_connections(a, c).unwrap();
    assert_eq!(path_with, path_without);
}

#[test]
fn dijkstra_with_cost_no_path() {
    let (g, _, _, _, d, ..) = build_graph();
    let a = VertexIndex(0);
    // d has no outgoing edges, so d→a has no path.
    let (cost, path) = g.get_dijkstra_connections_with_cost(d, a).unwrap();
    assert_eq!(cost, 0);
    assert!(path.is_empty());
}

// ---------------------------------------------------------------------------
// contains_vertex / get_vertex_index
// ---------------------------------------------------------------------------

#[test]
fn contains_vertex_present() {
    let (g, ..) = build_graph();
    assert!(g.contains_vertex(Vertex::new("a")));
    assert!(g.contains_vertex(Vertex::new("b")));
}

#[test]
fn contains_vertex_absent() {
    let (g, ..) = build_graph();
    assert!(!g.contains_vertex(Vertex::new("z")));
}

#[test]
fn get_vertex_index_roundtrip() {
    let (g, a, b, c, d, ..) = build_graph();
    assert_eq!(g.get_vertex_index(Vertex::new("a")), Some(a));
    assert_eq!(g.get_vertex_index(Vertex::new("b")), Some(b));
    assert_eq!(g.get_vertex_index(Vertex::new("c")), Some(c));
    assert_eq!(g.get_vertex_index(Vertex::new("d")), Some(d));
}

#[test]
fn get_vertex_index_absent() {
    let (g, ..) = build_graph();
    assert_eq!(g.get_vertex_index(Vertex::new("z")), None);
}

#[test]
fn get_vertex_index_after_remove() {
    let (mut g, a, ..) = build_graph();
    g.remove_vertex(a).unwrap();
    assert_eq!(g.get_vertex_index(Vertex::new("a")), None);
}

// ---------------------------------------------------------------------------
// is_acyclic
// ---------------------------------------------------------------------------

#[test]
fn is_acyclic_dag() {
    let (g, ..) = build_graph();
    assert!(g.is_acyclic());
}

#[test]
fn is_acyclic_with_cycle() {
    let mut g = Hypergraph::<Vertex, Hyperedge>::new();
    let a = g.add_vertex(Vertex::new("a")).unwrap();
    let b = g.add_vertex(Vertex::new("b")).unwrap();
    g.add_hyperedge(vec![a, b], Hyperedge::new("a-b", 1)).unwrap();
    g.add_hyperedge(vec![b, a], Hyperedge::new("b-a", 1)).unwrap();
    assert!(!g.is_acyclic());
}

#[test]
fn is_acyclic_empty() {
    let g = Hypergraph::<Vertex, Hyperedge>::new();
    assert!(g.is_acyclic());
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

#[test]
fn display_contains_vertex_weights() {
    let (g, ..) = build_graph();
    let s = g.to_string();
    assert!(s.contains("a"), "missing vertex a in: {s}");
    assert!(s.contains("b"), "missing vertex b in: {s}");
    assert!(s.contains("c"), "missing vertex c in: {s}");
    assert!(s.contains("d"), "missing vertex d in: {s}");
}

#[test]
fn display_contains_hyperedge_weights() {
    let (g, ..) = build_graph();
    let s = g.to_string();
    assert!(s.contains("a-b"), "missing hyperedge a-b in: {s}");
    assert!(s.contains("b-c"), "missing hyperedge b-c in: {s}");
}

#[test]
fn display_empty_graph() {
    let g = Hypergraph::<Vertex, Hyperedge>::new();
    let s = g.to_string();
    assert!(s.contains("vertices: []"), "unexpected: {s}");
    assert!(s.contains("hyperedges: []"), "unexpected: {s}");
}

// ---------------------------------------------------------------------------
// connected_components
// ---------------------------------------------------------------------------

#[test]
fn connected_components_single() {
    // a → b → c, all in one weakly-connected component; d is isolated.
    let (g, a, b, c, d, ..) = build_graph();
    let mut components = g.connected_components().unwrap();
    // Sort for determinism (already sorted by implementation, but be explicit).
    for comp in &mut components {
        comp.sort();
    }
    components.sort_by_key(|c| c[0]);

    assert_eq!(components.len(), 2, "expected 2 components");
    // The connected component contains a, b, c.
    assert!(components[0].contains(&a));
    assert!(components[0].contains(&b));
    assert!(components[0].contains(&c));
    // d is isolated.
    assert_eq!(components[1], vec![d]);
}

#[test]
fn connected_components_all_isolated() {
    let mut g = Hypergraph::<Vertex, Hyperedge>::new();
    let a = g.add_vertex(Vertex::new("a")).unwrap();
    let b = g.add_vertex(Vertex::new("b")).unwrap();
    let components = g.connected_components().unwrap();
    assert_eq!(components.len(), 2);
    assert!(components.iter().any(|c| c == &vec![a]));
    assert!(components.iter().any(|c| c == &vec![b]));
}

#[test]
fn connected_components_empty_graph() {
    let g = Hypergraph::<Vertex, Hyperedge>::new();
    assert_eq!(g.connected_components().unwrap(), Vec::<Vec<VertexIndex>>::new());
}

// ---------------------------------------------------------------------------
// get_dijkstra_from
// ---------------------------------------------------------------------------

#[test]
fn dijkstra_from_includes_source() {
    let (g, a, ..) = build_graph();
    let dists = g.get_dijkstra_from(a).unwrap();
    assert_eq!(dists[&a], 0);
}

#[test]
fn dijkstra_from_correct_distances() {
    let (g, a, b, c, ..) = build_graph();
    // a→b cost 1, a→c cost 5, b→c cost 2 (so a→b→c = 3 < 5)
    let dists = g.get_dijkstra_from(a).unwrap();
    assert_eq!(dists[&a], 0);
    assert_eq!(dists[&b], 1);
    assert_eq!(dists[&c], 3);
}

#[test]
fn dijkstra_from_excludes_unreachable() {
    let (g, _, _, _, d, ..) = build_graph();
    // d has no outgoing edges.
    let dists = g.get_dijkstra_from(d).unwrap();
    assert_eq!(dists.len(), 1);
    assert_eq!(dists[&d], 0);
}

#[test]
fn dijkstra_from_consistent_with_point_to_point() {
    let (g, a, _, c, ..) = build_graph();
    let (cost, _) = g.get_dijkstra_connections_with_cost(a, c).unwrap();
    let dists = g.get_dijkstra_from(a).unwrap();
    assert_eq!(dists[&c], cost);
}

#[test]
fn dijkstra_from_invalid_vertex() {
    let (g, ..) = build_graph();
    assert!(g.get_dijkstra_from(VertexIndex(999)).is_err());
}
