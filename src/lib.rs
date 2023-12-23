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

#[doc(hidden)]
pub mod core;

// Reexport of the public API.
#[doc(inline)]
pub use crate::core::*;
