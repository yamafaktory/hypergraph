use rayon::prelude::*;

use crate::{HyperedgeKey, HyperedgeTrait, Hypergraph, VertexTrait, errors::HypergraphError};

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

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns a borrowing iterator over the hyperedges as `(&HE, Vec<&V>)` tuples.
    pub fn iter(&self) -> HypergraphBorrowingIterator<'_, V, HE> {
        HypergraphBorrowingIterator {
            hypergraph: self,
            index: 0,
        }
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
        match self.hypergraph.hyperedges.iter().nth(self.index) {
            Some(HyperedgeKey { vertices, weight }) => {
                if let Ok(indexes) = self.hypergraph.get_vertices(&vertices.clone()) {
                    indexes
                        .par_iter()
                        .map(|index| self.hypergraph.get_vertex_weight(*index))
                        .collect::<Result<Vec<&V>, HypergraphError<V, HE>>>()
                        .ok()
                        .map(|vertices_weights| {
                            self.index += 1;

                            (*weight, vertices_weights.into_par_iter().cloned().collect())
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
        match self.hypergraph.hyperedges.iter().nth(self.index) {
            Some(HyperedgeKey { vertices, weight }) => {
                let hypergraph = self.hypergraph;

                if let Ok(indexes) = hypergraph.get_vertices(vertices) {
                    indexes
                        .par_iter()
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
