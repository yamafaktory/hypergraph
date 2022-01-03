use crate::{errors::HypergraphError, HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Gets the hyperedges of a vertex as a vector of vectors of VertexIndex.
    pub fn get_full_vertex_hyperedges(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>> {
        self.get_vertex_hyperedges(vertex_index).map(|hyperedges| {
            hyperedges
                .into_iter()
                .flat_map(|hyperedge_index| self.get_hyperedge_vertices(hyperedge_index))
                .collect()
        })
    }
}
