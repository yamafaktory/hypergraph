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
pub(crate) mod shared;
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
    traits::{
        HyperedgeTrait,
        VertexTrait,
    },
};
