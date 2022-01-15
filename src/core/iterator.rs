use itertools::Itertools;

use crate::{errors::HypergraphError, HyperedgeKey, HyperedgeTrait, Hypergraph, VertexTrait};

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

/// Ideally we should be able to use GATs to expose `iter()`:
/// <https://rust-lang.github.io/generic-associated-types-initiative/explainer.html>
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
        // Get the current hyperedge matching the index.
        match self.hypergraph.hyperedges.iter().nth(self.index) {
            // Extract the internal vertices and its weight.
            Some(HyperedgeKey { vertices, weight }) => {
                // Convert the internal vertices to a vector of VertexIndex.
                // Since this is a fallible operation and we can't deal with a
                // Result within this iterator, remap to None on error.
                match self.hypergraph.get_vertices(vertices.to_owned()) {
                    Ok(indexes) => {
                        match indexes
                            .iter()
                            .map(|index| self.hypergraph.get_vertex_weight(*index))
                            .collect::<Result<Vec<&V>, HypergraphError<V, HE>>>()
                        {
                            Ok(vertices_weights) => {
                                // Now we can increment the inner index.
                                self.index += 1;

                                Some((*weight, vertices_weights.into_iter().cloned().collect_vec()))
                            }
                            Err(_) => None,
                        }
                    }
                    Err(_) => None,
                }
            }
            None => None,
        }
    }
}
