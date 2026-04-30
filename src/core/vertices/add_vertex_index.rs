use crate::{HyperedgeTrait, Hypergraph, VertexIndex, VertexTrait};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    // This private method is infallible since adding the same vertex
    // will return the existing index.
    pub(crate) fn add_vertex_index(&mut self, internal_index: usize) -> VertexIndex {
        if let Some(vertex_index) = self.vertices_mapping.left.get(&internal_index) {
            *vertex_index
        } else {
            let vertex_index = VertexIndex(self.vertices_count);

            if self
                .vertices_mapping
                .left
                .insert(internal_index, vertex_index)
                .is_none()
            {
                // Update the counter only for the first insertion.
                self.vertices_count += 1;
            }

            self.vertices_mapping
                .right
                .insert(vertex_index, internal_index);

            vertex_index
        }
    }
}
