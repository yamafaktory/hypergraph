use std::fmt::{Display, Formatter, Result};

/// Vertex stable index representation as usize.
/// Uses the newtype index pattern.
/// <https://matklad.github.io/2018/06/04/newtype-index-pattern.html>
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VertexIndex(pub usize);

impl Display for VertexIndex {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", self.0)
    }
}

impl From<usize> for VertexIndex {
    fn from(index: usize) -> Self {
        VertexIndex(index)
    }
}

/// Hyperedge stable index representation as usize.
/// Uses the newtype index pattern.
/// <https://matklad.github.io/2018/06/04/newtype-index-pattern.html>
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HyperedgeIndex(pub usize);

impl Display for HyperedgeIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for HyperedgeIndex {
    fn from(index: usize) -> Self {
        HyperedgeIndex(index)
    }
}
