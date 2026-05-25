use std::{
    fmt::{
        Debug,
        Display,
    },
    hash::Hash,
};

/// Trait bound required for vertex weights.
///
/// Any type that implements `Copy + Debug + Display + Eq + Hash + Send + Sync`
/// satisfies this trait automatically via the blanket impl. You do not need to
/// implement it manually.
pub trait VertexTrait: Copy + Debug + Display + Eq + Hash + Send + Sync {}

impl<T> VertexTrait for T where T: Copy + Debug + Display + Eq + Hash + Send + Sync {}

/// Trait bound required for hyperedge weights.
///
/// In addition to [`VertexTrait`], a hyperedge weight must implement
/// `Into<usize>` so that its value can be used as a numeric cost in
/// shortest-path algorithms. The blanket impl covers any type that already
/// satisfies both constraints.
pub trait HyperedgeTrait: VertexTrait + Into<usize> {}

impl<T> HyperedgeTrait for T where T: VertexTrait + Into<usize> {}
