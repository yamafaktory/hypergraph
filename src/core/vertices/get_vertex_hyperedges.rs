use itertools::Itertools;

use crate::{
    errors::HypergraphError, HyperedgeIndex, HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Gets the hyperedges of a vertex as a vector of HyperedgeIndex.
    pub fn get_vertex_hyperedges(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        let internal_index = self.get_internal_vertex(vertex_index)?;

        let (_, hyperedges_index_set) = self
            .vertices
            .get_index(internal_index)
            .ok_or(HypergraphError::InternalVertexIndexNotFound(internal_index))?;

        self.get_hyperedges(hyperedges_index_set.clone().into_iter().collect_vec())
    }
}
