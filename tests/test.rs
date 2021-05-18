#![deny(unsafe_code, nonstandard_style)]
#![forbid(rust_2018_idioms)]

use hypergraph::Hypergraph;

#[test]
fn integration() {
    // Create a custom struct.
    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
    struct Vertex<'a> {
        name: &'a str,
    }

    impl<'a> Vertex<'a> {
        pub fn new(name: &'a str) -> Self {
            Vertex { name }
        }
    }

    // Create a new hypergraph.
    let mut graph = Hypergraph::<Vertex<'_>, &str>::new();

    // Add some vertices.
    let a = Vertex::new("a");
    assert_eq!(graph.add_vertex(a), 0);
    assert_eq!(graph.add_vertex(Vertex::new("b")), 1);
    assert_eq!(graph.add_vertex(Vertex::new("c")), 2);
    assert_eq!(graph.add_vertex(Vertex::new("d")), 3);
    let e = Vertex::new("e");
    assert_eq!(graph.add_vertex(e), 4);
    assert_eq!(graph.add_vertex(e), 4); // adding the same vertex results in an update.

    // Add some hyperedges.
    assert_eq!(graph.add_hyperedge(&[0, 1, 1, 3], "foo"), [0, 0]); // self-loop.
    assert_eq!(graph.add_hyperedge(&[0, 1, 1, 3], "foo_"), [0, 1]);
    assert_eq!(graph.add_hyperedge(&[4, 0, 3, 2], "bar"), [1, 0]);
    assert_eq!(graph.add_hyperedge(&[3], "woot"), [2, 0]); // unary.
    assert_eq!(graph.add_hyperedge(&[3], "woot"), [2, 0]); // adding the exact same hyperedge results in an update.
    assert_eq!(graph.add_hyperedge(&[3], "leet"), [2, 1]); // another unary on the same vertex with a different weight.

    // Count the vertices and the hyperedges.
    assert_eq!(graph.count_vertices(), 5);
    assert_eq!(graph.count_hyperedges(), 5);

    // Get the weights of some hyperedges and vertices.
    assert_eq!(graph.get_vertex_weight(0), Some(&a));
    assert_eq!(graph.get_vertex_weight(4), Some(&e));
    assert_eq!(graph.get_vertex_weight(5), None); // should not fail!
    assert_eq!(graph.get_hyperedge_weight([0, 0]), Some(&"foo"));
    assert_eq!(graph.get_hyperedge_weight([2, 1]), Some(&"leet"));
    assert_eq!(graph.get_hyperedge_weight([3, 0]), None); // should not fail!

    // Get the vertices of a hyperedge.
    assert_eq!(graph.get_hyperedge_vertices(0), Some(vec![0, 1, 1, 3]));
    assert_eq!(graph.get_hyperedge_vertices(3), None); // should not fail!

    // Get the hyperedges of some vertices as vectors of vertices.
    assert_eq!(
        graph.get_vertex_hyperedges(0),
        Some(vec![vec![0, 1, 1, 3], vec![4, 0, 3, 2]])
    );
    assert_eq!(graph.get_vertex_hyperedges(1), Some(vec![vec![0, 1, 1, 3]]));
    assert_eq!(graph.get_vertex_hyperedges(2), Some(vec![vec![4, 0, 3, 2]]));
    assert_eq!(
        graph.get_vertex_hyperedges(3),
        Some(vec![vec![0, 1, 1, 3], vec![4, 0, 3, 2], vec![3]])
    );
    assert_eq!(graph.get_vertex_hyperedges(4), Some(vec![vec![4, 0, 3, 2]]));

    // Check hyperedges intersections.
    assert_eq!(graph.get_hyperedges_intersections(&[0, 1]), vec![0, 3]);
    assert_eq!(graph.get_hyperedges_intersections(&[0, 1, 2]), vec![3]);
    assert_eq!(graph.get_hyperedges_intersections(&[0]), vec![0, 1, 3]);
    assert_eq!(
        graph.get_hyperedges_intersections(&[3]), // should not fail!
        vec![]
    );

    // Get the hyperedges connecting some vertices.
    assert_eq!(graph.get_hyperedges_connections(1, 1), vec![0]);
    assert_eq!(graph.get_hyperedges_connections(3, 2), vec![1]);
    assert_eq!(graph.get_hyperedges_connections(3, 0), vec![]); // no match, should stay empty!

    // Get the connections from some vertices.
    assert_eq!(graph.get_vertex_connections(0), vec![1, 3]);
    assert_eq!(graph.get_vertex_connections(1), vec![1, 3]);
    assert_eq!(graph.get_vertex_connections(2), vec![]);
    assert_eq!(graph.get_vertex_connections(3), vec![2]);

    // Get some paths via Dijkstra.
    assert_eq!(
        graph.get_dijkstra_connections(4, 2),
        Some(vec![4, 0, 3, 2,])
    );
    assert_eq!(graph.get_dijkstra_connections(0, 3), Some(vec![0, 3,]));
    assert_eq!(graph.get_dijkstra_connections(0, 4), None);
    assert_eq!(graph.get_dijkstra_connections(1, 1), Some(vec![1,]));
    assert_eq!(graph.get_dijkstra_connections(3, 3), Some(vec![3,]));

    // Update a vertex's weight.
    let a = Vertex::new("brand new heavies");
    assert!(graph.update_vertex_weight(0, a));
    assert_eq!(graph.get_vertex_weight(0), Some(&a));

    // Update a hyperedge's weight.
    graph.update_hyperedge_weight([0, 0], "yup");
    assert_eq!(graph.get_hyperedge_weight([0, 0]), Some(&"yup"));
    assert_eq!(
        graph.count_vertices(),
        5,
        "Number of vertices should remain the same."
    );
    assert_eq!(
        graph.count_hyperedges(),
        5,
        "Number of hyperedges should remain the same."
    );

    // Update the vertices of a hyperedge.
    assert!(graph.update_hyperedge_vertices(0, &[0, 4]));
    assert_eq!(graph.get_hyperedge_vertices(0), Some(vec![0, 4]));
    assert_eq!(
        graph.get_vertex_hyperedges(0),
        Some(vec![vec![0, 1, 1, 3], vec![4, 0, 3, 2]])
    );
    assert_eq!(graph.get_vertex_hyperedges(1), Some(vec![vec![0, 1, 1, 3]]));
    assert_eq!(graph.get_vertex_hyperedges(2), Some(vec![vec![4, 0, 3, 2]]));
    assert_eq!(
        graph.get_vertex_hyperedges(3),
        Some(vec![vec![0, 1, 1, 3], vec![4, 0, 3, 2], vec![3]])
    );
    assert_eq!(graph.get_vertex_hyperedges(4), Some(vec![vec![4, 0, 3, 2]]));

    // Remove a vertex.
    // graph.remove_vertex(1);

    // assert_eq!(graph.get_hyperedge_vertices(0), Some(&vec![0, 3]));

    // assert_eq!(graph.get_vertex_weight(0), Some(&a));
    // assert_eq!(graph.get_vertex_weight(1), Some(&a));

    // Render to graphviz dot format.
    // graph.render_to_graphviz_dot();
}
