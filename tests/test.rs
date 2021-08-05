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

    // Create some vertices.
    let andrea = Vertex::new("Andrea");
    let bjǫrn = Vertex::new("Bjǫrn");
    let charlie = Vertex::new("Charlie");
    let dana = Vertex::new("Dana");
    let enola = Vertex::new("Enola");

    // Add those vertices to the hypergraph.
    assert_eq!(
        graph.add_vertex(andrea),
        Ok(VertexIndex(0)),
        "should add the first vertex"
    );
    assert_eq!(
        graph.add_vertex(bjǫrn),
        Ok(VertexIndex(1)),
        "should add the second vertex"
    );
    assert_eq!(
        graph.add_vertex(charlie),
        Ok(VertexIndex(2)),
        "should add the third vertex"
    );
    assert_eq!(
        graph.add_vertex(dana),
        Ok(VertexIndex(3)),
        "should add the fourth vertex"
    );
    assert_eq!(
        graph.add_vertex(enola),
        Ok(VertexIndex(4)),
        "should add the fifth vertex"
    );
    assert_eq!(
        graph.add_vertex(enola),
        Ok(VertexIndex(4)),
        "should be a no-op since adding the exact same vertex results in an update"
    );

    // Count the vertices.
    assert_eq!(graph.count_vertices(), 5, "should have 5 vertices");

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
        "should add a first hyperedge which contains a self-loop on the VertexIndex 1"
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
        "should add a second hyperedge which contains the same vertices as the first one"
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
        "should add a third hyperedge which is unique"
    );
    assert_eq!(
        graph.add_hyperedge(vec![VertexIndex(3)], "meditate like a Jedi"),
        Ok(HyperedgeIndex(3)),
        "should add a fourth hyperedge which contains a unary"
    );
    assert_eq!(
        graph.add_hyperedge(vec![VertexIndex(3)], "meditate like a Jedi"),
        Err(HypergraphError::HyperedgeWeightAlreadyAssigned(
            "meditate like a Jedi"
        )),
        "should return an explicit error since this weight is already in use"
    );
    assert_eq!(
        graph.add_hyperedge(vec![VertexIndex(3)], "work out"), 
        Ok(HyperedgeIndex(4)),
        "should add a fifth hyperedge which contains the same unary as the fourth one but with a different weight"
    );
    assert_eq!(
        graph.add_hyperedge(vec![VertexIndex(9)], "nope"),
        Err(HypergraphError::VertexIndexNotFound(VertexIndex(9))),
        "should be out-of-bound and return an explicit error"
    );

    // Count the hyperedges.
    assert_eq!(graph.count_hyperedges(), 5, "should have 5 hyperedges");

    // Get the weights of some vertices.
    assert_eq!(
        graph.get_vertex_weight(VertexIndex(0)),
        Ok(andrea),
        "should return Andrea"
    );
    assert_eq!(
        graph.get_vertex_weight(VertexIndex(4)),
        Ok(enola),
        "should return Enola"
    );
    assert_eq!(
        graph.get_vertex_weight(VertexIndex(5)),
        Err(HypergraphError::VertexIndexNotFound(VertexIndex(5))),
        "should be out-of-bound and return an explicit error"
    );

    // Get the weights of some hyperedges.
    assert_eq!(
        graph.get_hyperedge_weight(HyperedgeIndex(0)),
        Ok("pass the pink ball"),
        "should get the weight of the first hyperedge"
    );
    assert_eq!(
        graph.get_hyperedge_weight(HyperedgeIndex(4)),
        Ok("work out"),
        "should get the weight of the fifth hyperedge"
    );
    assert_eq!(
        graph.get_hyperedge_weight(HyperedgeIndex(5)),
        Err(HypergraphError::HyperedgeIndexNotFound(HyperedgeIndex(5))),
        "should be out-of-bound and return an explicit error"
    );

    // Get the vertices of some hyperedges.
    assert_eq!(
        graph.get_hyperedge_vertices(HyperedgeIndex(0)),
        Ok(vec![
            VertexIndex(0),
            VertexIndex(1),
            VertexIndex(1),
            VertexIndex(3)
        ]),
        "should get the vertices of the first hyperedge"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(HyperedgeIndex(5)),
        Err(HypergraphError::HyperedgeIndexNotFound(HyperedgeIndex(5))),
        "should be out-of-bound and return an explicit error"
    );

    // Get the hyperedges of some vertices as vectors of HyperedgeIndex
    // and vectors of vectors of VertexIndex (full version).
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(0)),
        Ok(vec![
            HyperedgeIndex(0),
            HyperedgeIndex(1),
            HyperedgeIndex(2)
        ]),
        "should get the hyperedges of the first vertex"
    );
    assert_eq!(
        graph.get_full_vertex_hyperedges(VertexIndex(0)),
        Ok(vec![
            vec![
                VertexIndex(0),
                VertexIndex(1),
                VertexIndex(1),
                VertexIndex(3)
            ],
            vec![
                VertexIndex(0),
                VertexIndex(1),
                VertexIndex(1),
                VertexIndex(3)
            ],
            vec![
                VertexIndex(4),
                VertexIndex(0),
                VertexIndex(3),
                VertexIndex(2)
            ]
        ]),
        "should get the hyperedges of the first vertex - full version"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(1)),
        Ok(vec![HyperedgeIndex(0), HyperedgeIndex(1),]),
        "should get the hyperedges of the second vertex"
    );
    assert_eq!(
        graph.get_full_vertex_hyperedges(VertexIndex(1)),
        Ok(vec![
            vec![
                VertexIndex(0),
                VertexIndex(1),
                VertexIndex(1),
                VertexIndex(3)
            ],
            vec![
                VertexIndex(0),
                VertexIndex(1),
                VertexIndex(1),
                VertexIndex(3)
            ]
        ]),
        "should get the hyperedges of the second vertex - full version"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(2)),
        Ok(vec![HyperedgeIndex(2)]),
        "should get the hyperedges of the third vertex"
    );
    assert_eq!(
        graph.get_full_vertex_hyperedges(VertexIndex(2)),
        Ok(vec![vec![
            VertexIndex(4),
            VertexIndex(0),
            VertexIndex(3),
            VertexIndex(2)
        ]]),
        "should get the hyperedges of the third vertex - full version"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(3)),
        Ok(vec![
            HyperedgeIndex(0),
            HyperedgeIndex(1),
            HyperedgeIndex(2),
            HyperedgeIndex(3),
            HyperedgeIndex(4)
        ]),
        "should get the hyperedges of the fourth vertex"
    );
    assert_eq!(
        graph.get_full_vertex_hyperedges(VertexIndex(3)),
        Ok(vec![
            vec![
                VertexIndex(0),
                VertexIndex(1),
                VertexIndex(1),
                VertexIndex(3)
            ],
            vec![
                VertexIndex(0),
                VertexIndex(1),
                VertexIndex(1),
                VertexIndex(3)
            ],
            vec![
                VertexIndex(4),
                VertexIndex(0),
                VertexIndex(3),
                VertexIndex(2)
            ],
            vec![VertexIndex(3)],
            vec![VertexIndex(3)]
        ]),
        "should get the hyperedges of the fourth vertex - full version"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(4)),
        Ok(vec![HyperedgeIndex(2)]),
        "should get the hyperedges of the fifth vertex"
    );
    assert_eq!(
        graph.get_full_vertex_hyperedges(VertexIndex(4)),
        Ok(vec![vec![
            VertexIndex(4),
            VertexIndex(0),
            VertexIndex(3),
            VertexIndex(2)
        ]]),
        "should get the hyperedges of the fifth vertex - full version"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(5)),
        Err(HypergraphError::VertexIndexNotFound(VertexIndex(5))),
        "should be out-of-bound and return an explicit error"
    );
    assert_eq!(
        graph.get_full_vertex_hyperedges(VertexIndex(5)),
        Err(HypergraphError::VertexIndexNotFound(VertexIndex(5))),
        "should be out-of-bound and return an explicit error"
    );

    // Check hyperedges intersections.
    assert_eq!(
        graph.get_hyperedges_intersections(vec![HyperedgeIndex(0), HyperedgeIndex(2)]),
        Ok(vec![VertexIndex(0), VertexIndex(3)]),
        "should get two intersections"
    );
    assert_eq!(
        graph.get_hyperedges_intersections(vec![
            HyperedgeIndex(0),
            HyperedgeIndex(2),
            HyperedgeIndex(3)
        ]),
        Ok(vec![VertexIndex(3)]),
        "should get one intersection"
    );
    assert_eq!(
        graph.get_hyperedges_intersections(vec![HyperedgeIndex(0), HyperedgeIndex(0),]),
        Ok(vec![VertexIndex(0), VertexIndex(1), VertexIndex(3)]),
        "should return all the vertices of a hyperedge intersecting itself"
    );
    assert_eq!(
        graph.get_hyperedges_intersections(vec![]),
        Err(HypergraphError::HyperedgesIntersections),
        "should fail since computing the intersections of less than two hyperedges is not possible"
    );
    assert_eq!(
        graph.get_hyperedges_intersections(vec![HyperedgeIndex(0)]),
        Err(HypergraphError::HyperedgesIntersections),
        "should fail since computing the intersections of less than two hyperedges is not possible"
    );
    assert_eq!(
        graph.get_hyperedges_intersections(vec![HyperedgeIndex(5), HyperedgeIndex(6)]),
        Err(HypergraphError::HyperedgeIndexNotFound(HyperedgeIndex(5))),
        "should be out-of-bound and return an explicit error"
    );

    // Get the hyperedges directly connecting a vertex to another.
    assert_eq!(
        graph.get_hyperedges_connecting(VertexIndex(1), VertexIndex(1)),
        Ok(vec![HyperedgeIndex(0), HyperedgeIndex(1)]),
        "should get two matches"
    );
    assert_eq!(
        graph.get_hyperedges_connecting(VertexIndex(4), VertexIndex(0)),
        Ok(vec![HyperedgeIndex(2)]),
        "should get one match"
    );
    assert_eq!(
        graph.get_hyperedges_connecting(VertexIndex(3), VertexIndex(0)),
        Ok(vec![]),
        "should get no match"
    );
    assert_eq!(
        graph.get_hyperedges_connecting(VertexIndex(5), VertexIndex(0)),
        Err(HypergraphError::VertexIndexNotFound(VertexIndex(5))),
        "should be out-of-bound and return an explicit error"
    );

    // Get the adjacent vertices to a vertex.
    assert_eq!(
        graph.get_adjacent_vertices_to(VertexIndex(0)),
        Ok(vec![VertexIndex(1), VertexIndex(3)])
    );
    assert_eq!(
        graph.get_adjacent_vertices_to(VertexIndex(1)),
        Ok(vec![VertexIndex(1), VertexIndex(3)])
    );
    assert_eq!(graph.get_adjacent_vertices_to(VertexIndex(2)), Ok(vec![]));
    assert_eq!(
        graph.get_adjacent_vertices_to(VertexIndex(3)),
        Ok(vec![VertexIndex(2)])
    );
    assert_eq!(
        graph.get_adjacent_vertices_to(VertexIndex(5)),
        Err(HypergraphError::VertexIndexNotFound(VertexIndex(5))),
        "should be out-of-bound and return an explicit error"
    );

    // Get some paths via Dijkstra.
    assert_eq!(
        graph.get_dijkstra_connections(VertexIndex(4), VertexIndex(1)),
        Ok(vec![VertexIndex(4), VertexIndex(0), VertexIndex(1),]),
        "should get a path of three vertices"
    );
    assert_eq!(
        graph.get_dijkstra_connections(VertexIndex(0), VertexIndex(3)),
        Ok(vec![VertexIndex(0), VertexIndex(3)]),
        "should get a path of two vertices"
    );
    assert_eq!(
        graph.get_dijkstra_connections(VertexIndex(0), VertexIndex(4)),
        Ok(vec![]),
        "should get an empty path"
    );
    assert_eq!(
        graph.get_dijkstra_connections(VertexIndex(1), VertexIndex(1)),
        Ok(vec![VertexIndex(1)]),
        "should get a path of one vertex"
    );
    assert_eq!(
        graph.get_dijkstra_connections(VertexIndex(3), VertexIndex(3)),
        Ok(vec![VertexIndex(3)]),
        "should get a path of one vertex"
    );
    assert_eq!(
        graph.get_dijkstra_connections(VertexIndex(3), VertexIndex(5)),
        Err(HypergraphError::VertexIndexNotFound(VertexIndex(5))),
        "should be out-of-bound and return an explicit error"
    );

    // Update the weight of a vertex.
    let bjǫrg = Vertex::new("Bjǫrg");
    assert_eq!(graph.update_vertex_weight(VertexIndex(1), bjǫrg), Ok(()));
    assert_eq!(
        graph.get_vertex_weight(VertexIndex(1)),
        Ok(bjǫrg),
        "should return Bjǫrg instead of Bjǫrn"
    );
    assert_eq!(graph.count_vertices(), 5, "should still have 5 vertices");

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
