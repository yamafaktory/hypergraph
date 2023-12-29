use thiserror::Error;

/// Enumeration of all the possible errors.
#[derive(Debug, Error)]
pub enum HypergraphError {
    /// Error when a vertex is inserted.
    #[error("Vertex couldn't be inserted")]
    VertexInsertion,
    /// Error when an hyperedge is inserted.
    #[error("Hyperedge couldn't be inserted")]
    HyperedgeInsertion,
    //
    //
    //
    //
    //
    // /// Error when a HyperedgeIndex was not found.
    // #[error("HyperedgeIndex {0} was not found")]
    // HyperedgeIndexNotFound(HyperedgeIndex),
    //
    // /// Error when an internal hyperedge index was not found.
    // #[error("Internal hyperedge index {0} was not found")]
    // InternalHyperedgeIndexNotFound(usize),
    //
    // /// Error when a hyperedge weight was not found.
    // #[error("Hyperedge weight {0} was not found")]
    // HyperedgeWeightNotFound(HE),
    //
    // /// Error when a hyperedge is updated with the same weight.
    // #[error("HyperedgeIndex {index:?} weight {weight:?} is unchanged (no-op)")]
    // HyperedgeWeightUnchanged { index: HyperedgeIndex, weight: HE },
    //
    // /// Error when a hyperedge is updated with the same vertices.
    // #[error("HyperedgeIndex {0} vertices are unchanged (no-op)")]
    // HyperedgeVerticesUnchanged(HyperedgeIndex),
    //
    // /// Error when a hyperedge is updated with no vertices.
    // #[error("HyperedgeIndex {0} vertices are missing")]
    // HyperedgeCreationNoVertices(HE),
    //
    // /// Error when a hyperedge is updated with no vertices.
    // #[error("HyperedgeIndex {0} vertices are missing")]
    // HyperedgeUpdateNoVertices(HyperedgeIndex),
    //
    // /// Error when a hyperedge doesn't contain some vertices.
    // #[error("HyperedgeIndex {index:?} does not include vertices {vertices:?}")]
    // HyperedgeVerticesIndexesNotFound {
    //     index: HyperedgeIndex,
    //     vertices: Vec<VertexIndex>,
    // },
    //
    // /// Error when a hyperedge contraction is invalid.
    // #[error(
    //     "HyperedgeIndex {index:?} contraction of vertices {vertices:?} into vertex {target:?} is invalid"
    // )]
    // HyperedgeInvalidContraction {
    //     index: HyperedgeIndex,
    //     target: VertexIndex,
    //     vertices: Vec<VertexIndex>,
    // },
    //
    // /// Error when a hyperedge is updated with the weight of another one.
    // #[error("Hyperedge weight {0} was already assigned")]
    // HyperedgeWeightAlreadyAssigned(HE),
    //
    // /// Error when trying to get the intersections of less than two hyperedges.
    // #[error("At least two hyperedges must be provided to find their intersections")]
    // HyperedgesInvalidIntersections,
    //
    // /// Error when trying to join less than two hyperedges.
    // #[error("At least two hyperedges must be provided to be joined")]
    // HyperedgesInvalidJoin,
    //
    // /// Error when a VertexIndex was not found.
    // #[error("VertexIndex {0} was not found")]
    // VertexIndexNotFound(VertexIndex),
    //
    // /// Error when a an internal vertex index was not found.
    // #[error("Internal vertex index {0} was not found")]
    // InternalVertexIndexNotFound(usize),
    //
    // /// Error when a vertex weight was not found.
    // #[error("Vertex weight {0} was not found")]
    // VertexWeightNotFound(V),
    //
    // /// Error when a vertex weight is updated with the same value.
    // #[error("VertexIndex {index:?} weight {weight:?} unchanged (no-op)")]
    // VertexWeightUnchanged { index: VertexIndex, weight: V },
    //
    // /// Error when a vertex weight is updated with the weight of another one.
    // #[error("Vertex weight {0} was already assigned")]
    // VertexWeightAlreadyAssigned(V),
}
