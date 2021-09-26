use crate::{HyperedgeIndex, Hypergraph, SharedTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    // This private method is infallible since adding the same hyperedge
    // will return the existing index.
    pub(crate) fn add_hyperedge_index(&mut self, internal_index: usize) -> HyperedgeIndex {
        match self.hyperedges_mapping.left.get(&internal_index) {
            Some(hyperedge_index) => *hyperedge_index,
            None => {
                let hyperedge_index = HyperedgeIndex(self.hyperedges_count);

                if self
                    .hyperedges_mapping
                    .left
                    .insert(internal_index, hyperedge_index)
                    .is_none()
                {
                    // Update the counter only for the first insertion.
                    self.hyperedges_count += 1;
                }

                self.hyperedges_mapping
                    .right
                    .insert(hyperedge_index, internal_index);

                hyperedge_index
            }
        }
    }
}
