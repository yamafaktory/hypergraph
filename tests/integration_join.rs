#![deny(unsafe_code, nonstandard_style)]
#![forbid(rust_2021_compatibility)]

mod common;

use common::{Hyperedge, Vertex};
use hypergraph::{errors::HypergraphError, Hypergraph};

#[test]
fn integration_contration() {
    // Create a new hypergraph.
    let mut graph = Hypergraph::<Vertex, Hyperedge>::new();

    // Create some vertices.
    let a = graph.add_vertex(Vertex::new("a")).unwrap();
    let b = graph.add_vertex(Vertex::new("b")).unwrap();
    let c = graph.add_vertex(Vertex::new("c")).unwrap();
    let d = graph.add_vertex(Vertex::new("d")).unwrap();
    let e = graph.add_vertex(Vertex::new("e")).unwrap();

    // Create some hyperedges.
    let alpha = graph
        .add_hyperedge(vec![a, b, c, d, e], Hyperedge::new("α", 1))
        .unwrap();
    let beta = graph
        .add_hyperedge(vec![a, c, d, e, c], Hyperedge::new("β", 1))
        .unwrap();
    let gamma = graph
        .add_hyperedge(vec![a, e, b], Hyperedge::new("γ", 1))
        .unwrap();
    let delta = graph
        .add_hyperedge(vec![b, c, b, d, c], Hyperedge::new("δ", 1))
        .unwrap();
    let epsilon = graph
        .add_hyperedge(vec![c, c, c], Hyperedge::new("ε", 1))
        .unwrap();

    // Join some hyperedges.
    assert_eq!(
        graph.join_hyperedges(vec![delta, beta, epsilon]),
        Ok(()),
        "should join the delta and beta hyperedges"
    );

    // Check that the length has been updated.
    assert_eq!(
        graph.count_hyperedges(),
        3,
        "should have three hyperedges now"
    );

    // Check the untouched hyperedges.
    assert_eq!(
        graph.get_hyperedge_vertices(alpha),
        Ok(vec![a, b, c, d, e]),
        "should keep alpha untouched"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(gamma),
        Ok(vec![a, e, b]),
        "should keep gamma untouched"
    );

    // Check the removed ones.
    assert_eq!(
        graph.get_hyperedge_vertices(beta),
        Err(HypergraphError::HyperedgeIndexNotFound(beta)),
        "should have removed beta and return an explicit error"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(epsilon),
        Err(HypergraphError::HyperedgeIndexNotFound(epsilon)),
        "should have removed beta and return an explicit error"
    );

    // Check that delta contains the joined vertices.
    assert_eq!(
        graph.get_hyperedge_vertices(delta),
        Ok(vec![b, c, b, d, c, a, c, d, e, c, c, c, c]),
        "should have updated delta"
    );

    // Joining less then two hyperedges should not work.
    assert_eq!(
        graph.join_hyperedges(vec![delta]),
        Err(HypergraphError::HyperedgesInvalidJoin),
        "should return an explicit error"
    );
}
