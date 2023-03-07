#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]
#![deny(unsafe_code, nonstandard_style)]
#![forbid(rust_2021_compatibility)]
#![warn(missing_debug_implementations, missing_docs, unreachable_pub)]

//! Hypergraph is data structure library to generate directed [hypergraphs](https://en.wikipedia.org/wiki/Hypergraph).
//!
//! > A hypergraph is a generalization of a graph in which a hyperedge can join any number of vertices.
//!
//! ## Features
//!
//! This library enables you to:
//!
//! - represent **non-simple** hypergraphs with two or more hyperedges - with different weights - containing the exact same set of vertices
//! - represent **self-loops** - i.e., hyperedges containing vertices directed to themselves one or more times
//! - represent **unaries** - i.e., hyperedges containing a unique vertex
//!
//! Additional features:
//!
//! - Safe Rust implementation
//! - Proper error handling
//! - Stable indexes assigned for each hyperedge and each vertex
//!
//! ## Example
//!
//! Please notice that the hyperedges and the vertices must implement the
//! [`HyperedgeTrait`](crate::HyperedgeTrait) and the [`VertexTrait`](crate::VertexTrait) respectively.
//!
//! ```
//! use hypergraph::{HyperedgeIndex, Hypergraph, VertexIndex};
//! use std::fmt::{Display, Formatter, Result};
//!
//! // Create a new struct to represent a person.
//! #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
//! pub struct Person<'a> {
//!     name: &'a str,
//! }
//!
//! impl<'a> Person<'a> {
//!     pub fn new(name: &'a str) -> Self {
//!         Self { name }
//!     }
//! }
//!
//! impl<'a> Display for Person<'a> {
//!     fn fmt(&self, f: &mut Formatter<'_>) -> Result {
//!         write!(f, "{}", self)
//!     }
//! }
//!
//! // Create a new struct to represent a relation.
//! #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
//! pub struct Relation<'a> {
//!     cost: usize,
//!     name: &'a str,
//! }
//!
//! impl<'a> Relation<'a> {
//!     pub fn new(name: &'a str, cost: usize) -> Self {
//!         Self { cost, name }
//!     }
//! }
//!
//! impl<'a> Display for Relation<'a> {
//!     fn fmt(&self, f: &mut Formatter<'_>) -> Result {
//!         write!(f, "{}", self)
//!     }
//! }
//!
//! impl<'a> Into<usize> for Relation<'a> {
//!     fn into(self) -> usize {
//!         self.cost
//!     }
//! }
//!
//! fn main() -> std::result::Result<(),  Box<dyn std::error::Error>> {
//!     let mut graph = Hypergraph::<Person, Relation>::new();
//!
//!     // Create some folks.
//!     let ava_person = Person::new("Ava");
//!     let bianca_person = Person::new("Bianca");
//!     let charles_person = Person::new("Charles");
//!     let daena_person = Person::new("Daena");
//!     let ewan_person = Person::new("Ewan");
//!     let faarooq_person = Person::new("Faarooq");
//!     let ghanda_person = Person::new("Ghanda");
//!
//!     // Add those to the graph as vertices.
//!     let ava = graph.add_vertex(ava_person)?;
//!     let bianca = graph.add_vertex(bianca_person)?;
//!     let charles = graph.add_vertex(charles_person)?;
//!     let daena = graph.add_vertex(daena_person)?;
//!     let ewan = graph.add_vertex(ewan_person)?;
//!     let faarooq = graph.add_vertex(faarooq_person)?;
//!     let ghanda = graph.add_vertex(ghanda_person)?;
//!  
//!     // Each vertex gets a unique index by insertion order.
//!     assert_eq!(ava, VertexIndex(0));
//!     assert_eq!(ghanda, VertexIndex(6));
//!
//!     // Get the weight of a vertex.
//!     assert_eq!(graph.get_vertex_weight(VertexIndex(0)), Ok(&ava_person));
//!     
//!     // The hypergraph has seven vertices.
//!     assert_eq!(graph.count_vertices(), 7);
//!
//!     // Create some relations.
//!     let cat_video = Relation::new("share a viral video with a cat", 1);
//!     let dog_video = Relation::new("share a viral video with a dog", 1);
//!     let beaver_video = Relation::new("share a viral video with a beaver", 1);
//!     let playing_online = Relation::new("play online", 1);
//!     let passing_ball = Relation::new("pass the ball", 1);
//!
//!     // Add those to the graph as hyperedges.
//!     let first_relation = graph.add_hyperedge(vec![faarooq, ava, ghanda], cat_video)?;
//!     let second_relation = graph.add_hyperedge(vec![faarooq, ava, ghanda], dog_video)?;
//!     let third_relation = graph.add_hyperedge(vec![ewan, ava, bianca], beaver_video)?;
//!     let fourth_relation = graph.add_hyperedge(vec![daena], playing_online)?;
//!     let fifth_relation = graph.add_hyperedge(vec![ewan, charles, bianca, bianca, ewan], passing_ball)?;
//!
//!     // Each hyperedge gets a unique index by insertion order.
//!     assert_eq!(first_relation, HyperedgeIndex(0));
//!     assert_eq!(fifth_relation, HyperedgeIndex(4));
//!
//!     // Get the weight of a hyperedge.
//!     assert_eq!(graph.get_hyperedge_weight(HyperedgeIndex(0)), Ok(&cat_video));
//!
//!     // Get the vertices of a hyperedge.
//!     assert_eq!(graph.get_hyperedge_vertices(HyperedgeIndex(0)), Ok(vec![faarooq, ava, ghanda]));
//!
//!     // The hypergraph has 5 hyperedges.
//!     assert_eq!(graph.count_hyperedges(), 5);
//!
//!     // Get the hyperedges of a vertex.
//!     assert_eq!(graph.get_vertex_hyperedges(VertexIndex(0)), Ok(vec![first_relation, second_relation, third_relation]));
//!     assert_eq!(graph.get_full_vertex_hyperedges(VertexIndex(0)), Ok(vec![vec![faarooq, ava, ghanda], vec![faarooq, ava, ghanda], vec![ewan, ava, bianca]]));
//!     
//!     // Get the intersection of some hyperedges.
//!     assert_eq!(graph.get_hyperedges_intersections(vec![second_relation, third_relation]), Ok(vec![ava]));
//!
//!     // Find a hyperedge containing a connection between two vertices.
//!     assert_eq!(graph.get_hyperedges_connecting(bianca, bianca), Ok(vec![fifth_relation]));
//!
//!     // Get the adjacent vertices from a vertex.
//!     assert_eq!(graph.get_adjacent_vertices_from(VertexIndex(0)), Ok(vec![bianca, ghanda]));
//!
//!     // Get the adjacent vertices to a vertex.
//!     assert_eq!(graph.get_adjacent_vertices_to(VertexIndex(0)), Ok(vec![ewan, faarooq]));
//!
//!     // Find the shortest paths between some vertices.
//!     assert_eq!(graph.get_dijkstra_connections(faarooq, bianca), Ok(vec![(faarooq, None), (ava, Some(first_relation)), (bianca, Some(third_relation))]));
//!
//!     // Update the weight of a vertex.
//!     graph.update_vertex_weight(ava, Person::new("Avā"))?;
//!     
//!     // Update the weight of a hyperedge.
//!     graph.update_hyperedge_weight(third_relation, Relation::new("share a viral video with a capybara", 1))?;
//!
//!     // Update the vertices of a hyperedge.
//!     graph.update_hyperedge_vertices(third_relation, vec![ewan, ava, daena])?;
//!
//!     // Remove a hyperedge.
//!     graph.remove_hyperedge(first_relation)?;
//!
//!     // Remove a vertex.
//!     graph.remove_vertex(ewan)?;
//!
//!     // Reverse a hyperedge.
//!     graph.reverse_hyperedge(fifth_relation)?;
//!
//!     // Get the in-degree of a vertex.
//!     assert_eq!(graph.get_vertex_degree_in(ava), Ok(1));
//!
//!     // Get the out-degree of a vertex.
//!     assert_eq!(graph.get_vertex_degree_out(ghanda), Ok(0));
//!
//!     // Contract a hyperedge's vertices.
//!     graph.contract_hyperedge_vertices(fifth_relation, vec![bianca, charles], bianca)?;
//!
//!     // Join some hyperedges.
//!     graph.join_hyperedges(&[fifth_relation, third_relation]);
//!
//!     // Clear the hyperedges.
//!     graph.clear_hyperedges()?;
//!
//!     // Clear the whole hypergraph.
//!     graph.clear();
//!
//!     Ok(())
//! }
//! ```

#[doc(hidden)]
pub mod core;

// Reexport of the public API.
#[doc(inline)]
pub use crate::core::*;
