#[doc(hidden)]
pub mod actors;
#[doc(hidden)]
pub mod collections;
#[doc(hidden)]
pub mod defaults;
#[doc(hidden)]
pub mod entities;
#[doc(hidden)]
pub mod errors;
#[doc(hidden)]
pub mod file;
#[doc(hidden)]
pub mod operations;

use std::{borrow::Borrow, fmt::Debug, path::Path, sync::Arc};

use bincode::deserialize;
use errors::HypergraphError;
use futures::FutureExt;
use quick_cache::sync::Cache;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{create_dir_all, try_exists, File, OpenOptions},
    io::AsyncReadExt,
};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use self::{
    defaults::{HYPEREDGES_CACHE_SIZE, VERTICES_CACHE_SIZE},
    entities::{Entity, Hyperedge, Vertex},
};
use crate::{
    actors::ActorHandle,
    collections::HashMap,
    defaults::{HYPEREDGES_DB, VERTICES_DB},
    entities::{EntityKind, EntityRelation, EntityWeight},
    file::write_to_file,
    operations::{ReadOp, WriteOp},
};

#[derive(Debug)]
struct MemoryCacheState<V, HE> {
    hyperedges: Cache<Uuid, Hyperedge<HE>>,
    vertices: Cache<Uuid, Vertex<V>>,
}

impl<V, HE> MemoryCacheState<V, HE>
where
    V: Clone,
    HE: Clone,
{
    fn new(hyperedges_cache_size: usize, vertices_cache_size: usize) -> Self {
        Self {
            hyperedges: Cache::new(hyperedges_cache_size),
            vertices: Cache::new(vertices_cache_size),
        }
    }
}

#[derive(Debug)]
struct MemoryCache<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    state: Arc<MemoryCacheState<V, HE>>,
    reader: ActorHandle<Arc<MemoryCacheState<V, HE>>, ReadOp, Option<Entity<V, HE>>>,
    writer: ActorHandle<Arc<MemoryCacheState<V, HE>>, Arc<WriteOp<V, HE>>, Uuid>,
}

impl<V, HE> MemoryCache<V, HE>
where
    V: Clone + Debug + Send + Sync + 'static,
    HE: Clone + Debug + Send + Sync + 'static,
{
    async fn start(
        hyperedges_cache_size: usize,
        vertices_cache_size: usize,
    ) -> Result<Self, HypergraphError> {
        info!("Starting MemoryCache");

        let state = Arc::new(MemoryCacheState::new(
            hyperedges_cache_size,
            vertices_cache_size,
        ));
        let reader = Self::get_reader(state.clone()).await;
        let writer = Self::get_writer(state.clone()).await;

        Ok(Self {
            reader,
            state,
            writer,
        })
    }

    #[tracing::instrument]
    async fn get_reader(
        state: Arc<MemoryCacheState<V, HE>>,
    ) -> ActorHandle<Arc<MemoryCacheState<V, HE>>, ReadOp, Option<Entity<V, HE>>> {
        ActorHandle::new(state, &|state, ReadOp(uuid, entity_kind)| {
            async move {
                debug!("Reading from in-memory cache.");

                let entity = match entity_kind {
                    EntityKind::Hyperedge => state.hyperedges.get(&uuid).map(Entity::Hyperedge),
                    EntityKind::Vertex => state.vertices.get(&uuid).map(Entity::Vertex),
                };

                Ok(entity)
            }
            .boxed()
        })
    }

    #[instrument]
    async fn get_writer(
        state: Arc<MemoryCacheState<V, HE>>,
    ) -> ActorHandle<Arc<MemoryCacheState<V, HE>>, Arc<WriteOp<V, HE>>, Uuid> {
        ActorHandle::new(state, &|state, write_op| {
            async move {
                debug!("Writing to in-memory cache.");

                match write_op.borrow() {
                    WriteOp::Create(uuid, entity_weight) => {
                        match entity_weight {
                            EntityWeight::Hyperedge(weight) => state
                                .hyperedges
                                .insert(*uuid, Hyperedge::new(weight.to_owned())),
                            EntityWeight::Vertex(weight) => {
                                state.vertices.insert(*uuid, Vertex::new(weight.to_owned()))
                            }
                        }

                        Ok(*uuid)
                    }
                    WriteOp::Delete(uuid, entity_kind) => {
                        match entity_kind {
                            EntityKind::Hyperedge => {
                                state.hyperedges.remove(uuid);
                            }
                            EntityKind::Vertex => {
                                state.vertices.remove(uuid);
                            }
                        };

                        Ok(*uuid)
                    }
                    WriteOp::UpdateRelation(uuid, relation) => match relation {
                        EntityRelation::Hyperedge(vertices) => {
                            if let Some(mut hyperedge) = state.hyperedges.get(uuid) {
                                hyperedge.vertices = vertices.to_vec();

                                return state
                                    .hyperedges
                                    .replace(*uuid, hyperedge, false)
                                    .map_err(|_| HypergraphError::EntityUpdate)
                                    .map(|_| *uuid);
                            };

                            Err(HypergraphError::EntityUpdate)
                        }
                        EntityRelation::Vertex(hyperedges) => {
                            if let Some(mut vertex) = state.vertices.get(uuid) {
                                vertex.hyperedges = hyperedges.to_owned();

                                return state
                                    .vertices
                                    .replace(*uuid, vertex, false)
                                    .map_err(|_| HypergraphError::EntityUpdate)
                                    .map(|_| *uuid);
                            };

                            Err(HypergraphError::EntityUpdate)
                        }
                    },
                    WriteOp::UpdateWeight(uuid, weight) => match weight {
                        EntityWeight::Hyperedge(weight) => {
                            if let Some(mut hyperedge) = state.hyperedges.get(uuid) {
                                hyperedge.weight = weight.to_owned();

                                return state
                                    .hyperedges
                                    .replace(*uuid, hyperedge, false)
                                    .map_err(|_| HypergraphError::EntityUpdate)
                                    .map(|_| *uuid);
                            };

                            Err(HypergraphError::EntityUpdate)
                        }
                        EntityWeight::Vertex(weight) => {
                            if let Some(mut vertex) = state.vertices.get(uuid) {
                                vertex.weight = weight.to_owned();

                                return state
                                    .vertices
                                    .replace(*uuid, vertex, false)
                                    .map_err(|_| HypergraphError::EntityUpdate)
                                    .map(|_| *uuid);
                            };

                            Err(HypergraphError::EntityUpdate)
                        }
                    },
                }
            }
            .boxed()
        })
    }
}

#[derive(Debug)]
struct IOManager<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    hyperedges_db_path: Arc<Path>,
    vertices_db_path: Arc<Path>,
    path: Arc<Path>,
    reader: Option<ActorHandle<Arc<Path>, ReadOp, Option<Entity<V, HE>>>>,
    writer: Option<ActorHandle<(Arc<Path>, Arc<Path>), Arc<WriteOp<V, HE>>, ()>>,
}

impl<V, HE> IOManager<V, HE>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Serialize + Sync + 'static,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Serialize + Sync + 'static,
{
    #[instrument]
    async fn new<P>(path: P) -> Result<Self, HypergraphError>
    where
        P: AsRef<Path> + Copy + Debug,
    {
        info!("Creating new IOManager");

        let vertices_db_path = path.as_ref().join(VERTICES_DB).into();
        let hyperedges_db_path = path.as_ref().join(HYPEREDGES_DB).into();
        let path = path.as_ref().to_path_buf().into();

        Ok(Self {
            hyperedges_db_path,
            vertices_db_path,
            path,
            reader: None,
            writer: None,
        })
    }

    async fn start(&mut self) -> Result<(), HypergraphError> {
        let reader = self.get_reader().await;
        let writer = self.get_writer().await;

        self.reader = Some(reader);
        self.writer = Some(writer);

        let path = self.path.clone();

        match try_exists(path.clone()).await {
            Ok(true) => {
                debug!("Path already exists");

                if let Ok(exists) = try_exists(self.vertices_db_path.clone()).await {
                    if !exists {
                        self.create_entity_db(self.vertices_db_path.clone()).await?;
                        debug!("Vertices storage file was not found and has been created");
                    }
                } else {
                    return Err(HypergraphError::DatabasesCreation);
                }

                if let Ok(exists) = try_exists(self.hyperedges_db_path.clone()).await {
                    if !exists {
                        self.create_entity_db(self.hyperedges_db_path.clone())
                            .await?;
                        debug!("Hyperedges storage file was not found and has been created");
                    }
                } else {
                    return Err(HypergraphError::DatabasesCreation);
                }
            }
            Ok(false) => {
                debug!("Path does not exist");

                create_dir_all(path.clone())
                    .await
                    .map_err(|_| HypergraphError::PathCreation)?;

                self.create_entity_db(self.vertices_db_path.clone()).await?;
                self.create_entity_db(self.hyperedges_db_path.clone())
                    .await?;
            }
            Err(_) => return Err(HypergraphError::PathNotAccessible),
        };

        Ok(())
    }

    async fn create_entity_db(&self, path: Arc<Path>) -> Result<(), HypergraphError> {
        File::create(path)
            .await
            .map_err(|_| HypergraphError::DatabasesCreation)?
            .sync_data()
            .await
            .map_err(|_| HypergraphError::DatabasesCreation)?;

        Ok(())
    }

    #[instrument]
    async fn get_reader(&self) -> ActorHandle<Arc<Path>, ReadOp, Option<Entity<V, HE>>> {
        ActorHandle::new(self.vertices_db_path.clone(), &|vertices_db_path,
                                                          ReadOp(
            uuid,
            entity_kind,
        )| {
            async move {
                debug!("Reading from disk.");

                let mut entity = None;

                match entity_kind {
                    EntityKind::Hyperedge => {}
                    EntityKind::Vertex => {
                        let mut file = OpenOptions::new()
                            .read(true)
                            .open(vertices_db_path.clone())
                            .await
                            .map_err(|_| HypergraphError::PathNotAccessible)?;
                        let metadata = file.metadata().await.map_err(|_| HypergraphError::File)?;

                        if metadata.len() != 0 {
                            let mut contents = vec![];
                            file.read_to_end(&mut contents)
                                .await
                                .map_err(|_| HypergraphError::File)?;

                            let data: HashMap<Uuid, Entity<V, HE>> = deserialize(&contents)
                                .map_err(|_| HypergraphError::Deserialization)?;

                            entity = data.get(&uuid).cloned();
                        }
                    }
                };

                Ok(entity)
            }
            .boxed()
        })
    }

    #[instrument]
    async fn get_writer(&self) -> ActorHandle<(Arc<Path>, Arc<Path>), Arc<WriteOp<V, HE>>, ()> {
        ActorHandle::new(
            (
                self.hyperedges_db_path.clone(),
                self.vertices_db_path.clone(),
            ),
            &|paths, write_op| {
                async move {
                    match write_op.borrow() {
                        WriteOp::Create(uuid, entity_weight) => {
                            write_to_file(uuid, entity_weight, paths).await?;
                        }
                        WriteOp::Delete(_, _) => todo!(),
                        WriteOp::UpdateRelation(_, _) => todo!(),
                        WriteOp::UpdateWeight(uuid, weight) => todo!(),
                    };

                    debug!("Writing to disk.");

                    Ok(())
                }
                .boxed()
            },
        )
    }
}

#[derive(Clone, Debug)]
struct Handles<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    io_manager_reader: ActorHandle<Arc<Path>, ReadOp, Option<Entity<V, HE>>>,
    io_manager_writer: ActorHandle<(Arc<Path>, Arc<Path>), Arc<WriteOp<V, HE>>, ()>,
    memory_cache_reader: ActorHandle<Arc<MemoryCacheState<V, HE>>, ReadOp, Option<Entity<V, HE>>>,
    memory_cache_writer: ActorHandle<Arc<MemoryCacheState<V, HE>>, Arc<WriteOp<V, HE>>, Uuid>,
}

impl<V, HE> Handles<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    fn new(
        io_manager_reader: ActorHandle<Arc<Path>, ReadOp, Option<Entity<V, HE>>>,
        io_manager_writer: ActorHandle<(Arc<Path>, Arc<Path>), Arc<WriteOp<V, HE>>, ()>,
        memory_cache_reader: ActorHandle<
            Arc<MemoryCacheState<V, HE>>,
            ReadOp,
            Option<Entity<V, HE>>,
        >,
        memory_cache_writer: ActorHandle<Arc<MemoryCacheState<V, HE>>, Arc<WriteOp<V, HE>>, Uuid>,
    ) -> Self {
        Self {
            io_manager_reader,
            io_manager_writer,
            memory_cache_reader,
            memory_cache_writer,
        }
    }
}

#[derive(Debug)]
struct EntityManager<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    reader: ActorHandle<Handles<V, HE>, ReadOp, Option<Entity<V, HE>>>,
    writer: ActorHandle<Handles<V, HE>, Arc<WriteOp<V, HE>>, Uuid>,
}

impl<V, HE> EntityManager<V, HE>
where
    V: Clone + Debug + Send + Sync + 'static,
    HE: Clone + Debug + Send + Sync + 'static,
{
    async fn start(handles: Handles<V, HE>) -> Result<Self, HypergraphError> {
        info!("Starting EntityManager");

        let reader = Self::get_reader(handles.clone()).await;
        let writer = Self::get_writer(handles.clone()).await;

        Ok(Self { reader, writer })
    }

    #[instrument]
    async fn get_reader(
        handles: Handles<V, HE>,
    ) -> ActorHandle<Handles<V, HE>, ReadOp, Option<Entity<V, HE>>> {
        ActorHandle::new(handles, &|handles, read_op| {
            async move {
                debug!("Reading with entity manager.");

                let mut entity = handles.memory_cache_reader.process(read_op).await?;

                // We use a read-through strategy here.
                // This is a cache miss and we need to sync the cache with the data on disk.
                if entity.is_none() {
                    entity = handles.io_manager_reader.process(read_op).await?;

                    if let Some(ref entity) = entity {
                        handles
                            .memory_cache_writer
                            .process(Arc::new(WriteOp::Create(read_op.get_uuid(), entity.into())))
                            .await?;
                    };
                }

                Ok(entity)
            }
            .boxed()
        })
    }

    #[instrument]
    async fn get_writer(
        handles: Handles<V, HE>,
    ) -> ActorHandle<Handles<V, HE>, Arc<WriteOp<V, HE>>, Uuid> {
        ActorHandle::new(handles, &|handles, write_op| {
            async move {
                debug!("Writing with entity manager.");

                match write_op.borrow() {
                    WriteOp::Create(..) => {
                        let uuid = handles
                            .memory_cache_writer
                            .process(write_op.clone())
                            .await?;

                        // We don't wait for the IOManager to respond since we use a
                        // write-through strategy.
                        handles.io_manager_writer.process(write_op.clone()).await?;

                        Ok(uuid)
                    }
                    WriteOp::Delete(uuid, _) => {
                        handles
                            .memory_cache_writer
                            .process(write_op.clone())
                            .await?;

                        handles.io_manager_writer.process(write_op.clone()).await?;

                        Ok(*uuid)
                    }
                    WriteOp::UpdateWeight(uuid, weight) => {
                        // Try to read the entity from cache.
                        let entity = handles
                            .memory_cache_reader
                            .process(ReadOp(*uuid, weight.into()))
                            .await?;

                        // Here we have a cache hit.
                        // Either update or create the cache.
                        let write_op = Arc::new(if entity.is_some() {
                            WriteOp::UpdateWeight(*uuid, weight.clone())
                        } else {
                            WriteOp::Create(*uuid, weight.clone())
                        });

                        handles.memory_cache_writer.process(write_op).await?;

                        // Sync the data on disk.
                        handles
                            .io_manager_writer
                            .process(Arc::new(WriteOp::UpdateWeight(*uuid, weight.clone())))
                            .await?;

                        Ok(*uuid)
                    }
                    WriteOp::UpdateRelation(uuid, entity) => {
                        todo!()
                    }
                }
            }
            .boxed()
        })
    }
}

/// A directed hypergraph composed of generic vertices and hyperedges.
#[derive(Debug)]
pub struct Hypergraph<V, HE>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Serialize + Sync,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Serialize + Sync,
{
    entity_manager: EntityManager<V, HE>,
    io_manager: IOManager<V, HE>,
    memory_cache: MemoryCache<V, HE>,
}

impl<V, HE> Hypergraph<V, HE>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
    HE: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
{
    pub async fn init<P>(path: P) -> Result<Self, HypergraphError>
    where
        P: AsRef<Path> + Copy + Debug,
    {
        Self::init_with_config(path, HYPEREDGES_CACHE_SIZE, VERTICES_CACHE_SIZE).await
    }

    pub async fn init_with_config<P>(
        path: P,
        hyperedges_cache_size: usize,
        vertices_cache_size: usize,
    ) -> Result<Self, HypergraphError>
    where
        P: AsRef<Path> + Copy + Debug,
    {
        info!("Init Hypergraph");

        let mut io_manager = IOManager::new(path).await?;
        let memory_cache = MemoryCache::start(hyperedges_cache_size, vertices_cache_size).await?;

        io_manager.start().await?;

        Ok(Self {
            entity_manager: EntityManager::start(Handles::new(
                // We can safely unwrap here as we've just created the handles.
                io_manager.reader.clone().unwrap(),
                io_manager.writer.clone().unwrap(),
                memory_cache.reader.clone(),
                memory_cache.writer.clone(),
            ))
            .await?,
            io_manager,
            memory_cache,
        })
    }

    #[instrument]
    pub async fn create_vertex(&self, weight: V) -> Result<Uuid, HypergraphError> {
        let uuid = Uuid::now_v7();

        self.entity_manager
            .writer
            .process(Arc::new(WriteOp::Create(
                uuid,
                EntityWeight::Vertex(weight),
            )))
            .await?;

        debug!("Vertex {} created", uuid.to_string());

        Ok(uuid)
    }

    // #[instrument]
    // pub async fn update_vertex_weight(&self, uuid: Uuid, weight: V) -> Result<(), HypergraphError> {
    //     self.entity_manager
    //         .writer
    //         .process(Op::UpdateWeight {
    //             uuid,
    //             weight: EntityWeight::Vertex(weight),
    //         })
    //         .await?;
    //
    //     // debug!("Vertex {} updated", uuid.to_string());
    //
    //     Ok(())
    // }

    #[instrument]
    pub async fn get_vertex(&self, uuid: Uuid) -> Result<Option<V>, HypergraphError> {
        let vertex = self
            .entity_manager
            .reader
            .process(ReadOp(uuid, EntityKind::Vertex))
            .await?;

        if let Some(vertex) = vertex {
            debug!("Vertex {} found", uuid.to_string());

            match vertex {
                Entity::Hyperedge(_) => unreachable!(),
                Entity::Vertex(vertex) => Ok(Some(vertex.weight)),
            }
        } else {
            debug!("Vertex {} not found", uuid.to_string());

            Ok(None)
        }
    }

    // #[instrument]
    // pub async fn create_hyperedge(
    //     &self,
    //     weight: HE,
    // ) -> Result<Uuid, HypergraphError> {
    //     let uuid = self
    //         .entity_manager
    //         .writer
    //         .process(Entity::Hyperedge(Hyperedge::new(vertices, weight)))
    //         .await
    //         .map_err(|_| HypergraphError::HyperedgeInsertion)?;
    //
    //     debug!("Hyperedge {} added", uuid.to_string());
    //
    //     Ok(uuid)
    // }
}
