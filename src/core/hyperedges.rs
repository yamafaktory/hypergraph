use crate::{
    core::utils::are_arrays_equal, Hypergraph, SharedTrait, StableHyperedgeWeightedIndex,
    StableVertexIndex, UnstableHyperedgeWeightedIndex, UnstableVertexIndex,
};

use indexmap::IndexSet;
use itertools::{max, Itertools};

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    fn add_stable_hyperedge_weighted_index(
        &mut self,
        unstable_hyperedge_weighted_index: UnstableHyperedgeWeightedIndex,
    ) -> StableHyperedgeWeightedIndex {
        match self
            .hyperedges_mapping_left
            .get(&unstable_hyperedge_weighted_index)
        {
            Some(stable_hyperedge_weighted_index) => *stable_hyperedge_weighted_index,
            None => {
                let stable_index = StableHyperedgeWeightedIndex(self.hyperedges_count);

                self.hyperedges_mapping_left
                    .insert(unstable_hyperedge_weighted_index, stable_index);
                self.hyperedges_mapping_right
                    .insert(stable_index, unstable_hyperedge_weighted_index);

                // Update the counter.
                self.hyperedges_count += 1;

                stable_index
            }
        }
    }

    /// Adds a hyperedge as an array of vertices indexes and a custom weight in the hypergraph.
    /// Returns the weighted index of the hyperedge.
    pub fn add_hyperedge(
        &mut self,
        vertices: Vec<StableVertexIndex>,
        weight: HE,
    ) -> Option<StableHyperedgeWeightedIndex> {
        // Safe check to avoid out of bound index!
        match self
            .vertices_mapping_right
            .get(&max(vertices.to_owned()).unwrap())
        {
            Some(max_stable_index) => {
                if max_stable_index + 1 > self.vertices.len() {
                    return None;
                }

                let unstable_vertices = vertices
                    .iter()
                    .map(|vertex| *self.vertices_mapping_right.get(vertex).unwrap())
                    .collect_vec();

                // Insert the new hyperedge with the corresponding weight, get back the indexes.
                let new_hyperedge_weighted_index = match self.hyperedges.get(&unstable_vertices) {
                    // Some weights are already present, use the existing IndexSet.
                    Some(weights) => {
                        let mut new_weights = weights.clone();
                        let (weight_index, _) = new_weights.insert_full(weight);
                        let (hyperedge_index, _) =
                            self.hyperedges.insert_full(unstable_vertices, new_weights);

                        [hyperedge_index, weight_index]
                    }
                    // No weights are present, create a new IndexSet.
                    None => {
                        let mut weights = IndexSet::new();
                        let (weight_index, _) = weights.insert_full(weight);
                        let (hyperedge_index, _) =
                            self.hyperedges.insert_full(unstable_vertices, weights);

                        [hyperedge_index, weight_index]
                    }
                };

                // Update the vertices so that we keep directly track of the hyperedge.
                for vertex in vertices.iter() {
                    let unstable_index = *self.vertices_mapping_right.get(vertex).unwrap();
                    let mut index_set = self.vertices[unstable_index].clone();

                    index_set.insert(new_hyperedge_weighted_index);

                    self.vertices.insert(
                        *self.vertices.get_index(unstable_index).unwrap().0,
                        index_set,
                    );
                }

                Some(self.add_stable_hyperedge_weighted_index(new_hyperedge_weighted_index))
            }
            None => None,
        }
    }

    /// Returns the number of hyperedges in the hypergraph.
    pub fn count_hyperedges(&self) -> usize {
        self.hyperedges
            .iter()
            .fold(0, |count, (_, weights)| count + weights.len())
    }

    // Returns an iterator of all the hyperedges.
    pub fn get_hyperedges(
        &self,
    ) -> impl Iterator<Item = (&Vec<UnstableVertexIndex>, &IndexSet<HE>)> {
        self.hyperedges.iter()
    }

    /// Gets the list of all hyperedges containing a matching connection from
    /// one vertex to another.
    pub fn get_hyperedges_connections(
        &self,
        from: StableVertexIndex,
        to: StableVertexIndex,
    ) -> Vec<StableVertexIndex> {
        self.get_connections(from, Some(to))
            .iter()
            .map(|(_, stable_vertex_index)| *stable_vertex_index)
            .collect_vec()
    }

    /// Gets the hyperedge's vertices.
    pub fn get_hyperedge_vertices(
        &self,
        stable_hyperedge_index: StableHyperedgeWeightedIndex,
    ) -> Option<Vec<StableVertexIndex>> {
        match self.hyperedges_mapping_right.get(&stable_hyperedge_index) {
            Some([unstable_hyperedge_index, _]) => self
                .hyperedges
                .get_index(*unstable_hyperedge_index)
                .map(|(vertices, _)| {
                    vertices
                        .iter()
                        .map(|vertex| *self.vertices_mapping_left.get(vertex).unwrap())
                        .collect()
                }),
            None => None,
        }
    }

    /// Gets the weight of a hyperedge from its weighted index.
    pub fn get_hyperedge_weight(
        &self,
        stable_hyperedge_weighted_index: StableHyperedgeWeightedIndex,
    ) -> Option<HE> {
        match self
            .hyperedges_mapping_right
            .get(&stable_hyperedge_weighted_index)
        {
            Some([hyperedge_index, weight_index]) => {
                match self.hyperedges.get_index(*hyperedge_index) {
                    Some((_, weights)) => weights.get_index(*weight_index).cloned(),
                    None => None,
                }
            }
            None => None,
        }
    }

    /// Gets the intersections of a set of hyperedges as a vector of vertices.
    pub fn get_hyperedges_intersections(
        &self,
        hyperedges: &[StableHyperedgeWeightedIndex],
    ) -> Vec<UnstableVertexIndex> {
        hyperedges
            .iter()
            .filter_map(|stable_hyperedge_weighted_index| {
                match self
                    .hyperedges_mapping_right
                    .get(stable_hyperedge_weighted_index)
                {
                    Some([unstable_hypergraph_index, _]) => self
                        .hyperedges
                        .get_index(*unstable_hypergraph_index)
                        .map(|(vertices, _)| vertices.iter().unique().collect_vec()),
                    None => Some(vec![]),
                }
            })
            .flatten()
            .sorted()
            // Map the result to tuples where the second term is an arbitrary value.
            // The goal is to group them by indexes.
            .map(|index| (*index, 0))
            .into_group_map()
            .iter()
            // Filter the groups having the same size as the hyperedge.
            .filter_map(|(index, occurences)| {
                if occurences.len() == hyperedges.len() {
                    Some(*index)
                } else {
                    None
                }
            })
            .sorted()
            .collect_vec()
    }

    /// Removes a hyperedge based on its index.
    /// IndexMap doesn't allow holes by design, see:
    /// https://github.com/bluss/indexmap/issues/90#issuecomment-455381877
    /// As a consequence, we have two options. Either we use shift_remove
    /// and it will result in an expensive regeneration of all the indexes
    /// in the map or we use swap_remove and deal with the fact that the last
    /// element will be swapped in place of the removed one and will thus get
    /// a new index. We use the latter solution for performance reasons.
    pub fn remove_hyperedge(
        &mut self,
        stable_hyperedge_weighted_index: StableHyperedgeWeightedIndex,
    ) -> bool {
        match self
            .hyperedges_mapping_right
            .get(&stable_hyperedge_weighted_index)
        {
            Some([hyperedge_index, _]) => {
                match self.hyperedges.clone().get_index(*hyperedge_index) {
                    Some((vertices, weights)) => {
                        // Either we have multiple weights for the index or only one.
                        // In the first case, we only want to drop the weight.
                        if weights.len() > 1 {
                            let mut new_weights = weights.clone();

                            match self.get_hyperedge_weight(stable_hyperedge_weighted_index) {
                                Some(weight) => {
                                    // Use swap and remove.
                                    // This can potentially alter the indexes but only
                                    // at the internal hyperedge level, i.e. it doesn't
                                    // break the stability of the indexes.
                                    new_weights.swap_remove(&weight);

                                    // Update with the new weights.
                                    self.hyperedges
                                        .insert(vertices.clone(), new_weights)
                                        .is_some()
                                }
                                None => false,
                            }
                        } else {
                            // In the second case, we need to remove the hyperedge completely.
                            // First update the vertices accordingly.
                            for index in vertices.iter() {
                                if let Some((weight, vertex)) =
                                    self.vertices.clone().get_index(*index)
                                {
                                    self.vertices.insert(
                                        *weight,
                                        vertex.iter().fold(
                                            IndexSet::new(),
                                            |mut new_index_set, hyperedge| {
                                                if !are_arrays_equal(hyperedge, vertices) {
                                                    new_index_set.insert(*hyperedge);
                                                }

                                                new_index_set
                                            },
                                        ),
                                    );
                                }
                            }

                            // Finally remove it.
                            self.hyperedges.swap_remove(vertices).is_some()
                        }
                    }
                    None => false,
                }
            }
            None => false,
        }
    }

    /// Updates the weight of a hyperedge based on its weighted index.
    pub fn update_hyperedge_weight(
        &mut self,
        stable_hyperedge_weighted_index: StableHyperedgeWeightedIndex,
        weight: HE,
    ) -> bool {
        match self
            .hyperedges_mapping_right
            .get(&stable_hyperedge_weighted_index)
        {
            Some([hyperedge_index, weight_index]) => {
                match self.hyperedges.get_index_mut(*hyperedge_index) {
                    Some((_, weights)) => {
                        // We can't directly replace the value in the set.
                        // First, we need to insert the new weight, it will end up
                        // being at the last position.
                        if !weights.insert(weight) {
                            return false;
                        };

                        // Then get the value by index of the original weight.
                        match weights.clone().get_index(*weight_index) {
                            Some(weight) => {
                                // Last, use swap and remove. It will remove the old weight
                                // and insert the new one at the index position of the former.
                                weights.swap_remove(weight)
                            }
                            None => false,
                        }
                    }
                    None => false,
                }
            }
            None => false,
        }
    }

    /// Updates the vertices of a hyperedge based on its index.
    pub fn update_hyperedge_vertices(
        &mut self,
        stable_hyperedge_weighted_index: StableHyperedgeWeightedIndex,
        vertices: Vec<StableVertexIndex>,
    ) -> bool {
        match self
            .hyperedges_mapping_right
            .get(&stable_hyperedge_weighted_index)
        {
            Some([unstable_hyperedge_index, unstable_hyperedge_weight]) => {
                match self.hyperedges.clone().get_index(*unstable_hyperedge_index) {
                    Some((key, value)) => {
                        let new_vertices = vertices
                            .iter()
                            .map(|vertex| *self.vertices_mapping_right.get(vertex).unwrap())
                            .collect_vec();

                        // Keep track of the initial indexes.
                        let previous_vertices = self
                            .get_hyperedge_vertices(stable_hyperedge_weighted_index)
                            .unwrap()
                            .iter()
                            .map(|vertex| *self.vertices_mapping_right.get(vertex).unwrap())
                            .collect_vec();

                        // Find the indexes which have been added.
                        let added = new_vertices
                            .iter()
                            .fold(vec![], |mut acc: Vec<usize>, index| {
                                if !previous_vertices
                                    .iter()
                                    .any(|current_index| current_index == index)
                                {
                                    acc.push(*index)
                                }

                                acc
                            })
                            .into_iter()
                            .sorted()
                            .dedup()
                            .collect_vec();

                        // Find the indexes which have been removed.
                        let removed = previous_vertices
                            .iter()
                            .filter_map(|index| {
                                if !new_vertices
                                    .iter()
                                    .any(|current_index| index == current_index)
                                {
                                    Some(*index)
                                } else {
                                    None
                                }
                            })
                            .sorted()
                            .dedup()
                            .collect_vec();

                        // Finally get the unchanged ones.
                        let unchanged = previous_vertices
                            .iter()
                            .filter_map(|index| {
                                if !removed.iter().any(|current_index| index == current_index) {
                                    Some(*index)
                                } else {
                                    None
                                }
                            })
                            .sorted()
                            .dedup()
                            .collect_vec();

                        // Process the updated ones.
                        for index in unchanged.iter() {
                            if let Some((weight, vertex)) = self.vertices.clone().get_index(*index)
                            {
                                self.vertices.insert(
                                    *weight,
                                    vertex.iter().fold(
                                        IndexSet::new(),
                                        |mut new_index_set, hyperedge| {
                                            new_index_set.insert(
                                                // Insert the new ones if it's a match.
                                                if are_arrays_equal(
                                                    hyperedge,
                                                    &[
                                                        *unstable_hyperedge_index,
                                                        *unstable_hyperedge_weight,
                                                    ],
                                                ) {
                                                    [
                                                        *unstable_hyperedge_index,
                                                        *unstable_hyperedge_weight,
                                                    ]
                                                } else {
                                                    *hyperedge
                                                },
                                            );

                                            new_index_set
                                        },
                                    ),
                                );
                            };
                        }

                        // Process the removed ones.
                        for index in removed.iter() {
                            if let Some((weight, hyperedges)) =
                                self.vertices.clone().get_index(*index)
                            {
                                self.vertices.insert(
                                    *weight,
                                    hyperedges.iter().fold(
                                        IndexSet::new(),
                                        |mut new_index_set, hyperedge| {
                                            // Skip the removed ones, i.e. if there's no match.
                                            if !are_arrays_equal(
                                                hyperedge,
                                                &[
                                                    *unstable_hyperedge_index,
                                                    *unstable_hyperedge_weight,
                                                ],
                                            ) {
                                                new_index_set.insert(*hyperedge);
                                            }

                                            new_index_set
                                        },
                                    ),
                                );
                            };
                        }

                        // Process the added ones.
                        for index in added.iter() {
                            if let Some((vertex_weight, hyperedges)) =
                                self.vertices.to_owned().get_index_mut(*index)
                            {
                                // Insert the new vertices.
                                hyperedges.insert([
                                    *unstable_hyperedge_index,
                                    *unstable_hyperedge_weight,
                                ]);

                                // Update.
                                self.vertices.insert(*vertex_weight, hyperedges.to_owned());
                            };
                        }

                        // Finally update the hyperedge.
                        // There are two cases now:
                        // - one or more hyperdedges exist with the given key
                        // - no hyperdedges exist with the given key
                        let (new_key, _) = self.hyperedges.insert_full(
                            new_vertices.clone(),
                            match self.hyperedges.get(&new_vertices) {
                                Some(previous_index_set) => {
                                    // Get the weight of the current hyperedge.
                                    let current_hyperedge_weight = self
                                        .get_hyperedge_weight(stable_hyperedge_weighted_index)
                                        .unwrap();

                                    // Create a copy of the existing weights.
                                    let mut new_index_set = previous_index_set.clone();

                                    // Insert the new weight.
                                    new_index_set.insert(current_hyperedge_weight);

                                    new_index_set
                                }
                                None => value.to_owned(),
                            },
                        );

                        // In the last step there are two cases:
                        // - the hyperedge is simple - i.e. it has only one weight
                        // - the hyperedge is non-simple
                        if value.len() == 1 {
                            // Simple case. We can use the swap and remove operations.
                            // No index mapping is affected.
                            self.hyperedges.swap_remove(key).is_some()
                        } else {
                            // Non-simple case. We need to update the index mapping.
                            dbg!(new_key, unstable_hyperedge_weight);
                            self.hyperedges_mapping_right
                                .insert(
                                    stable_hyperedge_weighted_index,
                                    [new_key, *unstable_hyperedge_weight],
                                )
                                .is_some()
                        }
                    }
                    None => false,
                }
            }
            None => false,
        }
    }
}
