#![deny(unsafe_code, nonstandard_style)]
#![forbid(rust_2018_idioms)]
#![warn(missing_debug_implementations, missing_docs, unreachable_pub)]

//! Hypergraph is data structure library to generate directed [hypergraphs](https://en.wikipedia.org/wiki/Hypergraph).
//!
//! A hypergraph is a generalization of a graph in which a hyperedge can join any number of vertices.
//! ## Features
//!
//! This library enables you to:
//! - represent **non-simple** hypergraphs with two or more hyperedges containing the same set of vertices with different weights
//! - represent **self-loops** —i.e., hyperedges containing vertices directed to themselves one or more times
//! - represent **unaries** —i.e., hyperedges containing a unique vertex

/// Public API.
pub mod core;

// Reexport of the public API.
#[doc(inline)]
pub use crate::core::*;
