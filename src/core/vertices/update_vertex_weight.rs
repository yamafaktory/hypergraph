use crate::{HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait, errors::HypergraphError};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Updates the weight of a vertex by index.
    ///
    /// The new weight need not be unique — multiple vertices may carry the same
    /// weight value. Returns [`HypergraphError::VertexWeightUnchanged`] if
    /// `weight` equals the current weight (no-op guard).
    pub fn update_vertex_weight(
        &mut self,
        vertex_index: VertexIndex,
        weight: V,
    ) -> Result<(), HypergraphError<V, HE>> {
        let internal_index = self.get_internal_vertex(vertex_index)?;

        let (previous_weight, index_set) = self
            .vertices
            .get(internal_index)
            .map(|(w, s)| (*w, s.clone()))
            .ok_or(HypergraphError::InternalVertexIndexNotFound(internal_index))?;

        if weight == previous_weight {
            return Err(HypergraphError::VertexWeightUnchanged {
                index: vertex_index,
                weight,
            });
        }

        // Append the new entry at the end, then swap-remove the old position so
        // that the new entry lands at `internal_index`. The BiHashMap mapping is
        // unchanged — `internal_index` still maps to `vertex_index`.
        self.vertices.push((weight, index_set));
        self.vertices.swap_remove(internal_index);

        Ok(())
    }
}
