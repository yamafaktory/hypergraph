use thiserror::Error;

use crate::{
    HyperedgeIndex,
    VertexIndex,
};

/// Enumeration of all the possible errors.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum HypergraphError<V, HE>
where
    V: Copy + Eq,
    HE: Copy + Eq,
{
    /// Error when a `HyperedgeIndex` was not found.
    #[error("HyperedgeIndex {0} was not found")]
    HyperedgeIndexNotFound(HyperedgeIndex),

    /// Error when an internal hyperedge index was not found.
    #[error("Internal hyperedge index {0} was not found")]
    InternalHyperedgeIndexNotFound(usize),

    /// Error when a hyperedge weight was not found.
    #[error("Hyperedge weight {0} was not found")]
    HyperedgeWeightNotFound(HE),

    /// Error when a hyperedge is updated with the same weight.
    #[error("HyperedgeIndex {index:?} weight {weight:?} is unchanged (no-op)")]
    HyperedgeWeightUnchanged { index: HyperedgeIndex, weight: HE },

    /// Error when a hyperedge is updated with the same vertices.
    #[error("HyperedgeIndex {0} vertices are unchanged (no-op)")]
    HyperedgeVerticesUnchanged(HyperedgeIndex),

    /// Error when a hyperedge cannot be created because no vertices were provided.
    #[error("Hyperedge with weight {0} cannot be created: no vertices provided")]
    HyperedgeCreationNoVertices(HE),

    /// Error when a hyperedge cannot be updated because no vertices were provided.
    #[error("HyperedgeIndex {0} cannot be updated: no vertices provided")]
    HyperedgeUpdateNoVertices(HyperedgeIndex),

    /// Error when a hyperedge doesn't contain some vertices.
    #[error("HyperedgeIndex {index:?} does not include vertices {vertices:?}")]
    HyperedgeVerticesIndexesNotFound {
        index: HyperedgeIndex,
        vertices: Vec<VertexIndex>,
    },

    /// Error when a hyperedge contraction is invalid.
    #[error(
        "HyperedgeIndex {index:?} contraction of vertices {vertices:?} into vertex {target:?} is invalid"
    )]
    HyperedgeInvalidContraction {
        index: HyperedgeIndex,
        target: VertexIndex,
        vertices: Vec<VertexIndex>,
    },

    /// Kept for API compatibility.
    ///
    /// Previously returned when a hyperedge was added or updated with a weight
    /// already used by another hyperedge. The library no longer enforces
    /// hyperedge-weight uniqueness, so this variant is never produced by any
    /// public method.
    #[error("Hyperedge weight {0} was already assigned")]
    HyperedgeWeightAlreadyAssigned(HE),

    /// Error when trying to get the intersections of less than two hyperedges.
    #[error("At least two hyperedges must be provided to find their intersections")]
    HyperedgesInvalidIntersections,

    /// Error when trying to join less than two hyperedges.
    #[error("At least two hyperedges must be provided to be joined")]
    HyperedgesInvalidJoin,

    /// Error when a `VertexIndex` was not found.
    #[error("VertexIndex {0} was not found")]
    VertexIndexNotFound(VertexIndex),

    /// Error when a an internal vertex index was not found.
    #[error("Internal vertex index {0} was not found")]
    InternalVertexIndexNotFound(usize),

    /// Kept for API compatibility.
    ///
    /// Previously used as an internal safe-check during vertex insertion.
    /// No longer produced by any public method.
    #[error("Vertex weight {0} was not found")]
    VertexWeightNotFound(V),

    /// Error when a vertex weight is updated with the same value.
    #[error("VertexIndex {index:?} weight {weight:?} unchanged (no-op)")]
    VertexWeightUnchanged { index: VertexIndex, weight: V },

    /// Kept for API compatibility.
    ///
    /// Previously returned when a vertex was added or updated with a weight
    /// already used by another vertex. The library no longer enforces
    /// vertex-weight uniqueness, so this variant is never produced by any
    /// public method.
    #[error("Vertex weight {0} was already assigned")]
    VertexWeightAlreadyAssigned(V),

    /// Error when a hyperedge insert position is out of bounds.
    #[error("HyperedgeIndex {index:?} insert position {position} is out of bounds")]
    HyperedgeInsertIndexOutOfBounds {
        index: HyperedgeIndex,
        position: usize,
    },

    /// Error when a hyperedge split position is out of bounds.
    #[error("HyperedgeIndex {index:?} split position {at} is out of bounds")]
    HyperedgeSplitIndexOutOfBounds { index: HyperedgeIndex, at: usize },

    /// Error when `split_vertex` is called with an empty hyperedge list.
    #[error("split_vertex requires at least one hyperedge")]
    HyperedgesInvalidSplit,

    /// Error when `merge_vertices` is called with fewer than two vertices.
    #[error("merge_vertices requires at least two vertices")]
    NotEnoughVerticesProvided,

    /// Error when the hypergraph contains a cycle and a topological sort is requested.
    #[error("Hypergraph contains a cycle and cannot be topologically sorted")]
    HypergraphContainsCycle,

    /// Error from the persistent storage backend.
    #[error("Storage error: {0}")]
    StorageError(String),
}
