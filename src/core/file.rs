use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{remove_file, write, OpenOptions},
    io::AsyncReadExt,
    spawn,
    sync::RwLock,
};
use uuid::Uuid;

use crate::{
    collections::{HashMap, HashSet},
    defaults::DB_EXT,
    entities::{Entity, EntityKind, EntityRelation, EntityWeight, Hyperedge, Vertex},
    errors::HypergraphError,
};

#[derive(Debug)]
pub(crate) struct Paths {
    pub(crate) hyperedges: PathBuf,
    pub(crate) vertices: PathBuf,
    pub(crate) root: PathBuf,
}

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
struct ChunkManager {
    database: Arc<RwLock<ChunkManagerDatabase>>,
}

impl ChunkManager {
    fn new() -> Self {
        Self {
            database: Arc::new(RwLock::new(ChunkManagerDatabase::new())),
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
        let db_path = &self.get_db_path(entity_kind, paths);

        // Try to read from disk and update the struct if available.
        let data: Option<ChunkManagerDatabase> = read_from_file(db_path).await?;
        if let Some(chunk_manager_database) = data {
            self.database = Arc::new(RwLock::new(chunk_manager_database));

            return Ok(());
        }

        // Otherwise write the default database to disk.
        let r = self.database.read().await;
        write_to_file(&*r, db_path).await
    }

    async fn insert_new_chunk(&mut self) -> Result<Uuid, HypergraphError> {
        let uuid = Uuid::now_v7();
        let mut w = self.database.write().await;

        w.chunk_free_slots_map.insert(uuid, u16::MAX);

        Ok(uuid)
    }

    async fn get_chunk_uuid_from_entity_uuid(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<Uuid>, HypergraphError> {
        let r = self.database.read().await;

        Ok(r.entity_to_chunk_map.get(uuid).cloned())
    }

    async fn find_free_slot(&mut self) -> Result<Uuid, HypergraphError> {
        // Empty map, insert new a new chunk.
        if self.database.read().await.chunk_free_slots_map.is_empty() {
            return self.insert_new_chunk().await;
        }

        let r = self.database.read().await;
        let chunk_free_slots_map = r.chunk_free_slots_map.clone();

        drop(r);

        // Return the next chunk with capacity.
        for (&chunk_uuid, &capacity) in chunk_free_slots_map.iter() {
            if capacity > 0 {
                // Update the capacity of the chunk.
                let mut r = self.database.write().await;

                r.chunk_free_slots_map.insert(chunk_uuid, capacity - 1);

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
        let db_path = &self.get_db_path(entity_kind, paths);

        {
            let mut w = self.database.write().await;

            // Ensure to write minimum data to disk.
            w.chunk_to_entities_map.shrink_to_fit();
            w.chunk_free_slots_map.shrink_to_fit();
            w.entity_to_chunk_map.shrink_to_fit();
        }

        write_to_file(&*self.database.read().await, db_path).await
    }

    async fn read_op<V, HE>(
        &mut self,
        entity_kind: &EntityKind,
        paths: Arc<Paths>,
        uuid: &Uuid,
    ) -> Result<Option<Entity<V, HE>>, HypergraphError>
    where
        V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
        HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
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

    async fn create_op<V, HE, U>(
        &mut self,
        entity_kind: &EntityKind,
        paths: Arc<Paths>,
        uuid: &Uuid,
        updater: U,
    ) -> Result<(), HypergraphError>
    where
        V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
        HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
        U: FnOnce(&mut HashMap<Uuid, Entity<V, HE>>),
    {
        // Ensure to init the database.
        self.init(entity_kind, paths.clone()).await?;

        // Find a free slot.
        let free_slot = self.find_free_slot().await?;

        // Get all the entities in this chunk.
        // If this is a new chunk, fallback to an empty set.
        let r = self.database.read().await;
        let mut current_entities_in_chunk = r
            .chunk_to_entities_map
            .get(&free_slot)
            .cloned()
            .unwrap_or(HashSet::default());
        drop(r);

        // Add the new uuid to the set.
        current_entities_in_chunk.insert(*uuid);

        {
            // Update the chunk map.
            let mut w = self.database.write().await;
            w.chunk_to_entities_map
                .insert(free_slot, current_entities_in_chunk);

            // Update the entity map.
            w.entity_to_chunk_map.insert(*uuid, free_slot);
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

    async fn delete_op<V, HE>(
        &mut self,
        entity_kind: &EntityKind,
        paths: Arc<Paths>,
        uuid: &Uuid,
    ) -> Result<(), HypergraphError>
    where
        V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
        HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
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
                    let mut w = self.database.write().await;

                    // Remove the chunk from the file system.
                    remove_file(chunk_path)
                        .await
                        .map_err(HypergraphError::File)?;

                    // Remove the entity from the entity to chunk map.
                    w.entity_to_chunk_map.remove(uuid);

                    // Remove the chunk from the chunk to entity map.
                    w.chunk_to_entities_map.remove(&chunk_uuid);

                    // Remove the chunk from the slots map.
                    w.chunk_free_slots_map.remove(&chunk_uuid);

                    // Write the database to disk.
                    return self.sync_to_disk(entity_kind, paths).await;
                } else {
                    let mut w = self.database.write().await;

                    // Remove the chunk from the chunk to entity map.
                    w.chunk_to_entities_map
                        .get_mut(&chunk_uuid)
                        .ok_or(HypergraphError::EntityUpdate)?
                        .remove(uuid);

                    // Update the free slots map.
                    *w.chunk_free_slots_map
                        .get_mut(&chunk_uuid)
                        .ok_or(HypergraphError::EntityUpdate)? += 1;

                    // Remove the entity from the entity to chunk map.
                    w.entity_to_chunk_map.remove(uuid);

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

async fn read_from_file<D, P>(path: P) -> Result<Option<D>, HypergraphError>
where
    D: Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    P: AsRef<Path>,
{
    let path_buf = path.as_ref().to_path_buf();
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .await
        .map_err(|error| HypergraphError::PathNotAccessible(error, path_buf))?;
    let metadata = file.metadata().await.map_err(HypergraphError::File)?;

    if metadata.len() != 0 {
        let mut contents = vec![];

        file.read_to_end(&mut contents)
            .await
            .map_err(HypergraphError::File)?;

        return deserialize(&contents)
            .map_err(|_| HypergraphError::Deserialization)
            .map(Some);
    }

    Ok(None)
}

async fn write_to_file<D, P>(data: &D, path: P) -> Result<(), HypergraphError>
where
    D: Serialize,
    P: AsRef<Path>,
{
    let bytes = serialize(&data).map_err(|_| HypergraphError::Serialization)?;

    write(path, bytes).await.map_err(HypergraphError::File)
}

pub(crate) async fn read_entity_from_file<V, HE>(
    entity_kind: EntityKind,
    uuid: Uuid,
    paths: Arc<Paths>,
) -> Result<Option<Entity<V, HE>>, HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
{
    let handle = spawn(async move {
        let mut chunk_manager = ChunkManager::new();

        let entity = chunk_manager
            .read_op::<V, HE>(&entity_kind, paths, &uuid)
            .await?;

        Ok(entity)

        //Ok::<(), HypergraphError>(())
    });

    if let Ok(result) = handle.await {
        result
    } else {
        Err(HypergraphError::Processing)
    }
}

pub(crate) async fn write_relation_to_file<V, HE>(
    uuid: Uuid,
    entity_relation: EntityRelation,
    paths: Arc<Paths>,
) -> Result<(), HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    spawn(async move {
        let entity_kind: EntityKind = entity_relation.into();
        let mut chunk_manager = ChunkManager::new();

        chunk_manager
            .create_op(
                &entity_kind,
                paths,
                &uuid,
                |data: &mut HashMap<Uuid, Entity<V, HE>>| {},
            )
            .await?;

        Ok::<(), HypergraphError>(())
    });
    // let mut data = read_data_from_file::<V, HE>(entity_kind, uuid, paths.clone()).await?;
    // let entity = data.get_mut(uuid).ok_or(HypergraphError::EntityUpdate)?;
    //
    // match entity_relation {
    //     EntityRelation::Hyperedge(vertices) => match entity {
    //         Entity::Hyperedge(hyperedge) => {
    //             hyperedge.vertices = vertices.to_owned();
    //         }
    //         Entity::Vertex(_) => unreachable!(),
    //     },
    //     EntityRelation::Vertex(hyperedges) => match entity {
    //         Entity::Hyperedge(_) => unreachable!(),
    //         Entity::Vertex(vertex) => {
    //             vertex.hyperedges = hyperedges.to_owned();
    //         }
    //     },
    // };
    //
    // write_data_to_file(entity_kind, uuid, data, paths, true).await
    Ok(())
}

pub(crate) async fn write_weight_to_file<V, HE>(
    uuid: Uuid,
    entity_weight: EntityWeight<V, HE>,
    paths: Arc<Paths>,
    update: bool,
) -> Result<(), HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
{
    spawn(async move {
        let mut chunk_manager = ChunkManager::new();
        let entity_kind = (&entity_weight).into();

        chunk_manager
            .create_op(
                &entity_kind,
                paths,
                &uuid,
                |data: &mut HashMap<Uuid, Entity<V, HE>>| {
                    match entity_weight {
                        EntityWeight::Hyperedge(weight) => {
                            data.insert(uuid, Entity::Hyperedge(Hyperedge::new(weight.to_owned())));
                        }
                        EntityWeight::Vertex(weight) => {
                            data.insert(uuid, Entity::Vertex(Vertex::new(weight.to_owned())));
                        }
                    };
                },
            )
            .await?;

        Ok::<(), HypergraphError>(())
    });

    Ok(())
}

pub(crate) async fn remove_entity_from_file<V, HE>(
    uuid: Uuid,
    entity_kind: EntityKind,
    paths: Arc<Paths>,
) -> Result<(), HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    spawn(async move {
        let mut chunk_manager = ChunkManager::new();

        chunk_manager
            .delete_op::<V, HE>(&entity_kind, paths, &uuid)
            .await?;

        Ok::<(), HypergraphError>(())
    });

    Ok(())
}
