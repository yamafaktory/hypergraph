#![deny(unsafe_code, nonstandard_style)]
#![forbid(rust_2018_idioms)]

mod common;

use common::Vertex;
use hypergraph::{errors::HypergraphError, HyperedgeIndex, Hypergraph, VertexIndex};

#[test]
fn integration_contration() {
    // Create a new hypergraph.
    let mut graph = Hypergraph::<Vertex<'_>, &str>::new();

    // Create some vertices.
    let a = graph.add_vertex(Vertex::new("a")).unwrap();
    let b = graph.add_vertex(Vertex::new("b")).unwrap();
    let c = graph.add_vertex(Vertex::new("c")).unwrap();
    let d = graph.add_vertex(Vertex::new("d")).unwrap();
    let e = graph.add_vertex(Vertex::new("e")).unwrap();

    // Create some hyperedges.
    let alpha = graph.add_hyperedge(vec![a, b, c, d, e], "α").unwrap();
    let beta = graph.add_hyperedge(vec![a, c, d, e, c], "β").unwrap();
    let gamma = graph.add_hyperedge(vec![a, e, b], "γ").unwrap();
    let delta = graph.add_hyperedge(vec![b, c, b, d, c], "δ").unwrap();
    let epsilon = graph.add_hyperedge(vec![c, c, c], "ε").unwrap();

    // In the alpha hyperedge, contract the vertices b and c into one single vertex b.
    assert_eq!(
        graph.contract_hyperedge_vertices(alpha, vec![b, c], b),
        Ok(vec![a, b, d, e]),
        "should contract vertices b and c into b for alpha hyperedge"
    );

    // Check that the other hyperedges have been updated/not updated accordingly.
    assert_eq!(
        graph.get_hyperedge_vertices(beta),
        Ok(vec![a, b, d, e, b]),
        "should update beta hyperedge accordingly"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(gamma),
        Ok(vec![a, e, b]),
        "should not update gamma hyperedge"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(delta),
        Ok(vec![b, d, b]),
        "should not update delta hyperedge"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(epsilon),
        Ok(vec![b]),
        "should update epsilon hyperedge accordingly"
    );

    // Check error handling.
    assert_eq!(
        graph.contract_hyperedge_vertices(HyperedgeIndex(5), vec![b, c], b),
        Err(HypergraphError::HyperedgeIndexNotFound(HyperedgeIndex(5))),
        "should return an explicit error when HyperIndex is not found"
    );
    assert_eq!(
        graph.contract_hyperedge_vertices(alpha, vec![d, e], a),
        Err(HypergraphError::HyperedgeInvalidContraction {
            index: alpha,
            target: a,
            vertices: vec![d, e],
        }),
        "should return an explicit error when the contraction is invalid"
    );
    assert_eq!(
        graph.contract_hyperedge_vertices(
            alpha,
            vec![VertexIndex(5), VertexIndex(6)],
            VertexIndex(5)
        ),
        Err(HypergraphError::HyperedgeVerticesIndexesNotFound {
            index: HyperedgeIndex(0),
            vertices: vec![VertexIndex(5), VertexIndex(6)],
        }),
        "should return an explicit error when the hyperedge doesn't contains the vertices"
    );
}
