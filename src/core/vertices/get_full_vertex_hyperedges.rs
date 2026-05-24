use rayon::prelude::*;

use crate::{
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns the vertex list of every hyperedge that includes `vertex_index`.
    ///
    /// Each element of the outer `Vec` is the ordered vertex list of one
    /// hyperedge, in the same order as returned by
    /// [`get_vertex_hyperedges`](Self::get_vertex_hyperedges).
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `vertex_index` does
    /// not exist.
    pub fn get_full_vertex_hyperedges(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>> {
        self.get_vertex_hyperedges(vertex_index).map(|hyperedges| {
            hyperedges
                .into_par_iter()
                .flat_map(|hyperedge_index| self.get_hyperedge_vertices(hyperedge_index))
                .collect()
        })
    }
}
