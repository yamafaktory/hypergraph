#![deny(unsafe_code, nonstandard_style)]
#![forbid(rust_2021_compatibility)]

mod common;

use common::{Hyperedge, Vertex};
use hypergraph::Hypergraph;

#[test]
fn integration_iterator() {
    // Create a new hypergraph.
    let mut graph = Hypergraph::<Vertex, Hyperedge>::new();

    // Create some vertice weights.
    let vertex_one = Vertex::new("one");
    let vertex_two = Vertex::new("two");
    let vertex_three = Vertex::new("three");
    let vertex_four = Vertex::new("four");
    let vertex_five = Vertex::new("five");

    // Create some hyperedge weights.
    let hyperedge_one = Hyperedge::new("one", 10);
    let hyperedge_two = Hyperedge::new("two", 20);
    let hyperedge_three = Hyperedge::new("three", 1);
    let hyperedge_four = Hyperedge::new("four", 100);

    // Create some vertices.
    let a = graph.add_vertex(vertex_one).unwrap();
    let b = graph.add_vertex(vertex_two).unwrap();
    let c = graph.add_vertex(vertex_three).unwrap();
    let d = graph.add_vertex(vertex_four).unwrap();
    let e = graph.add_vertex(vertex_five).unwrap();

    // Add some hyperedges.
    graph.add_hyperedge(vec![a, b, c], hyperedge_one).unwrap();
    graph.add_hyperedge(vec![d, e], hyperedge_two).unwrap();
    graph.add_hyperedge(vec![c, c, c], hyperedge_three).unwrap();
    graph
        .add_hyperedge(vec![e, d, c, a], hyperedge_four)
        .unwrap();

    assert_eq!(
        graph.into_iter().collect::<Vec<(Hyperedge, Vec<Vertex>)>>(),
        vec![
            (hyperedge_one, vec![vertex_one, vertex_two, vertex_three]),
            (hyperedge_two, vec![vertex_four, vertex_five]),
            (hyperedge_three, vec![
                vertex_three,
                vertex_three,
                vertex_three
            ]),
            (hyperedge_four, vec![
                vertex_five,
                vertex_four,
                vertex_three,
                vertex_one
            ])
        ],
        "should provide `into_iter()` yelding a vector of tuples of the form (hyperedge, vector of vertices)"
    );
}
