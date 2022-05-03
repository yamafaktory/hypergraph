use rayon::prelude::*;

use crate::{errors::HypergraphError, HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    // Private method to get a vector of VertexIndex from a vector of internal indexes.
    pub(crate) fn get_vertices(
        &self,
        vertices: Vec<usize>,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        vertices
            .par_iter()
            .map(|vertex_index| self.get_vertex(*vertex_index))
            .collect()
    }
}
