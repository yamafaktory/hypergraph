use crate::{
    errors::HypergraphError, HyperedgeIndex, HyperedgeKey, Hypergraph, SharedTrait, VertexIndex,
};

use indexmap::IndexSet;
use itertools::Itertools;
use std::{cmp::Ordering, collections::BinaryHeap, fmt::Debug};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    // This private method is infallible since adding the same vertex
    // will return the existing index.
    fn add_vertex_index(&mut self, internal_index: usize) -> VertexIndex {
        match self.vertices_mapping.left.get(&internal_index) {
            Some(vertex_index) => *vertex_index,
            None => {
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

    // Private method to get the VertexIndex matching an internal index.
    pub(crate) fn get_vertex(
        &self,
        vertex_index: usize,
    ) -> Result<VertexIndex, HypergraphError<V, HE>> {
        match self.vertices_mapping.left.get(&vertex_index) {
            Some(index) => Ok(*index),
            None => Err(HypergraphError::InternalVertexIndexNotFound(vertex_index)),
        }
    }

    // Private method to get a vector of VertexIndex from a vector of internal indexes.
    pub(crate) fn get_vertices(
        &self,
        vertices: Vec<usize>,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        vertices
            .iter()
            .map(|vertex_index| self.get_vertex(*vertex_index))
            .collect()
    }

    // Private method to get the internal vertex matching a VertexIndex.
    pub(crate) fn get_internal_vertex(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<usize, HypergraphError<V, HE>> {
        match self.vertices_mapping.right.get(&vertex_index) {
            Some(index) => Ok(*index),
            None => Err(HypergraphError::VertexIndexNotFound(vertex_index)),
        }
    }

    // Private method to get the internal vertices from a vector of VertexIndex.
    pub(crate) fn get_internal_vertices(
        &self,
        vertices: Vec<VertexIndex>,
    ) -> Result<Vec<usize>, HypergraphError<V, HE>> {
        vertices
            .iter()
            .map(|vertex_index| self.get_internal_vertex(*vertex_index))
            .collect()
    }

    /// Adds a vertex with a custom weight to the hypergraph.
    /// Returns the index of the vertex.
    pub fn add_vertex(&mut self, weight: V) -> Result<VertexIndex, HypergraphError<V, HE>> {
        // Return an error if the weight is already assigned to another vertex.
        if self.vertices.contains_key(&weight) {
            return Err(HypergraphError::VertexWeightAlreadyAssigned(weight));
        }

        self.vertices
            .entry(weight)
            .or_insert(IndexSet::with_capacity(0));

        let internal_index = self
            .vertices
            .get_index_of(&weight)
            // This safe-check should always pass since the weight has been
            // inserted upfront.
            .ok_or(HypergraphError::VertexWeightNotFound(weight))?;

        Ok(self.add_vertex_index(internal_index))
    }

    /// Returns the number of vertices in the hypergraph.
    pub fn count_vertices(&self) -> usize {
        self.vertices.len()
    }

    /// Gets a list of the shortest path of vertices between two vertices.
    /// The implementation of the algorithm is based on
    /// <https://doc.rust-lang.org/std/collections/binary_heap/#examples>
    pub fn get_dijkstra_connections(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        struct Cursor {
            distance: usize,
            index: usize,
        }

        // Use a custom implementation of Ord as we want a min-heap BinaryHeap.
        impl Ord for Cursor {
            fn cmp(&self, other: &Cursor) -> Ordering {
                other
                    .distance
                    .cmp(&self.distance)
                    .then_with(|| self.distance.cmp(&other.distance))
            }
        }

        impl PartialOrd for Cursor {
            fn partial_cmp(&self, other: &Cursor) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        // Get the internal indexes of the vertices.
        let internal_from = self.get_internal_vertex(from)?;
        let internal_to = self.get_internal_vertex(to)?;

        // We need to initialize a vector of length equal to the number of vertices.
        // The default value, as per Dijkstra, must be set to infinity.
        // A value of usize::MAX is used.
        let mut distances = (0..self.vertices.len())
            .map(|_| usize::MAX)
            .collect::<Vec<usize>>();

        // Create an empty binary heap.
        let mut heap = BinaryHeap::new();

        // Initialize the first vertex to zero.
        distances[internal_from] = 0;

        // Push the first cursor to the heap.
        heap.push(Cursor {
            distance: 0,
            index: internal_from,
        });

        // Keep track of the traversal path.
        let mut path = Vec::<usize>::new();

        while let Some(Cursor { distance, index }) = heap.pop() {
            // End of the traversal.
            if index == internal_to {
                // We need to inject the index of the target vertex.
                path.push(internal_to);

                // Remove duplicates generated during the iteration of the algorithm.
                path.dedup();

                return self.get_vertices(path);
            }

            // Skip if a better path has already been found.
            if distance > distances[index] {
                continue;
            }

            let mapped_index = self.get_vertex(index)?;
            let indexes = self.get_adjacent_vertices_from(mapped_index)?;
            let internal_indexes = self.get_internal_vertices(indexes)?;

            // For every connected vertex, try to find the lowest distance.
            for vertex_index in internal_indexes {
                let next = Cursor {
                    // We assume a distance of one by default since vertices
                    // have custom weights.
                    distance: distance + 1,
                    index: vertex_index,
                };

                // If so, add it to the frontier and continue.
                if next.distance < distances[next.index] {
                    // Update the traversal accordingly.
                    path.push(index);

                    // Push it to the heap.
                    heap.push(next);

                    // Relaxation, we have now found a better way
                    distances[vertex_index] = next.distance;
                }
            }
        }

        // If we reach this point, return an empty vector.
        Ok(vec![])
    }

    /// Gets the list of all vertices connected from a given vertex.
    pub fn get_adjacent_vertices_from(
        &self,
        from: VertexIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        let results = self.get_connections(from, None)?;

        Ok(results
            .into_iter()
            .filter_map(|(_, vertex_index)| vertex_index)
            .sorted()
            .dedup()
            .collect_vec())
    }

    /// Gets the hyperedges of a vertex as a vector of HyperedgeIndex.
    pub fn get_vertex_hyperedges(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        let internal_index = self.get_internal_vertex(vertex_index)?;

        let (_, hyperedges_index_set) = self
            .vertices
            .get_index(internal_index)
            .ok_or(HypergraphError::InternalVertexIndexNotFound(internal_index))?;

        self.get_hyperedges(hyperedges_index_set.clone().into_iter().collect_vec())
    }

    /// Gets the hyperedges of a vertex as a vector of vectors of VertexIndex.
    pub fn get_full_vertex_hyperedges(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<Vec<Vec<VertexIndex>>, HypergraphError<V, HE>> {
        self.get_vertex_hyperedges(vertex_index).map(|hyperedges| {
            hyperedges
                .into_iter()
                .flat_map(|hyperedge_index| self.get_hyperedge_vertices(hyperedge_index))
                .collect()
        })
    }

    /// Gets the weight of a vertex from its index.
    pub fn get_vertex_weight(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<V, HypergraphError<V, HE>> {
        let internal_index = self.get_internal_vertex(vertex_index)?;

        self.vertices
            .get_index(internal_index)
            .map(|(weight, _)| *weight)
            .ok_or(HypergraphError::InternalVertexIndexNotFound(internal_index))
    }

    /// Removes a vertex by index.
    pub fn remove_vertex(
        &mut self,
        vertex_index: VertexIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        let internal_index = self.get_internal_vertex(vertex_index)?;

        // Get the hyperedges of the vertex.
        let hyperedges = self.get_internal_hyperedges(self.get_vertex_hyperedges(vertex_index)?)?;

        // Remove the vertex from the hyperedges which contain it.
        for hyperedge in hyperedges.into_iter() {
            let HyperedgeKey { vertices, .. } = self
                .hyperedges
                .get_index(hyperedge)
                .map(|hyperedge_key| hyperedge_key.to_owned())
                .ok_or(HypergraphError::InternalHyperedgeIndexNotFound(hyperedge))?;

            let hyperedge_index = self.get_hyperedge(hyperedge)?;

            // Get the unique vertices, i.e. check for self-loops.
            let unique_vertices = vertices.iter().sorted().dedup().collect_vec();

            // Remove the hyperedge if the vertex is the only one present.
            if unique_vertices.len() == 1 {
                self.remove_hyperedge(hyperedge_index)?;
            } else {
                // Otherwise update the hyperedge with the updated vertices.
                let updated_vertices = self.get_vertices(
                    vertices
                        .into_iter()
                        .filter(|vertex| *vertex != internal_index)
                        .collect_vec(),
                )?;

                self.update_hyperedge_vertices(hyperedge_index, updated_vertices)?;
            }
        }

        // Find the last index.
        let last_index = self.vertices.len() - 1;

        // Swap and remove by index.
        self.vertices.swap_remove_index(internal_index);

        // Update the mapping for the removed vertex.
        self.vertices_mapping.left.remove(&internal_index);
        self.vertices_mapping.right.remove(&vertex_index);

        // If the index to remove wasn't the last one, the last vertex has
        // been swapped in place of the removed one. See the remove_hyperedge
        // method for more details about the internals.
        if internal_index != last_index {
            // Get the index of the swapped vertex.
            let swapped_vertex_index = self.get_vertex(last_index)?;

            // Proceed with the aforementioned operations.
            self.vertices_mapping
                .right
                .insert(swapped_vertex_index, internal_index);
            self.vertices_mapping.left.remove(&last_index);
            self.vertices_mapping
                .left
                .insert(internal_index, swapped_vertex_index);

            let stale_hyperedges =
                self.get_internal_hyperedges(self.get_vertex_hyperedges(swapped_vertex_index)?)?;

            // Update the impacted hyperedges accordingly.
            for hyperedge in stale_hyperedges.into_iter() {
                let HyperedgeKey { vertices, weight } = self
                    .hyperedges
                    .get_index(hyperedge)
                    .map(|hyperedge_key| hyperedge_key.to_owned())
                    .ok_or(HypergraphError::InternalHyperedgeIndexNotFound(hyperedge))?;

                let updated_vertices = vertices
                    .into_iter()
                    .map(|vertex| {
                        // Remap the vertex if this is the swapped one.
                        if vertex == last_index {
                            internal_index
                        } else {
                            vertex
                        }
                    })
                    .collect_vec();

                // Insert the new entry with the updated vertices.
                // Since we are not altering the weight, we can safely perform
                // the operation without checking its output.
                self.hyperedges.insert(HyperedgeKey {
                    vertices: updated_vertices,
                    weight,
                });

                // Swap and remove by index.
                // Since we know that the hyperedge index is correct, we can
                // safely perform the operation without checking its output.
                self.hyperedges.swap_remove_index(hyperedge);
            }
        }

        // Return a unit.
        Ok(())
    }

    /// Updates the weight of a vertex by index.
    pub fn update_vertex_weight(
        &mut self,
        vertex_index: VertexIndex,
        weight: V,
    ) -> Result<(), HypergraphError<V, HE>> {
        let internal_index = self.get_internal_vertex(vertex_index)?;

        let (previous_weight, index_set) = self
            .vertices
            .get_index(internal_index)
            .map(|(previous_weight, index_set)| (previous_weight.to_owned(), index_set.to_owned()))
            .ok_or(HypergraphError::InternalVertexIndexNotFound(internal_index))?;

        // Return an error if the new weight is the same as the previous one.
        if weight == previous_weight {
            return Err(HypergraphError::VertexWeightUnchanged(vertex_index, weight));
        }

        // Return an error if the new weight is already assigned to another
        // vertex.
        if self.vertices.contains_key(&weight) {
            return Err(HypergraphError::VertexWeightAlreadyAssigned(weight));
        }

        // We can't directly replace the value in the map.
        // First, we need to insert the new weight, it will end up
        // being at the last position.
        // Since we have already checked that the new weight is not in the
        // map, we can safely perform the operation without checking its output.
        self.vertices.insert(weight, index_set);

        // Then we use swap and remove. This will remove the previous weight
        // and insert the new one at the index position of the former.
        // This doesn't alter the indexing.
        // See the update_hyperedge_weight method for more detailed explanation.
        // Since we know that the internal index is correct, we can safely
        // perform the operation without checking its output.
        self.vertices.swap_remove_index(internal_index);

        // Return a unit.
        Ok(())
    }
}
