use std::fmt::{
    Display,
    Formatter,
    Result,
};

/// Vertex stable index representation as usize.
/// Uses the newtype index pattern.
/// <https://matklad.github.io/2018/06/04/newtype-index-pattern.html>
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[cfg(test)]
mod tests {
    use crate::{
        HyperedgeIndex,
        VertexIndex,
    };

    #[test]
    fn vertex_index_display() {
        assert_eq!(VertexIndex(7).to_string(), "7");
    }

    #[test]
    fn hyperedge_index_display() {
        assert_eq!(HyperedgeIndex(42).to_string(), "42");
    }

    #[test]
    fn vertex_index_from_usize() {
        assert_eq!(VertexIndex::from(3usize), VertexIndex(3));
    }

    #[test]
    fn hyperedge_index_from_usize() {
        assert_eq!(HyperedgeIndex::from(9usize), HyperedgeIndex(9));
    }

    #[test]
    fn vertex_index_ordering() {
        assert!(VertexIndex(0) < VertexIndex(1));
    }

    #[test]
    fn hyperedge_index_ordering() {
        assert!(HyperedgeIndex(2) > HyperedgeIndex(1));
    }
}
