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
struct EntityDatabase {
    chunk_to_entities_map: HashMap<Uuid, HashSet<Uuid>>,
    chunk_free_slots_map: HashMap<Uuid, u16>,
}

impl EntityDatabase {
    fn new() -> Self {
        Self {
            chunk_to_entities_map: HashMap::default(),
            chunk_free_slots_map: HashMap::default(),
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

        return db_path.to_path_buf();
    }

    async fn get_or_init(
        &self,
        entity_kind: &EntityKind,
        paths: Arc<Paths>,
    ) -> Result<Self, HypergraphError> {
        let ref db_path = self.get_db_path(entity_kind, paths);

        if let Some(entity_database) = read_from_file(db_path).await? {
            return Ok(entity_database);
        }

        let entity_database = Self::new();

        write_to_file(&entity_database, db_path).await?;

        Ok(entity_database)
    }

    fn insert_new_chunk(&mut self) -> Uuid {
        let uuid = Uuid::now_v7();

        self.chunk_free_slots_map.insert(uuid, u16::MAX);

        uuid
    }

    fn find_free_slot(&mut self) -> Uuid {
        // Empty map, insert new a new chunk.
        if self.chunk_free_slots_map.is_empty() {
            return self.insert_new_chunk();
        }

        // Return the next chunk with capacity.
        for (&chunk_uuid, &capacity) in self.chunk_free_slots_map.iter() {
            if capacity > 0 {
                // Update the capacity of the chunk.
                self.chunk_free_slots_map.insert(chunk_uuid, 234);

                return chunk_uuid;
            }
        }

        // Here all chunks are full, insert a new one.
        return self.insert_new_chunk();
    }

    async fn sync_to_disk(
        &self,
        entity_kind: &EntityKind,
        paths: Arc<Paths>,
    ) -> Result<(), HypergraphError> {
        let ref db_path = self.get_db_path(entity_kind, paths);

        write_to_file(self, db_path).await
    }

    async fn read_op<V, HE>(
        &mut self,
        entity_kind: &EntityKind,
        paths: Arc<Paths>,
        uuid: &Uuid,
    ) -> Result<(), HypergraphError>
    where
        V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
        HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    {
        Ok(())
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
        self.get_or_init(entity_kind, paths.clone()).await?;

        // Find a free slot.
        let free_slot = self.find_free_slot();

        // Get all the entities in this chunk.
        // If this is a new chunk, fallback to an empty set.
        let mut current_entities_in_chunk = self
            .chunk_to_entities_map
            .get(&free_slot)
            .cloned()
            .unwrap_or(HashSet::default());

        // Add the new uuid to the set.
        current_entities_in_chunk.insert(*uuid);

        // Update the chunk map.
        self.chunk_to_entities_map
            .insert(free_slot, current_entities_in_chunk);

        // Sync the changes to disk.
        self.sync_to_disk(entity_kind, paths.clone()).await?;

        // Get the chunk path.
        let chunk_path = self.get_chunk_path(paths, &free_slot);

        // Try to retrieve the data from the chunk.
        // If the chunk doesn't exist yet - i.e. a None value - we need to create it.
        let data: Option<HashMap<Uuid, Entity<V, HE>>> = read_from_file(chunk_path).await?;
        let mut entities = data.unwrap_or(HashMap::default());

        // Run the updater.
        updater(&mut entities);

        Ok(())
    }

    async fn update_op(&mut self) -> Result<(), HypergraphError> {
        Ok(())
    }

    async fn delete_op(&mut self) -> Result<(), HypergraphError> {
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

async fn write_data_to_file<V, HE>(
    entity_kind: &EntityKind,
    uuid: &Uuid,
    data: HashMap<Uuid, Entity<V, HE>>,
    paths: Arc<Paths>,
    update: bool,
) -> Result<(), HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    // let chunk_path = if update {
    //     try_get_existing_chunk_path(entity_kind, paths.clone(), uuid).await?
    // } else {
    //     generate_new_chunk_path(entity_kind, paths, uuid).await?
    // };
    //
    // write_to_file(&data, chunk_path).await
    Ok(())
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
    let mut entity_database = EntityDatabase::new();
    entity_database.create_op(
        entity_kind,
        paths,
        uuid,
        |data: &mut HashMap<Uuid, Entity<V, HE>>| {},
    );
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
    // let entity_kind: EntityKind = entity_weight.into();
    //
    // let mut data = if update {
    //     read_data_from_file::<V, HE>(&entity_kind, uuid, paths.clone()).await?
    // } else {
    //     HashMap::default()
    // };
    //
    // match entity_weight {
    //     EntityWeight::Hyperedge(weight) => {
    //         data.insert(*uuid, Entity::Hyperedge(Hyperedge::new(weight.to_owned())));
    //     }
    //     EntityWeight::Vertex(weight) => {
    //         data.insert(*uuid, Entity::Vertex(Vertex::new(weight.to_owned())));
    //     }
    // };
    //
    // write_data_to_file(&entity_kind, uuid, data, paths, update).await?;

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
    // let mut data = read_data_from_file::<V, HE>(entity_kind, uuid, paths.clone()).await?;
    //
    // data.remove(uuid).ok_or(HypergraphError::EntityNotFound)?;
    //
    // if data.is_empty() {
    //     return remove_chunk(entity_kind, paths.clone(), uuid).await;
    // }
    //
    // update_chunk(entity_kind, paths.clone(), uuid, data).await
    //
    Ok(())
}
