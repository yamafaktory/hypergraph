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
        self.vertices.iter().map(|(&idx, (weight, _))| (idx, weight))
    }

    /// Returns an iterator over all hyperedges as `(HyperedgeIndex, &HE)` pairs, in insertion order.
    #[must_use = "the iterator is lazy and must be consumed"]
    pub fn hyperedges_iter(&self) -> impl Iterator<Item = (HyperedgeIndex, &HE)> + '_ {
        self.hyperedges.iter().map(|(&idx, (_, weight))| (idx, weight))
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
