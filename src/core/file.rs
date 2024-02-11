use std::{collections::HashMap, fmt::Debug, path::Path, sync::Arc};

use bincode::{deserialize, serialize};
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt},
};
use uuid::Uuid;

use crate::{
    entities::{EntityKind, EntityWeight, Hyperedge, Vertex},
    errors::HypergraphError,
};

pub(crate) async fn write_to_file<V, HE>(
    uuid: &Uuid,
    entity_weight: &EntityWeight<V, HE>,
    paths: (Arc<Path>, Arc<Path>),
) -> Result<(), HypergraphError>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    let entity_kind = entity_weight.into();
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(match entity_kind {
            EntityKind::Hyperedge => paths.0,
            EntityKind::Vertex => paths.1,
        })
        .await
        .map_err(|_| HypergraphError::PathNotAccessible)?;
    let metadata = file.metadata().await.map_err(|_| HypergraphError::File)?;
    let mut data = HashMap::default();

    if metadata.len() != 0 {
        let mut contents = vec![];
        file.read_to_end(&mut contents)
            .await
            .map_err(|_| HypergraphError::File)?;

        data = deserialize(&contents).map_err(|_| HypergraphError::Deserialization)?;
    }

    match entity_weight {
        EntityWeight::Hyperedge(weight) => {
            data.insert(*uuid, Hyperedge::new(weight));
        }
        EntityWeight::Vertex(weight) => {
            data.insert(*uuid, Vertex::new(weight));
        }
    };

    let bytes = serialize(&data).map_err(|_| HypergraphError::Serialization)?;

    file.write_all(&bytes)
        .await
        .map_err(|_| HypergraphError::File)?;
    file.sync_data().await.map_err(|_| HypergraphError::File)?;

    Ok(())
}
