use std::{collections::HashMap, fmt::Debug, path::PathBuf, sync::Arc};

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};
use uuid::Uuid;

use crate::{
    entities::{Entity, EntityKind, EntityRelation, EntityWeight, Hyperedge, Vertex},
    errors::HypergraphError,
};

#[derive(Debug)]
pub(crate) struct Paths {
    pub(crate) hyperedges: PathBuf,
    pub(crate) vertices: PathBuf,
    pub(crate) root: PathBuf,
}

struct FileWithData<V, HE>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    file: File,
    data: HashMap<Uuid, Entity<V, HE>>,
}

async fn get_file_with_data<V, HE>(
    entity_kind: EntityKind,
    paths: Arc<Paths>,
) -> Result<FileWithData<V, HE>, HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    let mut file = OpenOptions::new()
        .create(true) //TODO: remove unecessary check file logic
        .truncate(false)
        .read(true)
        .write(true)
        .open(match entity_kind {
            EntityKind::Hyperedge => &paths.hyperedges,
            EntityKind::Vertex => &paths.vertices,
        })
        .await
        .map_err(|_| HypergraphError::PathNotAccessible)?;
    let metadata = file.metadata().await.map_err(|_| HypergraphError::File)?;
    let mut data: HashMap<Uuid, Entity<V, HE>> = HashMap::default();

    if metadata.len() != 0 {
        let mut contents = vec![];
        file.read_to_end(&mut contents)
            .await
            .map_err(|_| HypergraphError::File)?;

        data = deserialize(&contents).map_err(|_| HypergraphError::Deserialization)?;
    }

    Ok(FileWithData { file, data })
}

async fn serialize_and_sync<V, HE>(
    FileWithData { mut file, data }: FileWithData<V, HE>,
) -> Result<(), HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    let bytes = serialize(&data).map_err(|_| HypergraphError::Serialization)?;
    dbg!(123, data.clone());
    file.write_all(&bytes)
        .await
        .map_err(|_| HypergraphError::File)?;
    file.sync_data().await.map_err(|_| HypergraphError::File)
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
    let entity_kind = entity_relation.into();
    let FileWithData { file, mut data }: FileWithData<V, HE> =
        get_file_with_data(entity_kind, paths).await?;
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

    serialize_and_sync(FileWithData { file, data }).await
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
    let entity_kind = entity_weight.into();
    let FileWithData { file, mut data }: FileWithData<V, HE> =
        get_file_with_data(entity_kind, paths).await?;

    match entity_weight {
        EntityWeight::Hyperedge(weight) => {
            data.insert(*uuid, Entity::Hyperedge(Hyperedge::new(weight.to_owned())));
        }
        EntityWeight::Vertex(weight) => {
            data.insert(*uuid, Entity::Vertex(Vertex::new(weight.to_owned())));
        }
    };

    serialize_and_sync(FileWithData { file, data }).await
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
    let FileWithData { file, mut data }: FileWithData<V, HE> =
        get_file_with_data(*entity_kind, paths).await?;

    data.remove(uuid).ok_or(HypergraphError::EntityNotFound)?;

    serialize_and_sync(FileWithData { file, data }).await
}
