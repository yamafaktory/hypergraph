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
//! Please notice that the hyperedges and the vertices must implement the [SharedTrait](crate::SharedTrait).
//!
//! ```
//! use hypergraph::{HyperedgeIndex, Hypergraph, VertexIndex};
//! use std::fmt::{Display, Formatter, Result};
//!
//! // Create a new struct to represent a vertex.
//! #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
//! pub struct Vertex<'a> {
//!     cost: usize,
//!     name: &'a str,
//! }
//!
//! impl<'a> Vertex<'a> {
//!     pub fn new(name: &'a str, cost: usize) -> Self {
//!         Vertex { cost, name }
//!     }
//! }
//!
//! impl<'a> Display for Vertex<'a> {
//!     fn fmt(&self, f: &mut Formatter<'_>) -> Result {
//!         write!(f, "{}", self)
//!     }
//! }
//!
//! impl<'a> Into<usize> for Vertex<'a> {
//!     fn into(self) -> usize {
//!         self.cost
//!     }
//! }
//!
//! // Create a new struct to represent a hyperedge.
//! #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
//! pub struct HyperEdge<'a> {
//!     cost: usize,
//!     name: &'a str,
//! }
//!
//! impl<'a> HyperEdge<'a> {
//!     pub fn new(name: &'a str, cost: usize) -> Self {
//!         HyperEdge { cost, name }
//!     }
//! }
//!
//! impl<'a> Display for HyperEdge<'a> {
//!     fn fmt(&self, f: &mut Formatter<'_>) -> Result {
//!         write!(f, "{}", self)
//!     }
//! }
//!
//! impl<'a> Into<usize> for HyperEdge<'a> {
//!     fn into(self) -> usize {
//!         self.cost
//!     }
//! }
//!
//! fn main() -> std::result::Result<(),  Box<dyn std::error::Error>> {
//!     let mut graph = Hypergraph::<Vertex, HyperEdge>::new();
//!
//!     // Add some vertices to the graph.
//!     let ava = graph.add_vertex(Vertex::new("Ava", 1))?;
//!     let bianca = graph.add_vertex(Vertex::new("Bianca", 1))?;
//!     let charles = graph.add_vertex(Vertex::new("Charles", 1))?;
//!     let daena = graph.add_vertex(Vertex::new("Daena", 1))?;
//!     let ewan = graph.add_vertex(Vertex::new("Ewan", 1))?;
//!     let faarooq = graph.add_vertex(Vertex::new("Faarooq", 1))?;
//!     let ghanda = graph.add_vertex(Vertex::new("Ghanda", 1))?;
//!  
//!     // Each vertex gets a unique index by insertion order.
//!     assert_eq!(ava, VertexIndex(0));
//!     assert_eq!(ghanda, VertexIndex(6));
//!
//!     // Get the weight of a vertex.
//!     assert_eq!(graph.get_vertex_weight(VertexIndex(0)), Ok(Vertex::new("Ava", 1)));
//!     
//!     // The hypergraph has 7 vertices.
//!     assert_eq!(graph.count_vertices(), 7);
//!
//!     // Add some hyperedges to the graph.
//!     let first_hyperedge = graph.add_hyperedge(vec![faarooq, ava, ghanda], HyperEdge::new("share a viral video with a cat", 1))?;
//!     let second_hyperedge = graph.add_hyperedge(vec![faarooq, ava, ghanda], HyperEdge::new("share a viral video with a dog", 1))?;
//!     let third_hyperedge = graph.add_hyperedge(vec![ewan, ava, bianca], HyperEdge::new("share a viral video with a beaver", 1))?;
//!     let fourth_hyperedge = graph.add_hyperedge(vec![daena], HyperEdge::new("play online", 1))?;
//!     let fifth_hyperedge = graph.add_hyperedge(vec![ewan, charles, bianca, bianca, ewan], HyperEdge::new("pass the ball", 1))?;
//!
//!     // Each hyperedge gets a unique index by insertion order.
//!     assert_eq!(first_hyperedge, HyperedgeIndex(0));
//!     assert_eq!(fifth_hyperedge, HyperedgeIndex(4));
//!
//!     // Get the weight of a hyperedge.
//!     assert_eq!(graph.get_hyperedge_weight(HyperedgeIndex(0)), Ok(HyperEdge::new("share a viral video with a cat", 1)));
//!
//!     // Get the vertices of a hyperedge.
//!     assert_eq!(graph.get_hyperedge_vertices(HyperedgeIndex(0)), Ok(vec![faarooq, ava, ghanda]));
//!
//!     // The hypergraph has 5 hyperedges.
//!     assert_eq!(graph.count_hyperedges(), 5);
//!
//!     // Get the hypergedges of a vertex.
//!     assert_eq!(graph.get_vertex_hyperedges(VertexIndex(0)), Ok(vec![first_hyperedge, second_hyperedge, third_hyperedge]));
//!     assert_eq!(graph.get_full_vertex_hyperedges(VertexIndex(0)), Ok(vec![vec![faarooq, ava, ghanda], vec![faarooq, ava, ghanda], vec![ewan, ava, bianca]]));
//!     
//!     // Get the intersection of some hyperedges.
//!     assert_eq!(graph.get_hyperedges_intersections(vec![second_hyperedge, third_hyperedge]), Ok(vec![ava]));
//!
//!     // Find a hyperedge containing a connection between two vertices.
//!     assert_eq!(graph.get_hyperedges_connecting(bianca, bianca), Ok(vec![fifth_hyperedge]));
//!
//!     // Get the adjacent vertices from a vertex.
//!     assert_eq!(graph.get_adjacent_vertices_from(VertexIndex(0)), Ok(vec![bianca, ghanda]));
//!
//!     // Get the adjacent vertices to a vertex.
//!     assert_eq!(graph.get_adjacent_vertices_to(VertexIndex(0)), Ok(vec![ewan, faarooq]));
//!
//!     // Find the shortest paths between some vertices.
//!     assert_eq!(graph.get_dijkstra_connections(faarooq, bianca), Ok(vec![faarooq, ava, bianca]));
//!
//!     // Update the weight of a vertex.
//!     graph.update_vertex_weight(ava, Vertex::new("Avā", 1))?;
//!     
//!     // Update the weight of a hyperedge.
//!     graph.update_hyperedge_weight(third_hyperedge, HyperEdge::new("share a viral video with a capybara", 1))?;
//!
//!     // Update the vertices of a hyperedge.
//!     graph.update_hyperedge_vertices(third_hyperedge, vec![ewan, ava, daena])?;
//!
//!     // Remove a hyperedge.
//!     graph.remove_hyperedge(first_hyperedge)?;
//!
//!     // Remove a vertex.
//!     graph.remove_vertex(ewan)?;
//!
//!     // Reverse a hyperedge.
//!     graph.reverse_hyperedge(fifth_hyperedge)?;
//!
//!     // Get the in-degree of a vertex.
//!     assert_eq!(graph.get_vertex_degree_in(ava), Ok(1));
//!
//!     // Get the out-degree of a vertex.
//!     assert_eq!(graph.get_vertex_degree_out(ghanda), Ok(0));
//!
//!     // Contract a hyperedge's vertices.
//!     graph.contract_hyperedge_vertices(fifth_hyperedge, vec![bianca, charles], bianca)?;
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

/// Public API.
pub mod core;

// Reexport of the public API.
#[doc(inline)]
pub use crate::core::*;
