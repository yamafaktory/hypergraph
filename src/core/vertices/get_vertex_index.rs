use crate::{HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns the stable index of the vertex with the given weight, or `None`
    /// if no such vertex exists.
    ///
    /// This is the reverse of [`get_vertex_weight`](Self::get_vertex_weight).
    pub fn get_vertex_index(&self, weight: V) -> Option<VertexIndex> {
        let internal = self.vertices.get_index_of(&weight)?;
        self.vertices_mapping.left.get(&internal).copied()
    }
}
