use crate::{HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns the stable indexes of all vertices whose weight equals `weight`.
    ///
    /// Because vertex weights are not required to be unique, multiple indexes
    /// may be returned. Returns an empty `Vec` if no match is found.
    ///
    /// This is the reverse of [`get_vertex_weight`](Self::get_vertex_weight).
    #[must_use]
    pub fn get_vertex_index(&self, weight: V) -> Vec<VertexIndex> {
        self.vertices
            .iter()
            .enumerate()
            .filter_map(|(internal, (w, _))| {
                (*w == weight).then(|| self.vertices_mapping.left.get(&internal).copied())?
            })
            .collect()
    }
}
