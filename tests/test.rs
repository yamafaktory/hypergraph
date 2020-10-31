#![forbid(rust_2018_idioms)]

use hypergraph::Hypergraph;

#[test]
fn consume() {
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
    assert_eq!(graph.add_hyperedge(&[0, 1, 1, 3], "foo"), (0, 0)); // self-loop.
    assert_eq!(graph.add_hyperedge(&[4, 0, 3, 2], "bar"), (1, 0));
    assert_eq!(graph.add_hyperedge(&[3], "woot"), (2, 0)); // unary.
    assert_eq!(graph.add_hyperedge(&[3], "woot"), (2, 0)); // adding the exact same hyperedge results in an update.
    assert_eq!(graph.add_hyperedge(&[3], "leet"), (2, 1)); // another unary on the same vertex with a different weight.

    // Count the vertices and the hyperedges.
    assert_eq!(graph.count_vertices(), 5);
    assert_eq!(graph.count_hyperedges(), 4);

    // Get the weights of some hyperedges and vertices.
    assert_eq!(graph.get_vertex_weight(0), Some(&a));
    assert_eq!(graph.get_vertex_weight(4), Some(&e));
    assert_eq!(graph.get_vertex_weight(5), None); // should not fail!
    assert_eq!(graph.get_hyperedge_weight((0, 0)), Some(&"foo"));
    assert_eq!(graph.get_hyperedge_weight((2, 1)), Some(&"leet"));
    assert_eq!(graph.get_hyperedge_weight((3, 0)), None); // should not fail!

    // Get the vertices of a hyperedge.
    assert_eq!(graph.get_hyperedge_vertices(0), Some(&vec![0, 1, 1, 3]));
    assert_eq!(graph.get_hyperedge_vertices(3), None); // should not fail!

    // Check hyperedges intersections.
    assert_eq!(
        graph.get_hyperedges_intersections(&[0, 1]),
        vec![0 as usize, 3 as usize]
    );
    assert_eq!(
        graph.get_hyperedges_intersections(&[0, 1, 2]),
        vec![3 as usize]
    );
    assert_eq!(
        graph.get_hyperedges_intersections(&[0]),
        vec![0 as usize, 1 as usize, 3 as usize]
    );
    assert_eq!(
        graph.get_hyperedges_intersections(&[3]), // should not fail!
        vec![]
    );
}
