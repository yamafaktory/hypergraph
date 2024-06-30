use std::{fmt::Debug, path::PathBuf, pin::Pin, sync::Arc};

use futures::{Future, FutureExt};
use serde::{Deserialize, Serialize};
use tokio::{fs::remove_file, sync::Mutex};
use tracing::{instrument, warn};
use uuid::Uuid;

use crate::{
    collections::{HashMap, HashSet},
    defaults::DB_EXT,
    entities::{Entity, EntityKind},
    errors::HypergraphError,
    file::{read_from_file, write_to_file, Paths},
};

#[derive(Debug, Deserialize, Serialize)]
struct ChunkManagerDatabase {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    chunk_free_slots_map: HashMap<Uuid, u16>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    chunk_to_entities_map: HashMap<Uuid, HashSet<Uuid>>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    entity_to_chunk_map: HashMap<Uuid, Uuid>,
}

impl ChunkManagerDatabase {
    fn new() -> Self {
        Self {
            chunk_free_slots_map: HashMap::default(),
            chunk_to_entities_map: HashMap::default(),
            entity_to_chunk_map: HashMap::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ChunkManager {
    database: Arc<Mutex<ChunkManagerDatabase>>,
}

impl ChunkManager {
    pub(crate) fn new() -> Self {
        Self {
            database: Arc::new(Mutex::new(ChunkManagerDatabase::new())),
        }
    }

    fn get_chunk_path(&self, paths: Arc<Paths>, uuid: &Uuid) -> PathBuf {
        let path = &paths.root;
        let mut chunk_path = path.join(uuid.to_string());
        chunk_path.set_extension(DB_EXT);

        chunk_path
    }

    fn get_db_path(&self, entity_kind: &EntityKind, paths: Arc<Paths>) -> PathBuf {
        let db_path = match entity_kind {
            EntityKind::Hyperedge => &paths.hyperedges,
            EntityKind::Vertex => &paths.vertices,
        };

        db_path.to_path_buf()
    }

    async fn init(
        &mut self,
        entity_kind: &EntityKind,
        paths: Arc<Paths>,
    ) -> Result<(), HypergraphError> {
        // Get the database path.
        let db_path = self.get_db_path(entity_kind, paths);

        // Try to read from disk and update the struct if available.
        let data: Option<ChunkManagerDatabase> = read_from_file(db_path.clone()).await?;
        if let Some(chunk_manager_database) = data {
            self.database = Arc::new(Mutex::new(chunk_manager_database));

            return Ok(());
        }

        // Otherwise write the default database to disk.
        let r = self.database.lock().await;
        write_to_file(&*r, db_path).await
    }

    async fn insert_new_chunk(&mut self) -> Result<Uuid, HypergraphError> {
        let uuid = Uuid::now_v7();
        let mut lock = self.database.lock().await;

        lock.chunk_free_slots_map.insert(uuid, u16::MAX);

        Ok(uuid)
    }

    async fn get_chunk_uuid_from_entity_uuid(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<Uuid>, HypergraphError> {
        let lock = self.database.lock().await;

        Ok(lock.entity_to_chunk_map.get(uuid).copied())
    }

    async fn find_free_slot(&mut self) -> Result<Uuid, HypergraphError> {
        let d = self.database.clone();
        let mut lock = d.lock().await;
        let state = &mut *lock;

        // Empty map, insert new a new chunk.
        if state.chunk_free_slots_map.is_empty() {
            drop(lock);

            return self.insert_new_chunk().await;
        }

        let chunk_free_slots_map = state.chunk_free_slots_map.clone();

        // Return the next chunk with capacity.
        for (&chunk_uuid, &capacity) in chunk_free_slots_map.iter() {
            if capacity > 0 {
                // Update the capacity of the chunk.
                state.chunk_free_slots_map.insert(chunk_uuid, capacity - 1);

                return Ok(chunk_uuid);
            }
        }

        // Here all chunks are full, insert a new one.
        self.insert_new_chunk().await
    }

    async fn sync_to_disk(
        &self,
        entity_kind: &EntityKind,
        paths: Arc<Paths>,
    ) -> Result<(), HypergraphError> {
        let mut lock = self.database.lock().await;
        let db_path = self.get_db_path(entity_kind, paths);

        // Ensure to write minimum data to disk.
        lock.chunk_to_entities_map.shrink_to_fit();
        lock.chunk_free_slots_map.shrink_to_fit();
        lock.entity_to_chunk_map.shrink_to_fit();

        drop(lock);

        write_to_file(&*self.database.lock().await, db_path).await
    }

    pub(crate) async fn read_op<V, HE>(
        &mut self,
        entity_kind: &EntityKind,
        paths: Arc<Paths>,
        uuid: &Uuid,
    ) -> Result<Option<Entity<V, HE>>, HypergraphError>
    where
        V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
        HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
    {
        // Ensure to init the database.
        self.init(entity_kind, paths.clone()).await?;

        // Try to retrieve the chunk UUID, its path and finally the entity from disk.
        if let Some(chunk_uuid) = self.get_chunk_uuid_from_entity_uuid(uuid).await? {
            let chunk_path = self.get_chunk_path(paths, &chunk_uuid);
            let data: Option<HashMap<Uuid, Entity<V, HE>>> = read_from_file(chunk_path).await?;

            let entity = data.and_then(|map| map.get(uuid).cloned());

            Ok(entity)
        } else {
            Ok(None)
        }
    }

    pub(crate) async fn create_op<V, HE, U>(
        &mut self,
        entity_kind: &EntityKind,
        paths: Arc<Paths>,
        uuid: &Uuid,
        updater: U,
    ) -> Result<(), HypergraphError>
    where
        V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
        HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
        U: FnOnce(&mut HashMap<Uuid, Entity<V, HE>>),
    {
        // Ensure to init the database.
        self.init(entity_kind, paths.clone()).await?;

        // Find a free slot.
        let free_slot = self.find_free_slot().await?;

        // Get all the entities in this chunk.
        // If this is a new chunk, fallback to an empty set.
        let lock = self.database.lock().await;
        let mut current_entities_in_chunk = lock
            .chunk_to_entities_map
            .get(&free_slot)
            .cloned()
            .unwrap_or(HashSet::default());
        drop(lock);

        // Add the new uuid to the set.
        current_entities_in_chunk.insert(*uuid);

        {
            // Update the chunk map.
            let mut lock = self.database.lock().await;
            lock.chunk_to_entities_map
                .insert(free_slot, current_entities_in_chunk);

            // Update the entity map.
            lock.entity_to_chunk_map.insert(*uuid, free_slot);
        }

        // Sync the changes to disk.
        self.sync_to_disk(entity_kind, paths.clone()).await?;

        // Get the chunk path.
        let chunk_path = self.get_chunk_path(paths, &free_slot);

        // Try to retrieve the data from the chunk.
        // If the chunk doesn't exist yet - i.e. a None value - we need to create it.
        let data: Option<HashMap<Uuid, Entity<V, HE>>> = read_from_file(chunk_path.clone()).await?;
        let mut entities = data.unwrap_or_default();

        // Run the updater.
        updater(&mut entities);

        // Write to chunk file.
        write_to_file(&entities, chunk_path).await
    }

    async fn update_op(
        &mut self,
        entity_kind: &EntityKind,
        paths: Arc<Paths>,
    ) -> Result<(), HypergraphError> {
        // Ensure to init the database.
        self.init(entity_kind, paths.clone()).await?;

        // TODO
        Ok(())
    }

    pub(crate) async fn delete_op<V, HE>(
        &mut self,
        entity_kind: &EntityKind,
        paths: Arc<Paths>,
        uuid: &Uuid,
    ) -> Result<(), HypergraphError>
    where
        V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
        HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
    {
        // Ensure to init the database.
        self.init(entity_kind, paths.clone()).await?;

        // Try to retrieve the chunk UUID, its path and finally the entity from disk.
        if let Some(chunk_uuid) = self.get_chunk_uuid_from_entity_uuid(uuid).await? {
            let chunk_path = self.get_chunk_path(paths.clone(), &chunk_uuid);
            let chunk_data: Option<HashMap<Uuid, Entity<V, HE>>> =
                read_from_file(chunk_path.clone()).await?;

            if let Some(mut chunk_data) = chunk_data {
                // Two cases: either the chunk contains solely this entity,
                // or multiple ones.
                // Note: it's not possible to have an empty map here since we
                // drop the chunk at length one.
                if chunk_data.len() == 1 {
                    let mut lock = self.database.lock().await;

                    // Remove the chunk from the file system.
                    remove_file(chunk_path)
                        .await
                        .map_err(HypergraphError::File)?;

                    // Remove the entity from the entity to chunk map.
                    lock.entity_to_chunk_map.remove(uuid);

                    // Remove the chunk from the chunk to entity map.
                    lock.chunk_to_entities_map.remove(&chunk_uuid);

                    // Remove the chunk from the slots map.
                    lock.chunk_free_slots_map.remove(&chunk_uuid);

                    // Write the database to disk.
                    return self.sync_to_disk(entity_kind, paths).await;
                } else {
                    let mut lock = self.database.lock().await;

                    // Remove the chunk from the chunk to entity map.
                    lock.chunk_to_entities_map
                        .get_mut(&chunk_uuid)
                        .ok_or(HypergraphError::EntityUpdate)?
                        .remove(uuid);

                    // Update the free slots map.
                    *lock
                        .chunk_free_slots_map
                        .get_mut(&chunk_uuid)
                        .ok_or(HypergraphError::EntityUpdate)? += 1;

                    // Remove the entity from the entity to chunk map.
                    lock.entity_to_chunk_map.remove(uuid);

                    // Write the database to disk.
                    self.sync_to_disk(entity_kind, paths).await?;

                    // Remove the entity from the chunk.
                    chunk_data.remove(uuid);

                    // Write the chunk to disk.
                    write_to_file(&chunk_data, chunk_path).await?;
                }
            }

            Err(HypergraphError::EntityNotFound)
        } else {
            Err(HypergraphError::EntityNotFound)
        }
    }
}
