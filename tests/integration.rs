#![deny(unsafe_code, nonstandard_style)]
#![forbid(rust_2018_idioms)]

use std::fmt::{Display, Formatter, Result};

use hypergraph::{error::HypergraphError, HyperedgeIndex, Hypergraph, VertexIndex};

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
        Err(HypergraphError::VertexWeightAlreadyAssigned(enola)),
        "should return an explicit error since this weight is already in use"
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
    assert_eq!(
        graph.add_hyperedge(vec![], "nope"),
        Err(HypergraphError::HyperedgeCreationNoVertices("nope")),
        "should return an explicit error since the vertices are missing"
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

    // Get the adjacent vertices from a vertex.
    assert_eq!(
        graph.get_adjacent_vertices_from(VertexIndex(0)),
        Ok(vec![VertexIndex(1), VertexIndex(3)])
    );
    assert_eq!(
        graph.get_adjacent_vertices_from(VertexIndex(1)),
        Ok(vec![VertexIndex(1), VertexIndex(3)])
    );
    assert_eq!(graph.get_adjacent_vertices_from(VertexIndex(2)), Ok(vec![]));
    assert_eq!(
        graph.get_adjacent_vertices_from(VertexIndex(3)),
        Ok(vec![VertexIndex(2)])
    );
    assert_eq!(
        graph.get_adjacent_vertices_from(VertexIndex(5)),
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

    // Update a hyperedge's weight.
    // First case: the index is the last one, no internal index alteration
    // occurs.
    assert_eq!(
        graph.update_hyperedge_weight(HyperedgeIndex(4), "sleep"),
        Ok(()),
        "should update the weight of the fifth hyperedge"
    );
    assert_eq!(
        graph.get_hyperedge_weight(HyperedgeIndex(4)),
        Ok("sleep"),
        "should get the new weight of the fifth hyperedge"
    );
    assert_eq!(
        graph.count_hyperedges(),
        5,
        "should still have 5 hyperedges"
    );
    // Second case: the index is not the last one, an internal index alteration
    // occurs but is anyway fixed by the insertion.
    assert_eq!(
        graph.update_hyperedge_weight(HyperedgeIndex(0), "pass the purple ball"),
        Ok(()),
        "should update the weight of the first hyperedge"
    );
    assert_eq!(
        graph.get_hyperedge_weight(HyperedgeIndex(0)),
        Ok("pass the purple ball"),
        "should get the new weight of the first hyperedge"
    );
    assert_eq!(
        graph.count_hyperedges(),
        5,
        "should still have 5 hyperedges"
    );
    // Check the eventual errors.
    assert_eq!(
        graph.update_hyperedge_weight(HyperedgeIndex(0), "pass the purple ball"),
        Err(HypergraphError::HyperedgeWeightUnchanged(
            HyperedgeIndex(0),
            "pass the purple ball"
        )),
        "should return an explicit error since this weight has not changed"
    );
    assert_eq!(
        graph.update_hyperedge_weight(HyperedgeIndex(0), "meditate like a Jedi"),
        Err(HypergraphError::HyperedgeWeightAlreadyAssigned(
            "meditate like a Jedi"
        )),
        "should return an explicit error since this weight is already assigned"
    );

    // Update the vertices of some hyperedges.
    assert_eq!(
        graph.update_hyperedge_vertices(HyperedgeIndex(0), vec![VertexIndex(0), VertexIndex(4)]),
        Ok(()),
        "should update the vertices of the first hyperedge from [0, 1, 1, 3] to [0, 4]"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(HyperedgeIndex(0)),
        Ok(vec![VertexIndex(0), VertexIndex(4)]),
        "should get the updated vertices of the first hyperedge"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(0)),
        Ok(vec![
            HyperedgeIndex(0),
            HyperedgeIndex(1),
            HyperedgeIndex(2)
        ]),
        "should get the same hyperedges for the first vertex"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(1)),
        Ok(vec![HyperedgeIndex(1)]),
        "should get different hyperedges for the second vertex - removed"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(3)),
        Ok(vec![
            HyperedgeIndex(4),
            HyperedgeIndex(1),
            HyperedgeIndex(2),
            HyperedgeIndex(3)
        ]),
        "should get different hyperedges for the fourth vertex - removed"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(4)),
        Ok(vec![HyperedgeIndex(2), HyperedgeIndex(0),]),
        "should get different hyperedges for the fifth vertex - added"
    );
    assert_eq!(
        graph.update_hyperedge_vertices(HyperedgeIndex(5), vec![VertexIndex(0), VertexIndex(4)]),
        Err(HypergraphError::HyperedgeIndexNotFound(HyperedgeIndex(5))),
        "should be out-of-bound and return an explicit error"
    );
    assert_eq!(
        graph.update_hyperedge_vertices(HyperedgeIndex(0), vec![]),
        Err(HypergraphError::HyperedgeUpdateNoVertices(HyperedgeIndex(
            0
        ))),
        "should return an explicit error since the vertices are missing"
    );
    assert_eq!(
        graph.update_hyperedge_vertices(HyperedgeIndex(0), vec![VertexIndex(0), VertexIndex(4)]),
        Err(HypergraphError::HyperedgeVerticesUnchanged(HyperedgeIndex(
            0
        ))),
        "should return an explicit error since the vertices have not changed"
    );

    // Check the hypergraph integrity.
    assert_eq!(graph.count_vertices(), 5);
    assert_eq!(graph.count_hyperedges(), 5);

    // Remove one hyperedge.
    // Start with the last one. No remapping is occurring internally.
    assert_eq!(
        graph.remove_hyperedge(HyperedgeIndex(4)),
        Ok(()),
        "should remove the fifth hyperedge"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(HyperedgeIndex(4)),
        Err(HypergraphError::HyperedgeIndexNotFound(HyperedgeIndex(4))),
        "should return an explicit error since the hyperedge has been removed"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(0)),
        Ok(vec![
            HyperedgeIndex(0),
            HyperedgeIndex(1),
            HyperedgeIndex(2)
        ]),
        "should get the same hyperedges for the first vertex"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(1)),
        Ok(vec![HyperedgeIndex(1)]),
        "should get the same hyperedges for the second vertex"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(2)),
        Ok(vec![HyperedgeIndex(2)]),
        "should get the same hyperedges for the third vertex"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(3)),
        Ok(vec![
            HyperedgeIndex(3),
            HyperedgeIndex(1),
            HyperedgeIndex(2),
        ]),
        "should get different hyperedges for the fourth vertex - removed"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(4)),
        Ok(vec![HyperedgeIndex(2), HyperedgeIndex(0),]),
        "should get the same hyperedges for the fifth vertex"
    );

    // Check the hypergraph integrity.
    assert_eq!(graph.count_vertices(), 5);
    assert_eq!(graph.count_hyperedges(), 4);

    // Remove another hyperedge.
    // Now remove the first one. A remapping is occurring internally.
    assert_eq!(
        graph.remove_hyperedge(HyperedgeIndex(0)),
        Ok(()),
        "should remove the first hyperedge"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(HyperedgeIndex(0)),
        Err(HypergraphError::HyperedgeIndexNotFound(HyperedgeIndex(0))),
        "should return an explicit error since the hyperedge has been removed"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(0)),
        Ok(vec![HyperedgeIndex(2), HyperedgeIndex(1),]),
        "should get different hyperedges for the first vertex - removed"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(1)),
        Ok(vec![HyperedgeIndex(1)]),
        "should get the same hyperedges for the second vertex"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(2)),
        Ok(vec![HyperedgeIndex(2)]),
        "should get the same hyperedges for the third vertex"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(3)),
        Ok(vec![
            HyperedgeIndex(3),
            HyperedgeIndex(1),
            HyperedgeIndex(2),
        ]),
        "should get the same hyperedges for the fourth vertex"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(4)),
        Ok(vec![HyperedgeIndex(2)]),
        "should get different hyperedges for the fifth vertex"
    );
    assert_eq!(
        graph.remove_hyperedge(HyperedgeIndex(0)),
        Err(HypergraphError::HyperedgeIndexNotFound(HyperedgeIndex(0))),
        "should be out-of-bound and return an explicit error"
    );

    // Check the hypergraph integrity.
    assert_eq!(graph.count_vertices(), 5);
    assert_eq!(graph.count_hyperedges(), 3);

    // Remove a vertex.
    // Start with the last one. No remapping is occurring internally.
    assert_eq!(graph.remove_vertex(VertexIndex(4)), Ok(()));
    assert_eq!(
        graph.get_vertex_weight(VertexIndex(4)),
        Err(HypergraphError::VertexIndexNotFound(VertexIndex(4))),
        "should be out-of-bound and return an explicit error - removed"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(HyperedgeIndex(1)),
        Ok(vec![
            VertexIndex(0),
            VertexIndex(1),
            VertexIndex(1),
            VertexIndex(3)
        ]),
        "should get the same vertices for the second hyperedge"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(HyperedgeIndex(2)),
        Ok(vec![VertexIndex(0), VertexIndex(3), VertexIndex(2)]),
        "should get different vertices for the third hyperedge - removed"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(HyperedgeIndex(3)),
        Ok(vec![VertexIndex(3)]),
        "should get the same vertices for the fourth hyperedge"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(0)),
        Ok(vec![HyperedgeIndex(2), HyperedgeIndex(1)]),
        "should get the hyperedges of the first vertex"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(1)),
        Ok(vec![HyperedgeIndex(1)]),
        "should get the hyperedges of the second vertex"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(2)),
        Ok(vec![HyperedgeIndex(2)]),
        "should get the hyperedges of the third vertex"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(3)),
        Ok(vec![
            HyperedgeIndex(3),
            HyperedgeIndex(1),
            HyperedgeIndex(2)
        ]),
        "should get the hyperedges of the fourth vertex"
    );

    // Check the hypergraph integrity.
    assert_eq!(graph.count_vertices(), 4);
    assert_eq!(graph.count_hyperedges(), 3);

    // Remove another vertex.
    // Now remove the first one. A remapping is occurring internally.
    assert_eq!(graph.remove_vertex(VertexIndex(0)), Ok(()));
    assert_eq!(
        graph.get_vertex_weight(VertexIndex(0)),
        Err(HypergraphError::VertexIndexNotFound(VertexIndex(0))),
        "should be out-of-bound and return an explicit error - removed"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(HyperedgeIndex(1)),
        Ok(vec![VertexIndex(1), VertexIndex(1), VertexIndex(3)]),
        "should get the different vertices for the second hyperedge - removed"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(HyperedgeIndex(2)),
        Ok(vec![VertexIndex(3), VertexIndex(2)]),
        "should get different vertices for the third hyperedge - removed"
    );
    assert_eq!(
        graph.get_hyperedge_vertices(HyperedgeIndex(3)),
        Ok(vec![VertexIndex(3)]),
        "should get the same vertices for the fourth hyperedge"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(1)),
        Ok(vec![HyperedgeIndex(1)]),
        "should get the hyperedges of the second vertex"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(2)),
        Ok(vec![HyperedgeIndex(2)]),
        "should get the hyperedges of the third vertex"
    );
    assert_eq!(
        graph.get_vertex_hyperedges(VertexIndex(3)),
        Ok(vec![
            HyperedgeIndex(3),
            HyperedgeIndex(1),
            HyperedgeIndex(2)
        ]),
        "should get the hyperedges of the fourth vertex"
    );

    // Check the hypergraph integrity.
    assert_eq!(graph.count_vertices(), 3);
    assert_eq!(graph.count_hyperedges(), 3);

    // // Render to graphviz dot format.
    // // graph.render_to_graphviz_dot();
}
