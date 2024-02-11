#[doc(hidden)]
pub mod actors;
#[doc(hidden)]
pub mod errors;

use std::{
    borrow::Borrow,
    collections::{HashMap as DefaultHashMap, HashSet as DefaultHashSet},
    fmt::Debug,
    path::Path,
    sync::Arc,
};

use ahash::RandomState;
use bincode::{deserialize, serialize};
use errors::HypergraphError;
use futures::FutureExt;
use quick_cache::sync::Cache;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{create_dir_all, try_exists, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::actors::ActorHandle;

static VERTICES_CACHE_SIZE: usize = 10_000;
static VERTICES_DB: &str = "vertices.db";
static HYPEREDGES_DB: &str = "hyperedges.db";
static HYPEREDGES_CACHE_SIZE: usize = 10_000;

#[derive(Clone, Copy, Debug)]
enum EntityKind {
    Hyperedge,
    Vertex,
}

#[derive(Clone, Debug)]
enum Entity<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    Hyperedge(Hyperedge<HE>),
    Vertex(Vertex<V>),
}

#[derive(Clone, Debug)]
enum EntityRelation {
    Hyperedge(Vec<Uuid>),
    Vertex(HashSet<Uuid>),
}

#[derive(Clone, Debug)]
enum EntityWeight<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    Hyperedge(HE),
    Vertex(V),
}

impl<V, HE> From<&EntityWeight<V, HE>> for EntityKind
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    fn from(entity_weight: &EntityWeight<V, HE>) -> Self {
        match entity_weight {
            EntityWeight::Hyperedge(_) => EntityKind::Hyperedge,
            EntityWeight::Vertex(_) => EntityKind::Vertex,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ReadOp(Uuid, EntityKind);

#[derive(Clone, Debug)]
enum WriteOp<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    Create(Uuid, EntityWeight<V, HE>),
    Delete(Uuid, EntityKind),
    UpdateRelation(Uuid, EntityRelation),
    UpdateWeight(Uuid, EntityWeight<V, HE>),
}

pub(crate) type HashSet<K> = DefaultHashSet<K, RandomState>;
pub(crate) type HashMap<K, V> = DefaultHashMap<K, V, RandomState>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Vertex<V> {
    hyperedges: HashSet<Uuid>,
    weight: V,
}

impl<V> Vertex<V> {
    fn new(weight: V) -> Self {
        Self {
            hyperedges: HashSet::default(),
            weight,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Hyperedge<HE> {
    vertices: Vec<Uuid>,
    weight: HE,
}

impl<HE> Hyperedge<HE> {
    fn new(vertices: Vec<Uuid>, weight: HE) -> Self {
        Self { vertices, weight }
    }
}

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
                                .insert(*uuid, Hyperedge::new(vec![], weight.clone())),
                            EntityWeight::Vertex(weight) => {
                                state.vertices.insert(*uuid, Vertex::new(weight.clone()))
                            }
                        }

                        Ok(uuid.clone())
                    }
                    WriteOp::Delete(_, _) => todo!(),
                    WriteOp::UpdateRelation(uuid, relation) => {
                        match relation {
                            EntityRelation::Hyperedge(vertices) => {
                                // state.hyperedges.replace(uuid, vertices, false);
                            }
                            EntityRelation::Vertex(hyperedges) => {
                                // state.vertices.replace(uuid, hyperedges, false);
                            }
                        };

                        Ok(uuid.clone())
                    }
                    WriteOp::UpdateWeight(uuid, weight) => {
                        match weight {
                            EntityWeight::Hyperedge(weight) => {
                                if let Some(mut hyperedge) = state.hyperedges.get(uuid) {
                                    hyperedge.weight = weight.clone();
                                    state.hyperedges.replace(*uuid, hyperedge, false);
                                } else {
                                    // TODO: error not found ?
                                };
                            }
                            EntityWeight::Vertex(weight) => todo!(),
                        }
                        Ok(uuid.clone())
                    }
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
    writer: Option<ActorHandle<Arc<Path>, Arc<WriteOp<V, HE>>, ()>>,
}

impl<V, HE> IOManager<V, HE>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Serialize + Sync + 'static,
    HE: Clone + Debug + Send + Sync + 'static,
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
                            .unwrap();
                        let metadata = file.metadata().await.unwrap();

                        if metadata.len() != 0 {
                            let mut contents = vec![];
                            file.read_to_end(&mut contents).await.unwrap();

                            let data: HashMap<Uuid, Vertex<V>> = deserialize(&contents).unwrap();

                            entity = data.get(&uuid).cloned().map(Entity::Vertex);
                        }
                    }
                };

                Ok(entity)
            }
            .boxed()
        })
    }

    #[instrument]
    async fn get_writer(&self) -> ActorHandle<Arc<Path>, Arc<WriteOp<V, HE>>, ()> {
        ActorHandle::new(
            self.vertices_db_path.clone(),
            &|vertices_db_path, write_op| {
                async move {
                    match write_op.borrow() {
                        WriteOp::Create(uuid, entity_weight) => {
                            match entity_weight {
                                EntityWeight::Hyperedge(weight) => {}
                                EntityWeight::Vertex(weight) => {
                                    let mut file = OpenOptions::new()
                                        .read(true)
                                        .write(true)
                                        .open(vertices_db_path.clone())
                                        .await
                                        .unwrap();
                                    let metadata = file.metadata().await.unwrap();
                                    let mut data: HashMap<Uuid, Vertex<V>> = HashMap::default();

                                    if metadata.len() != 0 {
                                        let mut contents = vec![];
                                        file.read_to_end(&mut contents).await.unwrap();

                                        data = deserialize(&contents).unwrap();
                                    }

                                    data.insert(*uuid, Vertex::new(weight.clone()));

                                    let bytes = serialize(&data).unwrap();

                                    file.write_all(&bytes).await.unwrap();
                                    file.sync_data().await.unwrap();
                                }
                            };
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
    io_manager_writer: ActorHandle<Arc<Path>, Arc<WriteOp<V, HE>>, ()>,
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
        io_manager_writer: ActorHandle<Arc<Path>, Arc<WriteOp<V, HE>>, ()>,
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

                let mut entity = handles
                    .memory_cache_reader
                    .process(read_op)
                    .await
                    .map_err(|_| HypergraphError::EntityNotFound)?;

                // We use a read-through strategy here.
                if entity.is_none() {
                    entity = handles
                        .io_manager_reader
                        .process(read_op)
                        .await
                        .map_err(|_| HypergraphError::EntityNotFound)?;

                    // TODO: cache miss but on disk -> sync cache
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
                match write_op.borrow() {
                    WriteOp::Create(..) => {
                        debug!("Writing with entity manager.");

                        let uuid = handles
                            .memory_cache_writer
                            .process(write_op.clone())
                            .await
                            .map_err(|_| HypergraphError::EntityCreation)?;

                        // We don't wait for the IOManager to respond since we use a
                        // write-through strategy.
                        handles
                            .io_manager_writer
                            .process(write_op.clone())
                            .await
                            .map_err(|_| HypergraphError::EntityCreation)?;

                        Ok(uuid)
                    }
                    WriteOp::Delete(_, _) => {
                        todo!()
                    }
                    WriteOp::UpdateWeight(uuid, weight) => {
                        debug!("Updating with entity manager.");

                        let result = handles
                            .memory_cache_reader
                            .process(ReadOp(*uuid, weight.into()))
                            .await
                            .map_err(|_| HypergraphError::EntityUpdate)?;

                        // Here we have a cache hit.
                        // Update the cache, then the local data.
                        if result.is_some() {
                            handles
                                .memory_cache_writer
                                .process(Arc::new(WriteOp::UpdateWeight(*uuid, weight.clone())))
                                .await
                                .map_err(|_| HypergraphError::EntityUpdate)?;
                        } else {
                            return Err(HypergraphError::EntityNotFound);
                        }

                        Ok(*uuid)
                    }
                    WriteOp::UpdateRelation(uuid, entity) => {
                        todo!();
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
    HE: Clone + Debug + Send + Sync,
{
    entity_manager: EntityManager<V, HE>,
    io_manager: IOManager<V, HE>,
    memory_cache: MemoryCache<V, HE>,
}

impl<V, HE> Hypergraph<V, HE>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
    HE: Clone + Debug + Send + Sync + 'static,
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
            .await
            .map_err(|_| HypergraphError::EntityCreation)?;

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
    //         .await
    //         .map_err(|_| HypergraphError::VertexInsertion)?;
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
            .await
            .map_err(|_| HypergraphError::VertexRetrieval)?;

        if vertex.is_some() {
            debug!("Vertex {} found", uuid.to_string());

            match vertex.unwrap() {
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
