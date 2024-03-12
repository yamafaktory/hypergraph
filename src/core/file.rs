use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{write, OpenOptions},
    io::AsyncReadExt,
};
use uuid::Uuid;

use crate::{
    collections::HashMap,
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
struct Pool {
    free_slot: Uuid,
    slot_count: u16,
}

impl Pool {
    fn new() -> Self {
        Self {
            free_slot: Uuid::now_v7(),
            slot_count: u16::MAX,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct EntityDatabase {
    pool: Pool,
    map: HashMap<Uuid, Uuid>,
}

impl EntityDatabase {
    fn new() -> Self {
        Self {
            pool: Pool::new(),
            map: HashMap::default(),
        }
    }
}

async fn get_chunk_path(
    entity_kind: &EntityKind,
    paths: Arc<Paths>,
    uuid: &Uuid,
) -> Result<PathBuf, HypergraphError> {
    let db_path = match entity_kind {
        EntityKind::Hyperedge => &paths.hyperedges,
        EntityKind::Vertex => &paths.vertices,
    };

    let entity_database_from_disk: Option<EntityDatabase> = read_from_file(db_path).await?;

    let mut entity_database = match entity_database_from_disk {
        Some(entity_database) => entity_database,
        None => EntityDatabase::new(),
    };

    if entity_database.pool.slot_count == 1 {
        entity_database.pool.free_slot = Uuid::now_v7();
        entity_database.pool.slot_count = u16::MAX;
    } else {
        entity_database.pool.slot_count -= 1;
    }

    let file_uuid = if entity_database.map.contains_key(uuid) {
        uuid
    } else {
        entity_database
            .map
            .insert(entity_database.pool.free_slot, *uuid);
        &entity_database.pool.free_slot
    };

    write_to_file(&entity_database, db_path).await?;

    Ok(Path::new(&format!("{}.db", file_uuid)).to_path_buf())
}

async fn read_from_file<D, P>(path: P) -> Result<Option<D>, HypergraphError>
where
    D: Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    P: AsRef<Path>,
{
    let t = path.as_ref().to_path_buf();
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .await
        .map_err(|error| HypergraphError::PathNotAccessible(error, t))?;
    let metadata = file.metadata().await.map_err(|_| HypergraphError::File)?;

    if metadata.len() != 0 {
        let mut contents = vec![];
        file.read_to_end(&mut contents)
            .await
            .map_err(|_| HypergraphError::File)?;

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

    write(path, bytes).await.map_err(|_| HypergraphError::File)
}

pub(crate) async fn read_data_from_file<V, HE>(
    entity_kind: &EntityKind,
    uuid: &Uuid,
    paths: Arc<Paths>,
) -> Result<HashMap<Uuid, Entity<V, HE>>, HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    let chunk_path = get_chunk_path(entity_kind, paths.clone(), uuid).await?;

    read_from_file(chunk_path)
        .await?
        .ok_or(HypergraphError::File)
}

async fn write_data_to_file<V, HE>(
    entity_kind: &EntityKind,
    uuid: &Uuid,
    data: HashMap<Uuid, Entity<V, HE>>,
    paths: Arc<Paths>,
) -> Result<(), HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    let chunk_path = get_chunk_path(entity_kind, paths.clone(), uuid).await?;

    write_to_file(&data, chunk_path).await
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
    let mut data = read_data_from_file::<V, HE>(entity_kind, uuid, paths.clone()).await?;
    let entity = data.get_mut(uuid).ok_or(HypergraphError::EntityUpdate)?;

    match entity_relation {
        EntityRelation::Hyperedge(vertices) => match entity {
            Entity::Hyperedge(hyperedge) => {
                hyperedge.vertices = vertices.to_owned();
            }
            Entity::Vertex(_) => unreachable!(),
        },
        EntityRelation::Vertex(hyperedges) => match entity {
            Entity::Hyperedge(_) => unreachable!(),
            Entity::Vertex(vertex) => {
                vertex.hyperedges = hyperedges.to_owned();
            }
        },
    };

    write_data_to_file(entity_kind, uuid, data, paths).await
}

pub(crate) async fn write_weight_to_file<V, HE>(
    uuid: &Uuid,
    entity_weight: &EntityWeight<V, HE>,
    paths: Arc<Paths>,
) -> Result<(), HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    let entity_kind: EntityKind = entity_weight.into();
    let mut data = read_data_from_file::<V, HE>(&entity_kind, uuid, paths.clone()).await?;

    match entity_weight {
        EntityWeight::Hyperedge(weight) => {
            data.insert(*uuid, Entity::Hyperedge(Hyperedge::new(weight.to_owned())));
        }
        EntityWeight::Vertex(weight) => {
            data.insert(*uuid, Entity::Vertex(Vertex::new(weight.to_owned())));
        }
    };

    write_data_to_file(&entity_kind, uuid, data, paths).await
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
    let mut data = read_data_from_file::<V, HE>(entity_kind, uuid, paths.clone()).await?;

    data.remove(uuid).ok_or(HypergraphError::EntityNotFound)?;

    write_data_to_file(entity_kind, uuid, data, paths).await
}
