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
    let vertex_a = Vertex::new("a");
    let vertex_b = Vertex::new("b");
    let vertex_c = Vertex::new("c");
    let vertex_d = Vertex::new("d");
    let vertex_e = Vertex::new("e");
    assert_eq!(graph.add_vertex(vertex_a), 0);
    assert_eq!(graph.add_vertex(vertex_b), 1);
    assert_eq!(graph.add_vertex(vertex_c), 2);
    assert_eq!(graph.add_vertex(vertex_d), 3);
    assert_eq!(graph.add_vertex(vertex_e), 4);
    assert_eq!(graph.add_vertex(vertex_e), 4); // adding the same vertex results in an update.

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
    assert_eq!(graph.get_vertex_weight(0), Some(&vertex_a));
    assert_eq!(graph.get_vertex_weight(4), Some(&vertex_e));
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
    let vertex_a = Vertex::new("brand new heavies");
    assert!(graph.update_vertex_weight(0, vertex_a));
    assert_eq!(graph.get_vertex_weight(0), Some(&vertex_a));

    // Update a hyperedge's weight.
    assert!(graph.update_hyperedge_weight([0, 0], "yup"));
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
    // Previous vertices were [0, 1, 1, 3]!
    assert!(graph.update_hyperedge_vertices(0, &[0, 4]));
    assert_eq!(graph.get_hyperedge_vertices(0), Some(vec![0, 4]));
    assert_eq!(
        graph.get_vertex_hyperedges(0),
        Some(vec![vec![0, 4], vec![4, 0, 3, 2]])
    );
    assert_eq!(graph.get_vertex_hyperedges(1), Some(vec![]));
    assert_eq!(graph.get_vertex_hyperedges(2), Some(vec![vec![4, 0, 3, 2]]));
    assert_eq!(
        graph.get_vertex_hyperedges(3),
        Some(vec![vec![4, 0, 3, 2], vec![3]])
    );
    assert_eq!(
        graph.get_vertex_hyperedges(4),
        Some(vec![vec![4, 0, 3, 2], vec![0, 4]])
    );

    // Remove a vertex with no index alteration since it's the last one.
    assert!(graph.remove_vertex(4));
    assert_eq!(graph.get_hyperedge_vertices(0), Some(vec![0])); // was [0, 4] before
    assert_eq!(graph.get_hyperedge_vertices(1), Some(vec![0, 3, 2])); // was [4, 0, 3, 2] before
    assert_eq!(graph.get_vertex_weight(4), None);
    assert_eq!(graph.count_vertices(), 4);
    assert_eq!(graph.get_vertex_hyperedges(2), Some(vec![vec![0, 3, 2]]));

    // Remove a vertex with index alteration.
    // In this case, index swapping is occurring, i.e. vertex of index 3 will become 0.
    assert!(graph.remove_vertex(0));
    assert_eq!(graph.get_hyperedge_vertices(0), Some(vec![])); // was [0] before
    assert_eq!(graph.get_hyperedge_vertices(1), Some(vec![0, 2])); // was [0, 3, 2] before
    assert_eq!(graph.get_hyperedge_vertices(2), Some(vec![0])); // was [3] before
    assert_eq!(graph.get_vertex_weight(3), None); // should be gone
    assert_eq!(graph.get_vertex_weight(0), Some(&vertex_d)); // index swapping 3 -> 0

    // Render to graphviz dot format.
    graph.render_to_graphviz_dot();
}
