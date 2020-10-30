#![forbid(rust_2018_idioms)]
#![warn(missing_debug_implementations, missing_docs, unreachable_pub)]
#![deny(unsafe_code, nonstandard_style)]

//! Hypergraph is an open-source library built in Rust to represent directed hypergraphs.
//! ## Example
//! ```
//!use hypergraph::Hypergraph;
//!
//!// Create a new hypergraph.
//!let mut graph = Hypergraph::<&str, &str>::new();
//!
//!// Add two vertices.
//!assert_eq!(graph.add_vertex("foo"), 0);
//!assert_eq!(graph.add_vertex("bar"), 1);
//!
//!// Add three hyperedges.
//!assert_eq!(graph.add_hyperedge(&[0], "hyperedge with a unary {foo}"), (0, 0));
//!assert_eq!(graph.add_hyperedge(&[0, 1, 1], "hyperedge with a self-loop {foo, bar, bar}"), (1, 0));
//!assert_eq!(graph.add_hyperedge(&[0, 1, 1], "same hyperedge with a self-loop {foo, bar, bar}"), (1, 1));
//! ```
//! - Deal with out of bound indexes

/// Public API.
pub mod core;
mod dot;
mod private;

// Reexport of the public API.
pub use crate::core::*;
