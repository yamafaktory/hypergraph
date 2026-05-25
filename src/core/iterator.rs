use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
};

impl<V, HE> IntoIterator for Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    type Item = (HE, Vec<V>);
    type IntoIter = HypergraphIterator<V, HE>;

    fn into_iter(self) -> Self::IntoIter {
        HypergraphIterator {
            hypergraph: self,
            index: 0,
        }
    }
}

impl<'a, V, HE> IntoIterator for &'a Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    type Item = (&'a HE, Vec<&'a V>);
    type IntoIter = HypergraphBorrowingIterator<'a, V, HE>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns a borrowing iterator over the hyperedges as `(&HE, Vec<&V>)` tuples.
    #[must_use]
    pub fn iter(&self) -> HypergraphBorrowingIterator<'_, V, HE> {
        HypergraphBorrowingIterator {
            hypergraph: self,
            index: 0,
        }
    }

    /// Returns an iterator over all vertices as `(VertexIndex, &V)` pairs, in insertion order.
    #[must_use = "the iterator is lazy and must be consumed"]
    pub fn vertices_iter(&self) -> impl Iterator<Item = (VertexIndex, &V)> + '_ {
        self.vertices
            .iter()
            .map(|(&idx, (weight, _))| (idx, weight))
    }

    /// Returns an iterator over all hyperedges as `(HyperedgeIndex, &HE)` pairs, in insertion order.
    #[must_use = "the iterator is lazy and must be consumed"]
    pub fn hyperedges_iter(&self) -> impl Iterator<Item = (HyperedgeIndex, &HE)> + '_ {
        self.hyperedges
            .iter()
            .map(|(&idx, (_, weight))| (idx, weight))
    }
}

/// A consuming iterator over a [`Hypergraph`].
#[derive(Debug)]
pub struct HypergraphIterator<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    hypergraph: Hypergraph<V, HE>,
    index: usize,
}

impl<V, HE> Iterator for HypergraphIterator<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    type Item = (HE, Vec<V>);

    fn next(&mut self) -> Option<Self::Item> {
        let (_, (vertices, weight)) = self.hypergraph.hyperedges.get_index(self.index)?;
        self.index += 1;

        let vertex_weights: Option<Vec<V>> = vertices
            .iter()
            .map(|v_idx| self.hypergraph.vertices.get(v_idx).map(|(w, _)| *w))
            .collect();

        vertex_weights.map(|vw| (*weight, vw))
    }
}

/// A borrowing iterator over a [`Hypergraph`].
#[derive(Debug)]
pub struct HypergraphBorrowingIterator<'s, V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    hypergraph: &'s Hypergraph<V, HE>,
    index: usize,
}

impl<'s, V, HE> Iterator for HypergraphBorrowingIterator<'s, V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    type Item = (&'s HE, Vec<&'s V>);

    fn next(&mut self) -> Option<Self::Item> {
        let (_, (vertices, weight)) = self.hypergraph.hyperedges.get_index(self.index)?;
        self.index += 1;

        let vertex_weights: Option<Vec<&'s V>> = vertices
            .iter()
            .map(|v_idx| self.hypergraph.vertices.get(v_idx).map(|(w, _)| w))
            .collect();

        vertex_weights.map(|vw| (weight, vw))
    }
}

#[cfg(test)]
mod tests {
    use crate::core::test_support::{
        E,
        W,
        build,
    };

    #[test]
    fn iter_yields_hyperedge_count() {
        let (g, _, _) = build();
        assert_eq!(g.iter().count(), 3);
    }

    #[test]
    fn iter_yields_correct_vertex_weights() {
        let (g, _, _) = build();
        let all_verts: Vec<_> = g.iter().flat_map(|(_, verts)| verts).collect();
        assert_eq!(all_verts.len(), 6); // 3 hyperedges × 2 vertices each
    }

    #[test]
    fn vertices_iter_yields_all() {
        let (g, [v0, v1, v2, v3], _) = build();
        let mut indices: Vec<_> = g.vertices_iter().map(|(idx, _)| idx).collect();
        indices.sort();
        assert_eq!(indices, vec![v0, v1, v2, v3]);
    }

    #[test]
    fn vertices_iter_weight_matches() {
        let (g, [v0, _, _, _], _) = build();
        let weight = g.vertices_iter().find(|(idx, _)| *idx == v0).unwrap().1;
        assert_eq!(*weight, W(0));
    }

    #[test]
    fn hyperedges_iter_yields_all() {
        let (g, _, [e0, e1, e2]) = build();
        let mut indices: Vec<_> = g.hyperedges_iter().map(|(idx, _)| idx).collect();
        indices.sort();
        assert_eq!(indices, vec![e0, e1, e2]);
    }

    #[test]
    fn hyperedges_iter_weight_matches() {
        let (g, _, [e0, _, _]) = build();
        let weight = g.hyperedges_iter().find(|(idx, _)| *idx == e0).unwrap().1;
        assert_eq!(*weight, E(1));
    }

    #[test]
    fn into_iter_consumes_graph() {
        let (g, _, _) = build();
        let items: Vec<_> = g.into_iter().collect();
        assert_eq!(items.len(), 3);
        assert!(items.iter().all(|(_, verts)| verts.len() == 2));
    }
}
