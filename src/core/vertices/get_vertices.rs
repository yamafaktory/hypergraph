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
    // Private method to get a vector of VertexIndex from a vector of internal indexes.
    pub(crate) fn get_vertices(
        &self,
        vertices: &[usize],
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        vertices
            .par_iter()
            .map(|vertex_index| self.get_vertex(*vertex_index))
            .collect()
    }
}
