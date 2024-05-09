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
    chunk_free_slots_map: HashMap<Uuid, u16>,
    chunk_to_entities_map: HashMap<Uuid, HashSet<Uuid>>,
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
        write_to_file(&*self.database.read().await, db_path).await?;

        Ok(())
    }

    async fn insert_new_chunk(&mut self) -> Result<Uuid, HypergraphError> {
        let uuid = Uuid::now_v7();

        self.database
            .write()
            .await
            .chunk_free_slots_map
            .insert(uuid, u16::MAX);

        Ok(uuid)
    }

    async fn find_free_slot(&mut self) -> Result<Uuid, HypergraphError> {
        // Empty map, insert new a new chunk.
        if self.database.read().await.chunk_free_slots_map.is_empty() {
            return self.insert_new_chunk().await;
        }

        // Return the next chunk with capacity.
        for (&chunk_uuid, &capacity) in self.database.read().await.chunk_free_slots_map.iter() {
            if capacity > 0 {
                // Update the capacity of the chunk.
                self.database
                    .write()
                    .await
                    .chunk_free_slots_map
                    .insert(chunk_uuid, 234);

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
        if let Some(chunk_uuid) = self.database.read().await.entity_to_chunk_map.get(uuid) {
            let chunk_path = self.get_chunk_path(paths, chunk_uuid);
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
        let mut current_entities_in_chunk = self
            .database
            .read()
            .await
            .chunk_to_entities_map
            .get(&free_slot)
            .cloned()
            .unwrap_or(HashSet::default());

        // Add the new uuid to the set.
        current_entities_in_chunk.insert(*uuid);

        // Update the chunk map.
        self.database
            .write()
            .await
            .chunk_to_entities_map
            .insert(free_slot, current_entities_in_chunk);

        // Update the entity map.
        self.database
            .write()
            .await
            .entity_to_chunk_map
            .insert(*uuid, free_slot);

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
        write_to_file(&entities, chunk_path).await?;

        Ok(())
    }

    async fn update_op(
        &mut self,
        entity_kind: &EntityKind,
        paths: Arc<Paths>,
    ) -> Result<(), HypergraphError> {
        // Ensure to init the database.
        self.init(entity_kind, paths.clone()).await?;

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
        if let Some(chunk_uuid) = self.database.read().await.entity_to_chunk_map.get(uuid) {
            let chunk_path = self.get_chunk_path(paths.clone(), chunk_uuid);
            let data: Option<HashMap<Uuid, Entity<V, HE>>> =
                read_from_file(chunk_path.clone()).await?;

            match data {
                Some(data) => {
                    // Two cases: either the chunk contains solely this entity, or multiple ones.
                    // In the first case, remove the chunk from the file system and the database.
                    if data.is_empty() {
                        remove_file(chunk_path)
                            .await
                            .map_err(HypergraphError::File)?;

                        self.database.write().await.entity_to_chunk_map.remove(uuid);

                        self.database
                            .write()
                            .await
                            .chunk_to_entities_map
                            .remove(chunk_uuid);
                        // TODO: free slot
                        self.sync_to_disk(entity_kind, paths).await?;
                    }
                }
                None => return Err(HypergraphError::EntityNotFound),
            };
        } else {
            //
        }

        Ok(())
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
    entity_kind: &EntityKind,
    uuid: &Uuid,
    paths: Arc<Paths>,
) -> Result<Option<Entity<V, HE>>, HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    let mut chunk_manager = ChunkManager::new();

    let entity = chunk_manager
        .read_op::<V, HE>(entity_kind, paths, uuid)
        .await?;

    Ok(entity)
}

pub(crate) async fn write_relation_to_file<V, HE>(
    uuid: &Uuid,
    entity_relation: &EntityRelation,
    paths: Arc<Paths>,
) -> Result<(), HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    let entity_kind = &entity_relation.into();
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
    let mut chunk_manager = ChunkManager::new();
    chunk_manager
        .create_op(
            entity_kind,
            paths,
            uuid,
            |data: &mut HashMap<Uuid, Entity<V, HE>>| {},
        )
        .await?;
    Ok(())
}

pub(crate) async fn write_weight_to_file<V, HE>(
    uuid: &Uuid,
    entity_weight: &EntityWeight<V, HE>,
    paths: Arc<Paths>,
    update: bool,
) -> Result<(), HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    let entity_kind: EntityKind = entity_weight.into();

    let mut chunk_manager = ChunkManager::new();
    chunk_manager
        .create_op(
            &entity_kind,
            paths,
            uuid,
            |data: &mut HashMap<Uuid, Entity<V, HE>>| {
                match entity_weight {
                    EntityWeight::Hyperedge(weight) => {
                        data.insert(*uuid, Entity::Hyperedge(Hyperedge::new(weight.to_owned())));
                    }
                    EntityWeight::Vertex(weight) => {
                        data.insert(*uuid, Entity::Vertex(Vertex::new(weight.to_owned())));
                    }
                };
            },
        )
        .await?;

    Ok(())
}

pub(crate) async fn remove_entity_from_file<V, HE>(
    uuid: &Uuid,
    entity_kind: &EntityKind,
    paths: Arc<Paths>,
) -> Result<(), HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    let mut chunk_manager = ChunkManager::new();

    chunk_manager
        .delete_op::<V, HE>(entity_kind, paths, uuid)
        .await
}
