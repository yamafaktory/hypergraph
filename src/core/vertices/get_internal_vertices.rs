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
    // Private method to get the internal vertices from a vector of VertexIndex.
    pub(crate) fn get_internal_vertices<R: AsRef<Vec<VertexIndex>>>(
        &self,
        vertices: R,
    ) -> Result<Vec<usize>, HypergraphError<V, HE>> {
        vertices
            .as_ref()
            .par_iter()
            .map(|vertex_index| self.get_internal_vertex(*vertex_index))
            .collect()
    }
}
