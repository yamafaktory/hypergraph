#![deny(unsafe_code, nonstandard_style)]
#![forbid(rust_2021_compatibility)]

mod common;

use common::{HyperEdge, Vertex};
use hypergraph::{errors::HypergraphError, HyperedgeIndex, Hypergraph, VertexIndex};

#[test]
fn integration() {
    // Create a new hypergraph.
    let mut graph = Hypergraph::<Vertex, HyperEdge>::new();

    // Create some vertice weights.
    let vertex_one = Vertex::new("one", 1);
    let vertex_two = Vertex::new("two", 1);
    let vertex_three = Vertex::new("three", 1);
    let vertex_four = Vertex::new("four", 1);
    let vertex_five = Vertex::new("five", 1);

    // Create some hyperedge weights.
    let hyperedge_one = HyperEdge::new("one", 10);
    let hyperedge_two = HyperEdge::new("two", 20);
    let hyperedge_three = HyperEdge::new("three", 1);

    // Create some vertices.
    let a = graph.add_vertex(vertex_one).unwrap();
    let b = graph.add_vertex(vertex_two).unwrap();
    let c = graph.add_vertex(vertex_three).unwrap();
    let d = graph.add_vertex(vertex_four).unwrap();
    let e = graph.add_vertex(vertex_five).unwrap();

    // Create some hyperedges.
    let alpha = graph.add_hyperedge(vec![a, b, e], hyperedge_one).unwrap();
    let beta = graph
        .add_hyperedge(vec![a, b, e, d], hyperedge_two)
        .unwrap();
    let gamma = graph.add_hyperedge(vec![b, c, e], hyperedge_three).unwrap();

    // Get some paths via Dijkstra.
    assert_eq!(
        graph.get_dijkstra_connections(a, e),
        Ok(vec![(alpha, a)]),
        "should get a path of three vertices"
    );
}
