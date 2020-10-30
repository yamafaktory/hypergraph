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
//!
//! ```
//! - Deal with out of bound indexes

/// Public API.
pub mod core;
mod dot;
mod private;

// Reexport of the public API.
pub use crate::core::*;
