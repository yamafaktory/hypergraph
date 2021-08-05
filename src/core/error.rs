use thiserror::Error;

use crate::{HyperedgeIndex, SharedTrait, VertexIndex};

/// Enumeration of all the possible errors.
#[derive(Eq, PartialEq, Clone, Copy, Debug, Error)]
pub enum HypergraphError<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Error when a HyperedgeIndex was not found.
    #[error("HyperedgeIndex {0} was not found")]
    HyperedgeIndexNotFound(HyperedgeIndex),

    /// Error when an internal hyperedge index was not found.
    #[error("Internal hyperedge index {0} was not found")]
    InternalHyperedgeIndexNotFound(usize),

    /// Error when a hyperedge weight was not found.
    #[error("Hyperedge weight {0} was not found")]
    HyperedgeWeightNotFound(HE),

    /// Error when a hyperedge weight is updated with the same value.
    #[error("HyperedgeIndex {0} weight unchanged (no-op)")]
    HyperedgeWeightUnchanged(HyperedgeIndex, HE),

    /// Error when a hyperedge weight is updated with the weight of another one.
    #[error("Hyperedge weight {0} was already assigned")]
    HyperedgeWeightAlreadyAssigned(HE),

    /// Error when trying to get the intersections of less than two hyperedges.
    #[error("At least two hyperedges must be provided to find their intersections")]
    HyperedgesIntersections,

    /// Error when a VertexIndex was not found.
    #[error("VertexIndex {0} was not found")]
    VertexIndexNotFound(VertexIndex),

    /// Error when a an internal vertex index was not found.
    #[error("Internal vertex index {0} was not found")]
    InternalVertexIndexNotFound(usize),

    /// Error when a vertex weight was not found.
    #[error("Vertex weight {0} was not found")]
    VertexWeightNotFound(V),

    /// Error when a vertex weight is updated with the same value.
    #[error("VertexIndex {0} weight unchanged (no-op)")]
    VertexWeightUnchanged(VertexIndex, V),

    /// Error when a vertex weight is updated with the weight of another one.
    #[error("Vertex weight {0} was already assigned")]
    VertexWeightAlreadyAssigned(V),
}
