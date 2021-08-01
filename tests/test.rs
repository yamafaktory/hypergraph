#![deny(unsafe_code, nonstandard_style)]
#![forbid(rust_2018_idioms)]

use std::fmt::{Display, Formatter, Result};

use hypergraph::{error::HypergraphError, HyperedgeIndex, Hypergraph, VertexIndex};
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

    impl<'a> Display for Vertex<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "{}", self)
        }
    }

    // Create a new hypergraph.
    let mut graph = Hypergraph::<Vertex<'_>, &str>::new();

    // Add some vertices.
    let andrea = Vertex::new("andrea");
    let björn = Vertex::new("björn");
    let charlie = Vertex::new("charlie");
    let dana = Vertex::new("dana");
    let enola = Vertex::new("enola");
    assert_eq!(graph.add_vertex(andrea), Ok(VertexIndex(0)));
    assert_eq!(graph.add_vertex(björn), Ok(VertexIndex(1)));
    assert_eq!(graph.add_vertex(charlie), Ok(VertexIndex(2)));
    assert_eq!(graph.add_vertex(dana), Ok(VertexIndex(3)));
    assert_eq!(graph.add_vertex(enola), Ok(VertexIndex(4)));
    assert_eq!(graph.add_vertex(enola), Ok(VertexIndex(4))); // adding the same vertex results in an update.

    // Count the vertices.
    assert_eq!(graph.count_vertices(), 5);

    // Add some hyperedges.
    assert_eq!(
        graph.add_hyperedge(
            vec![
                VertexIndex(0),
                VertexIndex(1),
                VertexIndex(1),
                VertexIndex(3)
            ],
            "pass the pink ball"
        ),
        Ok(HyperedgeIndex(0)),
        "this first hyperedge contains a self-loop on the VertexIndex 1"
    );
    assert_eq!(
        graph.add_hyperedge(
            vec![
                VertexIndex(0),
                VertexIndex(1),
                VertexIndex(1),
                VertexIndex(3)
            ],
            "pass the yellow ball",
        ),
        Ok(HyperedgeIndex(1)),
        "this second hyperedge contains the same vertices as the first one"
    );
    assert_eq!(
        graph.add_hyperedge(
            vec![
                VertexIndex(4),
                VertexIndex(0),
                VertexIndex(3),
                VertexIndex(2)
            ],
            "share the \"The Disordered Cosmos: A Journey into Dark Matter, Spacetime, and Dreams Deferred\" book"
        ),
        Ok(HyperedgeIndex(2)),
        "this third hyperedge is unique"
    );
    assert_eq!(
        graph.add_hyperedge(vec![VertexIndex(3)], "meditate like a Jedi"),
        Ok(HyperedgeIndex(3)),
        "this fourth hyperedge contains a unary"
    );
    assert_eq!(
        graph.add_hyperedge(vec![VertexIndex(3)], "meditate like a Jedi"),
        Ok(HyperedgeIndex(3)),
        "this is a no-op since adding the exact same hyperedge results in an update"
    );
    assert_eq!(
        graph.add_hyperedge(vec![VertexIndex(3)], "work out"), 
        Ok(HyperedgeIndex(4)),
        "this fifth hyperedge contains the same unary as the fourth one but with a different weight"
    );
    assert_eq!(
        graph.add_hyperedge(vec![VertexIndex(9)], "nope"),
        Err(HypergraphError::VertexIndexNotFound(VertexIndex(9))),
        "this is out-of-bound and should return an explicit error"
    );

    // Count the hyperedges.
    assert_eq!(graph.count_hyperedges(), 5);

    // // Get the weights of some hyperedges and vertices.
    // assert_eq!(
    //     graph.get_vertex_weight(StableVertexIndex(0)),
    //     Some(vertex_a)
    // );
    // assert_eq!(
    //     graph.get_vertex_weight(StableVertexIndex(4)),
    //     Some(vertex_e)
    // );
    // assert_eq!(graph.get_vertex_weight(StableVertexIndex(5)), None); // should not fail!
    // assert_eq!(
    //     graph.get_hyperedge_weight(StableHyperedgeWeightedIndex(0)),
    //     Some("foo")
    // );
    // assert_eq!(
    //     graph.get_hyperedge_weight(StableHyperedgeWeightedIndex(4)),
    //     Some("leet")
    // );
    // assert_eq!(
    //     graph.get_hyperedge_weight(StableHyperedgeWeightedIndex(5)), // should not fail!
    //     None
    // );

    // // Get the vertices of a hyperedge.
    // assert_eq!(
    //     graph.get_hyperedge_vertices(StableHyperedgeWeightedIndex(0)),
    //     Some(vec![
    //         StableVertexIndex(0),
    //         StableVertexIndex(1),
    //         StableVertexIndex(1),
    //         StableVertexIndex(3)
    //     ])
    // );
    // assert_eq!(
    //     graph.get_hyperedge_vertices(StableHyperedgeWeightedIndex(5)), // should not fail!
    //     None
    // );

    // // Get the hyperedges of some vertices as vectors of vertices.
    // assert_eq!(
    //     graph.get_vertex_hyperedges_full(StableVertexIndex(0)),
    //     Some(vec![
    //         vec![
    //             StableVertexIndex(0),
    //             StableVertexIndex(1),
    //             StableVertexIndex(1),
    //             StableVertexIndex(3)
    //         ],
    //         vec![
    //             StableVertexIndex(0),
    //             StableVertexIndex(1),
    //             StableVertexIndex(1),
    //             StableVertexIndex(3)
    //         ],
    //         vec![
    //             StableVertexIndex(4),
    //             StableVertexIndex(0),
    //             StableVertexIndex(3),
    //             StableVertexIndex(2)
    //         ]
    //     ])
    // );
    // assert_eq!(
    //     graph.get_vertex_hyperedges_full(StableVertexIndex(1)),
    //     Some(vec![
    //         vec![
    //             StableVertexIndex(0),
    //             StableVertexIndex(1),
    //             StableVertexIndex(1),
    //             StableVertexIndex(3)
    //         ],
    //         vec![
    //             StableVertexIndex(0),
    //             StableVertexIndex(1),
    //             StableVertexIndex(1),
    //             StableVertexIndex(3)
    //         ]
    //     ])
    // );
    // assert_eq!(
    //     graph.get_vertex_hyperedges_full(StableVertexIndex(2)),
    //     Some(vec![vec![
    //         StableVertexIndex(4),
    //         StableVertexIndex(0),
    //         StableVertexIndex(3),
    //         StableVertexIndex(2)
    //     ]])
    // );
    // assert_eq!(
    //     graph.get_vertex_hyperedges_full(StableVertexIndex(3)),
    //     Some(vec![
    //         vec![
    //             StableVertexIndex(0),
    //             StableVertexIndex(1),
    //             StableVertexIndex(1),
    //             StableVertexIndex(3)
    //         ],
    //         vec![
    //             StableVertexIndex(0),
    //             StableVertexIndex(1),
    //             StableVertexIndex(1),
    //             StableVertexIndex(3)
    //         ],
    //         vec![
    //             StableVertexIndex(4),
    //             StableVertexIndex(0),
    //             StableVertexIndex(3),
    //             StableVertexIndex(2)
    //         ],
    //         vec![StableVertexIndex(3)],
    //         vec![StableVertexIndex(3)]
    //     ])
    // );
    // assert_eq!(
    //     graph.get_vertex_hyperedges_full(StableVertexIndex(4)),
    //     Some(vec![vec![
    //         StableVertexIndex(4),
    //         StableVertexIndex(0),
    //         StableVertexIndex(3),
    //         StableVertexIndex(2)
    //     ]])
    // );

    // // Check hyperedges intersections.
    // assert_eq!(
    //     graph.get_hyperedges_intersections(&[
    //         StableHyperedgeWeightedIndex(0),
    //         StableHyperedgeWeightedIndex(2)
    //     ]),
    //     vec![0, 3]
    // );
    // assert_eq!(
    //     graph.get_hyperedges_intersections(&[
    //         StableHyperedgeWeightedIndex(0),
    //         StableHyperedgeWeightedIndex(2),
    //         StableHyperedgeWeightedIndex(3)
    //     ]),
    //     vec![3]
    // );
    // assert_eq!(
    //     graph.get_hyperedges_intersections(&[StableHyperedgeWeightedIndex(0)]),
    //     vec![0, 1, 3]
    // );
    // assert_eq!(
    //     graph.get_hyperedges_intersections(&[StableHyperedgeWeightedIndex(5)]), // should not fail!
    //     vec![]
    // );

    // // TODO: this is actually wrong!
    // // Get the hyperedges connecting some vertices.
    // // assert_eq!(
    // //     graph.get_hyperedges_connections(StableVertexIndex(1), StableVertexIndex(1)),
    // //     vec![0]
    // // );
    // // assert_eq!(
    // //     graph.get_hyperedges_connections(StableVertexIndex(4), StableVertexIndex(2)),
    // //     vec![1]
    // // );
    // // assert_eq!(
    // //     graph.get_hyperedges_connections(StableVertexIndex(3), StableVertexIndex(0)),
    // //     vec![] // no match, should stay empty!
    // // );

    // // Get the adjacent vertices of a vertex.
    // assert_eq!(
    //     graph.get_adjacent_vertices_to(StableVertexIndex(0)),
    //     vec![StableVertexIndex(1), StableVertexIndex(3)]
    // );
    // assert_eq!(
    //     graph.get_adjacent_vertices_to(StableVertexIndex(1)),
    //     vec![StableVertexIndex(1), StableVertexIndex(3)]
    // );
    // assert_eq!(graph.get_adjacent_vertices_to(StableVertexIndex(2)), vec![]);
    // assert_eq!(
    //     graph.get_adjacent_vertices_to(StableVertexIndex(3)),
    //     vec![StableVertexIndex(2)]
    // );

    // // Get some paths via Dijkstra.
    // assert_eq!(
    //     graph.get_dijkstra_connections(StableVertexIndex(4), StableVertexIndex(2)),
    //     Some(vec![
    //         StableVertexIndex(4),
    //         StableVertexIndex(0),
    //         StableVertexIndex(3),
    //         StableVertexIndex(2),
    //     ])
    // );
    // assert_eq!(
    //     graph.get_dijkstra_connections(StableVertexIndex(0), StableVertexIndex(3)),
    //     Some(vec![StableVertexIndex(0), StableVertexIndex(3),])
    // );
    // assert_eq!(
    //     graph.get_dijkstra_connections(StableVertexIndex(0), StableVertexIndex(4)),
    //     None
    // );
    // assert_eq!(
    //     graph.get_dijkstra_connections(StableVertexIndex(1), StableVertexIndex(1)),
    //     Some(vec![StableVertexIndex(1)])
    // );
    // assert_eq!(
    //     graph.get_dijkstra_connections(StableVertexIndex(3), StableVertexIndex(3)),
    //     Some(vec![StableVertexIndex(3)])
    // );

    // // Update a vertex's weight.
    // let vertex_a = Vertex::new("brand new heavies");
    // assert!(graph.update_vertex_weight(StableVertexIndex(0), vertex_a));
    // assert_eq!(
    //     graph.get_vertex_weight(StableVertexIndex(0)),
    //     Some(vertex_a)
    // );

    // // Update a hyperedge's weight.
    // assert!(graph.update_hyperedge_weight(StableHyperedgeWeightedIndex(0), "yup"));
    // assert_eq!(
    //     graph.get_hyperedge_weight(StableHyperedgeWeightedIndex(0)),
    //     Some("yup")
    // );
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
    // // Previous vertices were {0, 1, 1, 3}!
    // assert!(graph.update_hyperedge_vertices(
    //     StableHyperedgeWeightedIndex(0),
    //     vec![StableVertexIndex(0), StableVertexIndex(4)]
    // ));
    // assert_eq!(
    //     graph.get_hyperedge_vertices(StableHyperedgeWeightedIndex(0)),
    //     Some(vec![StableVertexIndex(0), StableVertexIndex(4)])
    // );
    // assert_eq!(
    //     graph.get_vertex_hyperedges(StableVertexIndex(0)),
    //     Some(vec![
    //         StableHyperedgeWeightedIndex(0),
    //         StableHyperedgeWeightedIndex(1),
    //         StableHyperedgeWeightedIndex(2),
    //     ])
    // );
    // assert_eq!(
    //     graph.get_vertex_hyperedges_full(StableVertexIndex(0)),
    //     Some(vec![
    //         vec![StableVertexIndex(0), StableVertexIndex(4)],
    //         vec![
    //             StableVertexIndex(0),
    //             StableVertexIndex(1),
    //             StableVertexIndex(1),
    //             StableVertexIndex(3)
    //         ],
    //         vec![
    //             StableVertexIndex(4),
    //             StableVertexIndex(0),
    //             StableVertexIndex(3),
    //             StableVertexIndex(2)
    //         ]
    //     ])
    // );
    // assert_eq!(
    //     graph.get_vertex_hyperedges_full(StableVertexIndex(1)),
    //     Some(vec![vec![
    //         StableVertexIndex(0),
    //         StableVertexIndex(1),
    //         StableVertexIndex(1),
    //         StableVertexIndex(3)
    //     ]])
    // );
    // assert_eq!(
    //     graph.get_vertex_hyperedges_full(StableVertexIndex(2)),
    //     Some(vec![vec![
    //         StableVertexIndex(4),
    //         StableVertexIndex(0),
    //         StableVertexIndex(3),
    //         StableVertexIndex(2)
    //     ]])
    // );
    // assert_eq!(
    //     graph.get_vertex_hyperedges(StableVertexIndex(3)),
    //     Some(vec![
    //         StableHyperedgeWeightedIndex(1),
    //         StableHyperedgeWeightedIndex(2),
    //         StableHyperedgeWeightedIndex(3),
    //         StableHyperedgeWeightedIndex(4),
    //     ])
    // );
    // assert_eq!(
    //     graph.get_vertex_hyperedges_full(StableVertexIndex(3)),
    //     Some(vec![
    //         vec![
    //             StableVertexIndex(0),
    //             StableVertexIndex(1),
    //             StableVertexIndex(1),
    //             StableVertexIndex(3)
    //         ],
    //         vec![
    //             StableVertexIndex(4),
    //             StableVertexIndex(0),
    //             StableVertexIndex(3),
    //             StableVertexIndex(2)
    //         ],
    //         vec![StableVertexIndex(3)],
    //         vec![StableVertexIndex(3)]
    //     ])
    // );
    // assert_eq!(
    //     graph.get_vertex_hyperedges_full(StableVertexIndex(4)),
    //     Some(vec![
    //         vec![
    //             StableVertexIndex(4),
    //             StableVertexIndex(0),
    //             StableVertexIndex(3),
    //             StableVertexIndex(2)
    //         ],
    //         vec![StableVertexIndex(0), StableVertexIndex(4)]
    //     ])
    // );

    // // Check that the graph is still valid.
    // assert_eq!(graph.count_vertices(), 5);
    // assert_eq!(graph.count_hyperedges(), 5);

    // // Remove a vertex with no index alteration since it's the last one.
    // assert!(graph.remove_vertex(StableVertexIndex(4)));
    // assert_eq!(
    //     graph.get_hyperedge_vertices(StableHyperedgeWeightedIndex(0)),
    //     Some(vec![StableVertexIndex(0)]) // was {0, 4} before.
    // );
    // assert_eq!(
    //     graph.get_hyperedge_vertices(StableHyperedgeWeightedIndex(2)),
    //     Some(vec![
    //         StableVertexIndex(0),
    //         StableVertexIndex(3),
    //         StableVertexIndex(2)
    //     ]) // was {4, 0, 3, 2} before.
    // );
    // assert_eq!(graph.get_vertex_weight(StableVertexIndex(4)), None);
    // assert_eq!(graph.count_vertices(), 4);
    // assert_eq!(
    //     graph.get_vertex_hyperedges_full(StableVertexIndex(2)),
    //     Some(vec![vec![
    //         StableVertexIndex(0),
    //         StableVertexIndex(3),
    //         StableVertexIndex(2)
    //     ]])
    // );

    // Remove a vertex with index alteration.
    // In this case, index swapping is occurring, i.e. vertex of unstable index 3 will become 0.
    // assert!(graph.remove_vertex(StableVertexIndex(0)));
    // assert_eq!(
    //     graph.get_hyperedge_vertices(StableHyperedgeWeightedIndex(0)),
    //     Some(vec![]) // was {0} before.
    // );
    // assert_eq!(
    //     graph.get_hyperedge_vertices(StableHyperedgeWeightedIndex(2)),
    //     Some(vec![StableVertexIndex(3), StableVertexIndex(2)]) // was {0, 3, 2} before.
    // );
    // assert_eq!(graph.get_vertex_weight(StableVertexIndex(0)), None); // should be gone.
    // assert_eq!(
    //     graph.get_vertex_weight(StableVertexIndex(3)),
    //     Some(vertex_d) // index swapping 3 -> 0.
    // );

    // // Remove a multi-weighted hyperedge with no weight index alteration since it's the last one.
    // // Start by adding one more hyperedge for testing reasons.
    // dbg!(graph.count_hyperedges());
    // assert_eq!(
    //     graph.add_hyperedge(vec![StableVertexIndex(1)], "last"),
    //     Some(StableHyperedgeWeightedIndex(5))
    // );

    // assert!(graph.remove_hyperedge(StableHyperedgeWeightedIndex(5)));

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
    // assert_eq!(graph.get_vertex_hyperedges_full(0), Some(vec![vec![0]]));
    // assert!(graph.remove_hyperedge([2, 0]));
    // assert_eq!(graph.get_hyperedge_weight([2, 0]), None); // should be gone.
    // assert_eq!(graph.get_vertex_hyperedges_full(0), Some(vec![]));

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
