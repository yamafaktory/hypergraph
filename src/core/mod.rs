#[cfg(feature = "persistence")]
#[doc(hidden)]
pub mod disk;
#[doc(hidden)]
pub mod errors;
#[doc(hidden)]
pub mod hyperedges;
pub(crate) mod hypergraph;
pub(crate) mod indexes;
#[doc(hidden)]
pub mod iterator;
pub mod query;
pub(crate) mod shared;
#[cfg(test)]
pub(crate) mod test_support;
pub(crate) mod traits;
pub(crate) mod types;
#[doc(hidden)]
pub mod vertices;

#[cfg(feature = "persistence")]
pub use crate::core::disk::PersistentHypergraph;
pub use crate::core::{
    hypergraph::Hypergraph,
    indexes::{
        HyperedgeIndex,
        VertexIndex,
    },
    iterator::{
        HypergraphBorrowingIterator,
        HypergraphIterator,
    },
    query::{
        CentralityScores,
        HypergraphQuery,
        NestednessEntry,
    },
    traits::{
        HyperedgeTrait,
        VertexTrait,
    },
};
