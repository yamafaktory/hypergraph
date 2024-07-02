use std::{
    fmt::Debug,
    fs::{write, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use tokio::{spawn, sync::Mutex, task::spawn_blocking};
use uuid::Uuid;

use crate::{
    chunk::ChunkManager,
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

pub(crate) async fn read_from_file<D, P>(path: P) -> Result<Option<D>, HypergraphError>
where
    D: Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
    P: AsRef<Path> + Send + 'static,
{
    let path_buf = path.as_ref().to_path_buf();

    spawn_blocking(move || {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|error| HypergraphError::PathNotAccessible(error, path_buf))?;
        let metadata = file.metadata().map_err(HypergraphError::File)?;

        if metadata.len() != 0 {
            let mut contents = vec![];

            file.read_to_end(&mut contents)
                .map_err(HypergraphError::File)?;

            return deserialize(&contents)
                .map_err(|_| HypergraphError::Deserialization)
                .map(Some);
        }

        Ok(None)
    })
    .await
    .map_err(|_| HypergraphError::Processing)?
}

pub(crate) async fn write_to_file<D, P>(data: &D, path: P) -> Result<(), HypergraphError>
where
    D: Serialize,
    P: AsRef<Path> + Send + 'static,
{
    let bytes = serialize(&data).map_err(|_| HypergraphError::Serialization)?;

    spawn_blocking(move || write(path, bytes).map_err(|_| HypergraphError::Processing))
        .await
        .map_err(|_| HypergraphError::Processing)?
}

pub(crate) async fn read_entity_from_file<V, HE>(
    entity_kind: EntityKind,
    uuid: Uuid,
    paths: Arc<Paths>,
    chunk_manager: Arc<Mutex<ChunkManager>>,
) -> Result<Option<Entity<V, HE>>, HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
{
    let handle = spawn(async move {
        let mut lock = chunk_manager.lock().await;
        let entity = lock.read_op::<V, HE>(&entity_kind, paths, &uuid).await?;

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
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
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
    chunk_manager: Arc<Mutex<ChunkManager>>,
) -> Result<(), HypergraphError>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
{
    spawn(async move {
        let mut lock = chunk_manager.lock().await;
        let entity_kind = (&entity_weight).into();

        lock.create_op(
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
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
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
