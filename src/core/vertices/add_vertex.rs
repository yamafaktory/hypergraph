use std::fmt::{Debug, Display};

use crate::{errors::HypergraphError, Hypergraph};

impl<V, HE> Hypergraph<V, HE>
where
    V: Clone + Debug + Display,
    HE: Clone + Debug + Display,
{
    /// Adds a vertex with a custom weight to the hypergraph.
    /// Returns the index of the vertex.
    pub fn add_vertex(&mut self, weight: V) -> Result<VertexIndex, HypergraphError<V, HE>> {
        self.cache
        // // Return an error if the weight is already assigned to another vertex.
        // if self.vertices.contains_key(&weight) {
        //     return Err(HypergraphError::VertexWeightAlreadyAssigned(weight));
        // }
        //
        // self.vertices
        //     .entry(weight)
        //     .or_insert(AIndexSet::with_capacity_and_hasher(
        //         0,
        //         ARandomState::default(),
        //     ));
        //
        // let internal_index = self
        //     .vertices
        //     .get_index_of(&weight)
        //     // This safe-check should always pass since the weight has been
        //     // inserted upfront.
        //     .ok_or(HypergraphError::VertexWeightNotFound(weight))?;
        //
        // Ok(self.add_vertex_index(internal_index))
    }
}
