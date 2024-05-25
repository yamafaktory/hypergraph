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

use errors::HypergraphError;
use futures::FutureExt;
use quick_cache::sync::Cache;
use serde::{Deserialize, Serialize};
use tokio::{fs::create_dir_all, try_join};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::{
    actors::ActorHandle,
    defaults::{DB_EXT, HYPEREDGES_CACHE_SIZE, HYPEREDGES_DB, VERTICES_CACHE_SIZE, VERTICES_DB},
    entities::{Entity, EntityKind, EntityRelation, EntityWeight, Hyperedge, Vertex},
    file::{
        read_entity_from_file, remove_entity_from_file, write_relation_to_file,
        write_weight_to_file, Paths,
    },
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

#[allow(clippy::type_complexity)]
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

    #[allow(clippy::type_complexity)]
    #[tracing::instrument]
    async fn get_reader(
        state: Arc<MemoryCacheState<V, HE>>,
    ) -> ActorHandle<Arc<MemoryCacheState<V, HE>>, ReadOp, Option<Entity<V, HE>>> {
        ActorHandle::new(state, &|state, read_op| {
            async move {
                info!("Reading from in-memory cache {}", read_op);

                let ReadOp(uuid, entity_kind) = read_op;
                let entity = match entity_kind {
                    EntityKind::Hyperedge => state.hyperedges.get(&uuid).map(Entity::Hyperedge),
                    EntityKind::Vertex => state.vertices.get(&uuid).map(Entity::Vertex),
                };

                info!(
                    "{} {}",
                    read_op,
                    if entity.is_some() {
                        "found"
                    } else {
                        "not found"
                    }
                );

                Ok(entity)
            }
            .boxed()
        })
    }

    #[allow(clippy::type_complexity)]
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
                                hyperedges.clone_into(&mut vertex.hyperedges);

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
                                weight.clone_into(&mut hyperedge.weight);

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
                                weight.clone_into(&mut vertex.weight);

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

#[allow(clippy::type_complexity)]
#[derive(Debug)]
struct IOManager<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    paths: Arc<Paths>,
    reader: Option<ActorHandle<Arc<Paths>, ReadOp, Option<Entity<V, HE>>>>,
    writer: Option<ActorHandle<Arc<Paths>, Arc<WriteOp<V, HE>>, ()>>,
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

        let path = path.as_ref();

        let mut hyperedges = path.join(HYPEREDGES_DB);
        hyperedges.set_extension(DB_EXT);

        let mut vertices = path.join(VERTICES_DB);
        vertices.set_extension(DB_EXT);

        Ok(Self {
            paths: Arc::new(Paths {
                hyperedges,
                vertices,
                root: path.to_path_buf(),
            }),
            reader: None,
            writer: None,
        })
    }

    async fn start(&mut self) -> Result<(), HypergraphError> {
        let reader = self.get_reader().await;
        let writer = self.get_writer().await;

        create_dir_all(&self.paths.root)
            .await
            .map_err(|_| HypergraphError::PathCreation)?;

        self.reader = Some(reader);
        self.writer = Some(writer);

        Ok(())
    }

    #[instrument]
    async fn get_reader(&self) -> ActorHandle<Arc<Paths>, ReadOp, Option<Entity<V, HE>>> {
        ActorHandle::new(self.paths.clone(), &|paths, read_op| {
            async move {
                info!("Reading from disk {}", read_op);

                let ReadOp(uuid, entity_kind) = read_op;
                let entity = read_entity_from_file(entity_kind, uuid, paths).await?;

                info!(
                    "{} {}",
                    read_op,
                    if entity.is_some() {
                        "found"
                    } else {
                        "not found"
                    }
                );

                Ok(entity)
            }
            .boxed()
        })
    }

    #[instrument]
    async fn get_writer(&self) -> ActorHandle<Arc<Paths>, Arc<WriteOp<V, HE>>, ()> {
        ActorHandle::new(self.paths.clone(), &|paths, write_op| {
            async move {
                debug!("Writing to disk {}.", write_op);
                match (*write_op).clone() {
                    WriteOp::Create(uuid, entity_weight) => {
                        write_weight_to_file(uuid, entity_weight, paths, false).await?;
                    }
                    WriteOp::UpdateWeight(uuid, entity_weight) => {
                        write_weight_to_file(uuid, entity_weight, paths, true).await?;
                    }
                    WriteOp::Delete(uuid, entity_kind) => {
                        remove_entity_from_file::<V, HE>(uuid, entity_kind, paths).await?
                    }
                    WriteOp::UpdateRelation(uuid, entity_relation) => {
                        write_relation_to_file::<V, HE>(uuid, entity_relation, paths).await?;
                    }
                };

                Ok(())
            }
            .boxed()
        })
    }
}

#[allow(clippy::type_complexity)]
#[derive(Clone, Debug)]
struct Handles<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    io_manager_reader: ActorHandle<Arc<Paths>, ReadOp, Option<Entity<V, HE>>>,
    io_manager_writer: ActorHandle<Arc<Paths>, Arc<WriteOp<V, HE>>, ()>,
    memory_cache_reader: ActorHandle<Arc<MemoryCacheState<V, HE>>, ReadOp, Option<Entity<V, HE>>>,
    memory_cache_writer: ActorHandle<Arc<MemoryCacheState<V, HE>>, Arc<WriteOp<V, HE>>, Uuid>,
}

#[allow(clippy::type_complexity)]
impl<V, HE> Handles<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    fn new(
        io_manager_reader: ActorHandle<Arc<Paths>, ReadOp, Option<Entity<V, HE>>>,
        io_manager_writer: ActorHandle<Arc<Paths>, Arc<WriteOp<V, HE>>, ()>,
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
                        // We don't wait for the IOManager to respond since we use a
                        // write-through strategy.
                        let (uuid, _) = try_join!(
                            handles.memory_cache_writer.process(write_op.clone()),
                            handles
                                .io_manager_writer
                                .process_no_response(write_op.clone())
                        )?;

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

    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
    pub async fn delete_vertex(&self, uuid: Uuid) -> Result<Uuid, HypergraphError> {
        self.entity_manager
            .writer
            .process(Arc::new(WriteOp::Delete(uuid, EntityKind::Vertex)))
            .await?;

        debug!("Vertex {} deleted", uuid.to_string());

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

    #[instrument(skip(self))]
    pub async fn get_vertex(&self, uuid: Uuid) -> Result<Option<V>, HypergraphError> {
        let entity = self
            .entity_manager
            .reader
            .process(ReadOp(uuid, EntityKind::Vertex))
            .await?;

        if let Some(entity) = entity {
            debug!("Vertex {} found", uuid.to_string());

            match entity {
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
