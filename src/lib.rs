#![deny(unsafe_code, nonstandard_style)]
#![forbid(rust_2018_idioms)]
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
//! struct Vertex<'a> {
//!     name: &'a str,
//! }
//!
//! impl<'a> Vertex<'a> {
//!     pub fn new(name: &'a str) -> Self {
//!         Vertex { name }
//!     }
//! }
//!
//! impl<'a> Display for Vertex<'a> {
//!     fn fmt(&self, f: &mut Formatter<'_>) -> Result {
//!         write!(f, "{}", self)
//!   }
//! }
//!
//! fn main() -> std::result::Result<(),  Box<dyn std::error::Error>> {
//!     let mut graph = Hypergraph::<Vertex<'_>, &str>::new();
//!
//!     // Add some vertices to the graph.
//!     let ava = graph.add_vertex(Vertex::new("Ava"))?;
//!     let bianca = graph.add_vertex(Vertex::new("Bianca"))?;
//!     let charles = graph.add_vertex(Vertex::new("Charles"))?;
//!     let daena = graph.add_vertex(Vertex::new("Daena"))?;
//!     let ewan = graph.add_vertex(Vertex::new("Ewan"))?;
//!     let faarooq = graph.add_vertex(Vertex::new("Faarooq"))?;
//!     let ghanda = graph.add_vertex(Vertex::new("Ghanda"))?;
//!  
//!     // Each vertex gets a unique index by insertion order.
//!     assert_eq!(ava, VertexIndex(0));
//!     assert_eq!(ghanda, VertexIndex(6));
//!
//!     // Get the weight of a vertex.
//!     assert_eq!(graph.get_vertex_weight(VertexIndex(0)), Ok(Vertex::new("Ava")));
//!     
//!     // The hypergraph has 7 vertices.
//!     assert_eq!(graph.count_vertices(), 7);
//!
//!     // Add some hyperedges to the graph.
//!     let first_hyperedge = graph.add_hyperedge(vec![faarooq, ava, ghanda], "share a viral video with a cat")?;
//!     let second_hyperedge = graph.add_hyperedge(vec![faarooq, ava, ghanda], "share a viral video with a dog")?;
//!     let third_hyperedge = graph.add_hyperedge(vec![ewan, ava, bianca], "share a viral video with a beaver")?;
//!     let fourth_hyperedge = graph.add_hyperedge(vec![daena], "play online")?;
//!     let fifth_hyperedge = graph.add_hyperedge(vec![ewan, charles, bianca, bianca, ewan], "pass the ball")?;
//!
//!     // Each hyperedge gets a unique index by insertion order.
//!     assert_eq!(first_hyperedge, HyperedgeIndex(0));
//!     assert_eq!(fifth_hyperedge, HyperedgeIndex(4));
//!
//!     // Get the weight of a hyperedge.
//!     assert_eq!(graph.get_hyperedge_weight(HyperedgeIndex(0)), Ok("share a viral video with a cat"));
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
//!     // Find the shortest paths between some vertices.
//!     assert_eq!(graph.get_dijkstra_connections(faarooq, bianca), Ok(vec![faarooq, ava, bianca]));
//!
//!     // Update the weight of a vertex.
//!     graph.update_vertex_weight(ava, Vertex::new("Avā"))?;
//!     
//!     // Update the weight of a hyperedge.
//!     graph.update_hyperedge_weight(third_hyperedge, "share a viral video with a capybara")?;
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
//!     Ok(())
//! }
//! ```

/// Public API.
pub mod core;

// Reexport of the public API.
#[doc(inline)]
pub use crate::core::*;
