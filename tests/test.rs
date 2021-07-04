#![deny(unsafe_code, nonstandard_style)]
#![forbid(rust_2018_idioms)]

use hypergraph::{Hypergraph, StableHyperedgeWeightedIndex, StableVertexIndex};
use indexmap::IndexSet;

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
    assert_eq!(graph.add_vertex(vertex_a), StableVertexIndex(0));
    assert_eq!(graph.add_vertex(vertex_b), StableVertexIndex(1));
    assert_eq!(graph.add_vertex(vertex_c), StableVertexIndex(2));
    assert_eq!(graph.add_vertex(vertex_d), StableVertexIndex(3));
    assert_eq!(graph.add_vertex(vertex_e), StableVertexIndex(4));
    assert_eq!(graph.add_vertex(vertex_e), StableVertexIndex(4)); // adding the same vertex results in an update.

    // Add some hyperedges.
    assert_eq!(
        graph.add_hyperedge(&[0, 1, 1, 3], "foo"), // self-loop.
        Some(StableHyperedgeWeightedIndex(0))
    );
    assert_eq!(
        graph.add_hyperedge(&[0, 1, 1, 3], "foo_"),
        Some(StableHyperedgeWeightedIndex(1))
    );
    assert_eq!(
        graph.add_hyperedge(&[4, 0, 3, 2], "bar"),
        Some(StableHyperedgeWeightedIndex(2))
    );
    assert_eq!(
        graph.add_hyperedge(&[3], "woot"), // unary.
        Some(StableHyperedgeWeightedIndex(3))
    );
    assert_eq!(
        graph.add_hyperedge(&[3], "woot"), // adding the exact same hyperedge results in an update.
        Some(StableHyperedgeWeightedIndex(3))
    );
    assert_eq!(
        graph.add_hyperedge(&[3], "leet"), // another unary on the same vertex with a different weight.
        Some(StableHyperedgeWeightedIndex(4))
    );
    assert_eq!(graph.add_hyperedge(&[9], "nope"), None); // out-of-bound, should return None as no-op.

    // Count the vertices and the hyperedges.
    assert_eq!(graph.count_vertices(), 5);
    assert_eq!(graph.count_hyperedges(), 5);

    // Get the weights of some hyperedges and vertices.
    assert_eq!(
        graph.get_vertex_weight(StableVertexIndex(0)),
        Some(&vertex_a)
    );
    assert_eq!(
        graph.get_vertex_weight(StableVertexIndex(4)),
        Some(&vertex_e)
    );
    assert_eq!(graph.get_vertex_weight(StableVertexIndex(5)), None); // should not fail!
    assert_eq!(
        graph.get_hyperedge_weight(StableHyperedgeWeightedIndex(0)),
        Some(&"foo")
    );
    assert_eq!(
        graph.get_hyperedge_weight(StableHyperedgeWeightedIndex(4)),
        Some(&"leet")
    );
    assert_eq!(
        graph.get_hyperedge_weight(StableHyperedgeWeightedIndex(5)), // should not fail!
        None
    );

    // Get the vertices of a hyperedge.
    assert_eq!(
        graph.get_hyperedge_vertices(StableHyperedgeWeightedIndex(0)),
        Some(vec![0, 1, 1, 3])
    );
    assert_eq!(
        graph.get_hyperedge_vertices(StableHyperedgeWeightedIndex(5)), // should not fail!
        None
    );

    // Get the hyperedges of some vertices as vectors of vertices.
    assert_eq!(
        graph.get_vertex_hyperedges(StableVertexIndex(0)),
        Some(vec![vec![0, 1, 1, 3], vec![4, 0, 3, 2]])
    );
    assert_eq!(
        graph.get_vertex_hyperedges(StableVertexIndex(1)),
        Some(vec![vec![0, 1, 1, 3]])
    );
    assert_eq!(
        graph.get_vertex_hyperedges(StableVertexIndex(2)),
        Some(vec![vec![4, 0, 3, 2]])
    );
    assert_eq!(
        graph.get_vertex_hyperedges(StableVertexIndex(3)),
        Some(vec![vec![0, 1, 1, 3], vec![4, 0, 3, 2], vec![3]])
    );
    assert_eq!(
        graph.get_vertex_hyperedges(StableVertexIndex(4)),
        Some(vec![vec![4, 0, 3, 2]])
    );

    // Check hyperedges intersections.
    assert_eq!(
        graph.get_hyperedges_intersections(&[
            StableHyperedgeWeightedIndex(0),
            StableHyperedgeWeightedIndex(2)
        ]),
        vec![0, 3]
    );
    assert_eq!(
        graph.get_hyperedges_intersections(&[
            StableHyperedgeWeightedIndex(0),
            StableHyperedgeWeightedIndex(2),
            StableHyperedgeWeightedIndex(3)
        ]),
        vec![3]
    );
    assert_eq!(
        graph.get_hyperedges_intersections(&[StableHyperedgeWeightedIndex(0)]),
        vec![0, 1, 3]
    );
    assert_eq!(
        graph.get_hyperedges_intersections(&[StableHyperedgeWeightedIndex(5)]), // should not fail!
        vec![]
    );

    // TODO: this is actually wrong!
    // Get the hyperedges connecting some vertices.
    // assert_eq!(
    //     graph.get_hyperedges_connections(StableVertexIndex(1), StableVertexIndex(1)),
    //     vec![0]
    // );
    // assert_eq!(
    //     graph.get_hyperedges_connections(StableVertexIndex(4), StableVertexIndex(2)),
    //     vec![1]
    // );
    // assert_eq!(
    //     graph.get_hyperedges_connections(StableVertexIndex(3), StableVertexIndex(0)),
    //     vec![] // no match, should stay empty!
    // );

    // Get the connections from some vertices.
    assert_eq!(
        graph.get_vertex_connections(StableVertexIndex(0)),
        vec![StableVertexIndex(1), StableVertexIndex(3)]
    );
    assert_eq!(
        graph.get_vertex_connections(StableVertexIndex(1)),
        vec![StableVertexIndex(1), StableVertexIndex(3)]
    );
    assert_eq!(graph.get_vertex_connections(StableVertexIndex(2)), vec![]);
    assert_eq!(
        graph.get_vertex_connections(StableVertexIndex(3)),
        vec![StableVertexIndex(2)]
    );

    // Get some paths via Dijkstra.
    assert_eq!(
        graph.get_dijkstra_connections(StableVertexIndex(4), StableVertexIndex(2)),
        Some(vec![
            StableVertexIndex(4),
            StableVertexIndex(0),
            StableVertexIndex(3),
            StableVertexIndex(2),
        ])
    );
    assert_eq!(
        graph.get_dijkstra_connections(StableVertexIndex(0), StableVertexIndex(3)),
        Some(vec![StableVertexIndex(0), StableVertexIndex(3),])
    );
    assert_eq!(
        graph.get_dijkstra_connections(StableVertexIndex(0), StableVertexIndex(4)),
        None
    );
    assert_eq!(
        graph.get_dijkstra_connections(StableVertexIndex(1), StableVertexIndex(1)),
        Some(vec![StableVertexIndex(1)])
    );
    assert_eq!(
        graph.get_dijkstra_connections(StableVertexIndex(3), StableVertexIndex(3)),
        Some(vec![StableVertexIndex(3)])
    );

    // // Update a vertex's weight.
    // let vertex_a = Vertex::new("brand new heavies");
    // assert!(graph.update_vertex_weight(0, vertex_a));
    // assert_eq!(graph.get_vertex_weight(0), Some(&vertex_a));

    // // Update a hyperedge's weight.
    // assert!(graph.update_hyperedge_weight([0, 0], "yup"));
    // assert_eq!(graph.get_hyperedge_weight([0, 0]), Some(&"yup"));
    // assert_eq!(
    //     graph.count_vertices(),
    //     5,
    //     "Number of vertices should remain the same."
    // );
    // assert_eq!(
    //     graph.count_hyperedges(),
    //     5,
    //     "Number of hyperedges should remain the same."
    // );

    // // Update the vertices of a hyperedge.
    // // Previous vertices were [0, 1, 1, 3]!
    // assert!(graph.update_hyperedge_vertices(0, &[0, 4]));
    // assert_eq!(graph.get_hyperedge_vertices(0), Some(vec![0, 4]));
    // assert_eq!(
    //     graph.get_vertex_hyperedges(0),
    //     Some(vec![vec![0, 4], vec![4, 0, 3, 2]])
    // );
    // assert_eq!(graph.get_vertex_hyperedges(1), Some(vec![]));
    // assert_eq!(graph.get_vertex_hyperedges(2), Some(vec![vec![4, 0, 3, 2]]));
    // assert_eq!(
    //     graph.get_vertex_hyperedges(3),
    //     Some(vec![vec![4, 0, 3, 2], vec![3]])
    // );
    // assert_eq!(
    //     graph.get_vertex_hyperedges(4),
    //     Some(vec![vec![4, 0, 3, 2], vec![0, 4]])
    // );

    // // Remove a vertex with no index alteration since it's the last one.
    // assert!(graph.remove_vertex(4));
    // assert_eq!(graph.get_hyperedge_vertices(0), Some(vec![0])); // was [0, 4] before.
    // assert_eq!(graph.get_hyperedge_vertices(1), Some(vec![0, 3, 2])); // was [4, 0, 3, 2] before.
    // assert_eq!(graph.get_vertex_weight(4), None);
    // assert_eq!(graph.count_vertices(), 4);
    // assert_eq!(graph.get_vertex_hyperedges(2), Some(vec![vec![0, 3, 2]]));

    // // Remove a vertex with index alteration.
    // // In this case, index swapping is occurring, i.e. vertex of index 3 will become 0.
    // assert!(graph.remove_vertex(0));
    // assert_eq!(graph.get_hyperedge_vertices(0), Some(vec![])); // was [0] before.
    // assert_eq!(graph.get_hyperedge_vertices(1), Some(vec![0, 2])); // was [0, 3, 2] before.
    // assert_eq!(graph.get_hyperedge_vertices(2), Some(vec![0])); // was [3] before.
    // assert_eq!(graph.get_vertex_weight(3), None); // should be gone.
    // assert_eq!(graph.get_vertex_weight(0), Some(&vertex_d)); // index swapping 3 -> 0.

    // // Remove a multi-weighted hyperedge with no weight index alteration since it's the last one.
    // assert_eq!(graph.add_hyperedge(&[0], "last"), Some([2, 2])); // add one more for testing reason.
    // assert!(graph.remove_hyperedge([2, 2]));
    // assert_eq!(graph.get_hyperedge_weight([2, 0]), Some(&"woot")); // should still be there.
    // assert_eq!(graph.get_hyperedge_weight([2, 1]), Some(&"leet")); // should still be there too.
    // assert_eq!(graph.get_hyperedge_weight([2, 2]), None); // should be gone.

    // // Remove a multi-weighted hyperedge with weight index alteration.
    // // In this case, index swapping is occurring, i.e. hyperedge of index 1 will become 0.
    // assert!(graph.remove_hyperedge([2, 0]));
    // assert_eq!(graph.get_hyperedge_weight([2, 0]), Some(&"leet")); // was [2, 1] before.
    // assert_eq!(graph.get_hyperedge_weight([2, 1]), None); // should be gone.

    // // Remove a single-weighted hyperedge.
    // // In this case, the hyperedge index is completely removed.
    // // Index alteration doesn't matter since vertices don't directly store it.
    // assert_eq!(graph.get_hyperedge_vertices(2), Some(vec![0]));
    // assert_eq!(graph.get_vertex_hyperedges(0), Some(vec![vec![0]]));
    // assert!(graph.remove_hyperedge([2, 0]));
    // assert_eq!(graph.get_hyperedge_weight([2, 0]), None); // should be gone.
    // assert_eq!(graph.get_vertex_hyperedges(0), Some(vec![]));

    // assert_eq!(
    //     graph
    //         .get_hyperedges()
    //         .collect::<Vec<(&Vec<usize>, &IndexSet<&str>)>>(),
    //     vec![
    //         (&vec![], {
    //             let mut index_set: IndexSet<&str> = IndexSet::new();
    //             index_set.insert("yup");
    //             index_set.insert("foo_");
    //             &index_set.clone()
    //         }),
    //         (&vec![0, 2], {
    //             let mut index_set: IndexSet<&str> = IndexSet::new();
    //             index_set.insert("bar");
    //             &index_set.clone()
    //         })
    //     ]
    // );

    // graph[vertex_d];

    // // Render to graphviz dot format.
    // // graph.render_to_graphviz_dot();
}
