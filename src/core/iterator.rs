use crate::{
    HyperedgeIndex,
    HyperedgeKey,
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
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
            .enumerate()
            .filter_map(|(internal, (weight, _))| {
                self.vertices_mapping
                    .left
                    .get(&internal)
                    .copied()
                    .map(|stable| (stable, weight))
            })
    }

    /// Returns an iterator over all hyperedges as `(HyperedgeIndex, &HE)` pairs, in insertion order.
    #[must_use = "the iterator is lazy and must be consumed"]
    pub fn hyperedges_iter(&self) -> impl Iterator<Item = (HyperedgeIndex, &HE)> + '_ {
        self.hyperedges
            .iter()
            .enumerate()
            .filter_map(|(internal, key)| {
                self.hyperedges_mapping
                    .left
                    .get(&internal)
                    .copied()
                    .map(|stable| (stable, &**key))
            })
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
        match self.hypergraph.hyperedges.get_index(self.index) {
            Some(HyperedgeKey { vertices, weight }) => {
                if let Ok(indexes) = self.hypergraph.get_vertices(&vertices.clone()) {
                    indexes
                        .iter()
                        .map(|index| self.hypergraph.get_vertex_weight(*index))
                        .collect::<Result<Vec<&V>, HypergraphError<V, HE>>>()
                        .ok()
                        .map(|vertices_weights| {
                            self.index += 1;
                            (*weight, vertices_weights.into_iter().copied().collect())
                        })
                } else {
                    None
                }
            }
            None => None,
        }
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
        match self.hypergraph.hyperedges.get_index(self.index) {
            Some(HyperedgeKey { vertices, weight }) => {
                let hypergraph = self.hypergraph;

                if let Ok(indexes) = hypergraph.get_vertices(vertices) {
                    indexes
                        .iter()
                        .map(|index| hypergraph.get_vertex_weight(*index))
                        .collect::<Result<Vec<&'s V>, HypergraphError<V, HE>>>()
                        .ok()
                        .map(|vertex_weights| {
                            self.index += 1;
                            (weight, vertex_weights)
                        })
                } else {
                    None
                }
            }
            None => None,
        }
    }
}
