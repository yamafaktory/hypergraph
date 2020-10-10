extern crate hypergraph;

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
    let mut graph = Hypergraph::<Vertex, &str>::default();

    // Add some nodes.
    let a = Vertex::new("a");
    assert_eq!(graph.add_vertex(a), 0);
    assert_eq!(graph.add_vertex(Vertex::new("b")), 1);
    assert_eq!(graph.add_vertex(Vertex::new("c")), 2);
    assert_eq!(graph.add_vertex(Vertex::new("d")), 3);
    assert_eq!(graph.add_vertex(Vertex::new("e")), 4);

    // Add some hyperedges.
    assert_eq!(graph.add_hyperedge(&[0, 1, 1, 3], "foo"), 0); // self-loop.
    assert_eq!(graph.add_hyperedge(&[4, 0, 3, 2], "bar"), 1);
    assert_eq!(graph.add_hyperedge(&[3], "foo"), 2); // unary.

    // Count the vertices and the hyperedges.
    assert_eq!(graph.count_vertices(), 5);
    assert_eq!(graph.count_hyperedges(), 3);

    // Get the weights of a vertex and a hyperedge.
    assert_eq!(graph.get_vertex_weight(0), Some(&a));
    assert_eq!(graph.get_hyperedge_weight(0), Some(&"foo"));

    // Get the vertices of a hyperedge.
    assert_eq!(graph.get_hyperedge_vertices(0), Some(&vec![0, 1, 1, 3]));

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

    dbg!(graph);
}
