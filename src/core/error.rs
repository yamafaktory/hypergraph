use thiserror::Error;

use crate::{HyperedgeIndex, SharedTrait, VertexIndex};

/// Enumeration of all the possible errors.
#[derive(Eq, PartialEq, Clone, Copy, Debug, Error)]
pub enum HypergraphError<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Error when an HyperedgeIndex is not found.
    #[error("HyperedgeIndex {0} not found")]
    HyperedgeIndexNotFound(HyperedgeIndex),

    /// Error when an internal hyperedge index is not found.
    #[error("Internal hyperedge index {0} not found")]
    InternalHyperedgeIndexNotFound(usize),

    /// Error when an hyperedge weight is not found.
    #[error("Hyperedge weight {0} not found")]
    HyperedgeWeightNotFound(HE),

    /// Error when a VertexIndex is not found.
    #[error("VertexIndex {0} not found")]
    VertexIndexNotFound(VertexIndex),

    /// Error when a an internal vertex index is not found.
    #[error("Internal vertex index {0} not found")]
    InternalVertexIndexNotFound(usize),

    /// Error when a vertex weight is not found.
    #[error("Vertex weight {0} not found")]
    VertexWeightNotFound(V),
}
