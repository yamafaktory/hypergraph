use std::{fmt::Debug, path::PathBuf, sync::Arc};

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

pub(crate) async fn read_data_from_file<V, HE>(
    entity_kind: EntityKind,
    paths: Arc<Paths>,
) -> Result<HashMap<Uuid, Entity<V, HE>>, HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    let mut file = OpenOptions::new()
        // .create(true) //TODO: remove unecessary check file logic
        .read(true)
        // .truncate(true)
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

    Ok(data)
}

async fn write_data_to_file<V, HE>(
    data: HashMap<Uuid, Entity<V, HE>>,
    entity_kind: EntityKind,
    paths: Arc<Paths>,
) -> Result<(), HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize,
{
    let bytes = serialize(&data).map_err(|_| HypergraphError::Serialization)?;

    write(
        match entity_kind {
            EntityKind::Hyperedge => &paths.hyperedges,
            EntityKind::Vertex => &paths.vertices,
        },
        bytes,
    )
    .await
    .map_err(|_| HypergraphError::File)
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
    let mut data = read_data_from_file::<V, HE>(entity_kind, paths.clone()).await?;
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

    write_data_to_file(data, entity_kind, paths).await
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
    let mut data = read_data_from_file::<V, HE>(entity_kind, paths.clone()).await?;

    match entity_weight {
        EntityWeight::Hyperedge(weight) => {
            data.insert(*uuid, Entity::Hyperedge(Hyperedge::new(weight.to_owned())));
        }
        EntityWeight::Vertex(weight) => {
            data.insert(*uuid, Entity::Vertex(Vertex::new(weight.to_owned())));
        }
    };

    write_data_to_file(data, entity_kind, paths).await
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
    let mut data = read_data_from_file::<V, HE>(*entity_kind, paths.clone()).await?;

    data.remove(uuid).ok_or(HypergraphError::EntityNotFound)?;

    write_data_to_file(data, *entity_kind, paths).await
}
