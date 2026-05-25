use std::fmt::{
    Debug,
    Display,
    Formatter,
    Result,
};

use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    VertexIndex,
    VertexTrait,
    core::types::{
        AIndexMap,
        ARandomState,
    },
};

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "V: serde::Serialize, HE: serde::Serialize",
        deserialize = "V: serde::Deserialize<'de>, HE: serde::Deserialize<'de>"
    ))
)]
/// A directed hypergraph composed of generic vertices and hyperedges.
#[derive(Clone)]
pub struct Hypergraph<V, HE> {
    /// Vertices keyed by their stable index.
    /// Each entry holds the weight and the set of hyperedge indices that include this vertex.
    pub(crate) vertices: AIndexMap<VertexIndex, (V, crate::core::types::AIndexSet<HyperedgeIndex>)>,

    /// Hyperedges keyed by their stable index.
    /// Each entry holds the ordered vertex list and the weight.
    pub(crate) hyperedges: AIndexMap<HyperedgeIndex, (Vec<VertexIndex>, HE)>,

    /// Monotonically increasing counter used to generate unique [`VertexIndex`] values.
    pub(crate) vertices_count: usize,

    /// Monotonically increasing counter used to generate unique [`HyperedgeIndex`] values.
    pub(crate) hyperedges_count: usize,
}

impl<V, HE> Debug for Hypergraph<V, HE>
where
    V: Debug,
    HE: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Hypergraph")
            .field("vertices", &self.vertices)
            .field("hyperedges", &self.hyperedges)
            .finish_non_exhaustive()
    }
}

impl<V, HE> Display for Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut vertices: Vec<(VertexIndex, &V)> = self.vertices_iter().collect();
        vertices.sort_by_key(|(idx, _)| *idx);

        write!(f, "Hypergraph {{ vertices: [")?;
        for (i, (idx, weight)) in vertices.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", idx.0, weight)?;
        }

        write!(f, "], hyperedges: [")?;
        let mut hyperedges: Vec<(HyperedgeIndex, &HE)> = self.hyperedges_iter().collect();
        hyperedges.sort_by_key(|(idx, _)| *idx);

        for (i, (idx, weight)) in hyperedges.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {} [", idx.0, weight)?;
            if let Ok(vertex_indexes) = self.get_hyperedge_vertices(*idx) {
                for (j, v_idx) in vertex_indexes.iter().enumerate() {
                    if j > 0 {
                        write!(f, " → ")?;
                    }
                    if let Ok(v_weight) = self.get_vertex_weight(*v_idx) {
                        write!(f, "{v_weight}")?;
                    }
                }
            }
            write!(f, "]")?;
        }

        write!(f, "] }}")
    }
}

impl<V, HE> Default for Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    fn default() -> Self {
        Hypergraph::new()
    }
}

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns `true` if the hypergraph contains no vertices.
    ///
    /// Because hyperedges require at least one vertex to exist, an empty vertex
    /// set implies an empty hyperedge set as well.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Removes all vertices and hyperedges from the hypergraph.
    ///
    /// Both internal maps are emptied and the monotonic index counters are
    /// reset to zero, so the next insertion will start from index `0` again.
    pub fn clear(&mut self) {
        self.hyperedges.clear();
        self.vertices.clear();
        self.hyperedges_count = 0;
        self.vertices_count = 0;
    }

    /// Creates a new hypergraph with no allocation.
    #[must_use]
    pub fn new() -> Self {
        Hypergraph::with_capacity(0, 0)
    }

    /// Creates a new hypergraph with the specified capacity.
    #[must_use]
    pub fn with_capacity(vertices: usize, hyperedges: usize) -> Self {
        Hypergraph {
            vertices: AIndexMap::with_capacity_and_hasher(vertices, ARandomState::default()),
            hyperedges: AIndexMap::with_capacity_and_hasher(hyperedges, ARandomState::default()),
            vertices_count: 0,
            hyperedges_count: 0,
        }
    }
}
