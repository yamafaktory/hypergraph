mod hypergraph_impl;
mod lookups;
mod paths;
mod projections;
mod properties;
mod structural;
mod trait_def;
mod traversal;

#[cfg(feature = "persistence")]
mod persistent_impl;

pub use trait_def::HypergraphQuery;
