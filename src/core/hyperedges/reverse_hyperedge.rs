use rayon::prelude::*;

use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    Hypergraph,
    VertexTrait,
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Reverses the vertex order of the hyperedge at `hyperedge_index`.
    ///
    /// This inverts the direction of the hyperedge without changing which
    /// vertices it connects. The weight is unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist.
    pub fn reverse_hyperedge(
        &mut self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        let vertices = self.get_hyperedge_vertices(hyperedge_index)?;

        self.update_hyperedge_vertices(hyperedge_index, vertices.into_par_iter().rev().collect())
    }
}
