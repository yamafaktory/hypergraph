mod hypergraph_impl;
#[cfg(feature = "persistence")]
mod persistent_impl;
mod trait_def;

pub use trait_def::HypergraphQuery;
