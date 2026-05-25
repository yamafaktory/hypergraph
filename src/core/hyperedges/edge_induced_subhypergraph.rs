use ahash::{
    AHashMap,
    AHashSet,
};

use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    core::types::{
        AIndexSet,
        ARandomState,
    },
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns a new hypergraph containing only the specified hyperedges and
    /// the vertices that appear in at least one of them.
    ///
    /// Vertex stable indices are reassigned from `0` in sorted order of the
    /// original indices. Hyperedge indices are reassigned from `0` in the
    /// order given by `edges`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if any index in
    /// `edges` does not exist.
    pub fn get_edge_induced_subhypergraph(
        &self,
        edges: &[HyperedgeIndex],
    ) -> Result<Hypergraph<V, HE>, HypergraphError<V, HE>> {
        // Validate all edges and collect their vertex lists.
        let he_data: Vec<(HyperedgeIndex, Vec<VertexIndex>, HE)> = edges
            .iter()
            .map(|&hei| {
                let (verts, w) = self
                    .hyperedges
                    .get(&hei)
                    .ok_or(HypergraphError::HyperedgeIndexNotFound(hei))?;
                Ok((hei, verts.clone(), *w))
            })
            .collect::<Result<Vec<_>, HypergraphError<V, HE>>>()?;

        // Collect the set of vertices that appear in at least one of the edges.
        let mut vertex_set: AHashSet<VertexIndex> = AHashSet::new();
        for (_, verts, _) in &he_data {
            for &v in verts {
                vertex_set.insert(v);
            }
        }

        // Sort the vertex set for deterministic ordering.
        let mut sorted_vertices: Vec<VertexIndex> = vertex_set.into_iter().collect();
        sorted_vertices.sort();

        // Build the new graph.
        let mut new_graph = Hypergraph::new();

        // Build old→new vertex mapping.
        let mut old_to_new: AHashMap<VertexIndex, VertexIndex> = AHashMap::new();
        for &old_v in &sorted_vertices {
            let weight = self
                .vertices
                .get(&old_v)
                .map(|(w, _)| *w)
                .ok_or(HypergraphError::VertexIndexNotFound(old_v))?;
            let new_v_idx = VertexIndex(new_graph.vertices_count);
            new_graph.vertices_count += 1;
            let he_set = AIndexSet::with_capacity_and_hasher(0, ARandomState::default());
            new_graph.vertices.insert(new_v_idx, (weight, he_set));
            old_to_new.insert(old_v, new_v_idx);
        }

        // Insert hyperedges in the given order, updating back-refs.
        for (_, old_verts, weight) in &he_data {
            let new_verts: Vec<VertexIndex> = old_verts.iter().map(|v| old_to_new[v]).collect();
            let new_he_idx = HyperedgeIndex(new_graph.hyperedges_count);
            new_graph.hyperedges_count += 1;
            let unique: AIndexSet<VertexIndex> = new_verts.iter().copied().collect();
            for v in &unique {
                if let Some((_, he_set)) = new_graph.vertices.get_mut(v) {
                    he_set.insert(new_he_idx);
                }
            }
            new_graph
                .hyperedges
                .insert(new_he_idx, (new_verts, *weight));
        }

        Ok(new_graph)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        HyperedgeIndex,
        core::test_support::build,
    };

    #[test]
    fn edge_induced_subhypergraph_basic() {
        let (g, _, [e0, e1, _]) = build();
        let sub = g.get_edge_induced_subhypergraph(&[e0, e1]).unwrap();
        // e0=[v0,v1], e1=[v1,v2] → 3 unique vertices, 2 edges
        assert_eq!(sub.count_vertices(), 3);
        assert_eq!(sub.count_hyperedges(), 2);
    }

    #[test]
    fn edge_induced_subhypergraph_invalid_edge() {
        let (g, _, _) = build();
        assert!(
            g.get_edge_induced_subhypergraph(&[HyperedgeIndex(999)])
                .is_err()
        );
    }

    #[test]
    fn edge_induced_subhypergraph_single_edge() {
        let (g, _, [e0, _, _]) = build();
        let sub = g.get_edge_induced_subhypergraph(&[e0]).unwrap();
        assert_eq!(sub.count_vertices(), 2);
        assert_eq!(sub.count_hyperedges(), 1);
    }
}
