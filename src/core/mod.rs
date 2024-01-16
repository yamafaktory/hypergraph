#[doc(hidden)]
pub mod actors;
#[doc(hidden)]
pub mod errors;

use std::{
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

#[derive(Clone, Debug)]
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

pub(crate) type HashSet<K> = DefaultHashSet<K, RandomState>;
pub(crate) type HashMap<K, V> = DefaultHashMap<K, V, RandomState>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Vertex<V> {
    relations: HashSet<Uuid>,
    weight: V,
}

impl<V> Vertex<V> {
    fn new(relations: HashSet<Uuid>, weight: V) -> Self {
        Self { relations, weight }
    }
}

#[derive(Clone, Debug)]
pub struct Hyperedge<HE> {
    relations: Vec<Uuid>,
    weight: HE,
}

impl<HE> Hyperedge<HE> {
    fn new(relations: Vec<Uuid>, weight: HE) -> Self {
        Self { relations, weight }
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
    reader: ActorHandle<Arc<MemoryCacheState<V, HE>>, (EntityKind, Uuid), Option<Entity<V, HE>>>,
    writer: ActorHandle<Arc<MemoryCacheState<V, HE>>, Entity<V, HE>, Uuid>,
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
    ) -> ActorHandle<Arc<MemoryCacheState<V, HE>>, (EntityKind, Uuid), Option<Entity<V, HE>>> {
        ActorHandle::new(state, &|state, (entity_kind, uuid)| {
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
    ) -> ActorHandle<Arc<MemoryCacheState<V, HE>>, Entity<V, HE>, Uuid> {
        ActorHandle::new(state, &|state, entity| {
            async move {
                debug!("Writing to in-memory cache.");

                let uuid = Uuid::now_v7();

                match entity {
                    Entity::Hyperedge(hyperedge) => state.hyperedges.insert(uuid, hyperedge),
                    Entity::Vertex(vertex) => state.vertices.insert(uuid, vertex),
                }

                Ok(uuid)
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
    reader: Option<ActorHandle<Arc<Path>, (EntityKind, Uuid), Option<Entity<V, HE>>>>,
    writer: Option<ActorHandle<Arc<Path>, (Entity<V, HE>, Uuid), ()>>,
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
    async fn get_reader(
        &self,
    ) -> ActorHandle<Arc<Path>, (EntityKind, Uuid), Option<Entity<V, HE>>> {
        ActorHandle::new(
            self.vertices_db_path.clone(),
            &|vertices_db_path, (entity_kind, uuid)| {
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

                                let data: HashMap<Uuid, Vertex<V>> =
                                    deserialize(&contents).unwrap();

                                entity = data.get(&uuid).cloned().map(Entity::Vertex);
                            }
                        }
                    };

                    Ok(entity)
                }
                .boxed()
            },
        )
    }

    #[instrument]
    async fn get_writer(&self) -> ActorHandle<Arc<Path>, (Entity<V, HE>, Uuid), ()> {
        ActorHandle::new(
            self.vertices_db_path.clone(),
            &|vertices_db_path, (entity, uuid)| {
                async move {
                    debug!("Writing to disk.");

                    match entity {
                        Entity::Hyperedge(hyperedge) => {}
                        Entity::Vertex(vertex) => {
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

                            data.insert(uuid, vertex);

                            let bytes = serialize(&data).unwrap();

                            file.write_all(&bytes).await.unwrap();
                            file.sync_data().await.unwrap();
                        }
                    };

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
    io_manager_reader: ActorHandle<Arc<Path>, (EntityKind, Uuid), Option<Entity<V, HE>>>,
    io_manager_writer: ActorHandle<Arc<Path>, (Entity<V, HE>, Uuid), ()>,
    memory_cache_reader:
        ActorHandle<Arc<MemoryCacheState<V, HE>>, (EntityKind, Uuid), Option<Entity<V, HE>>>,
    memory_cache_writer: ActorHandle<Arc<MemoryCacheState<V, HE>>, Entity<V, HE>, Uuid>,
}

impl<V, HE> Handles<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    fn new(
        io_manager_reader: ActorHandle<Arc<Path>, (EntityKind, Uuid), Option<Entity<V, HE>>>,
        io_manager_writer: ActorHandle<Arc<Path>, (Entity<V, HE>, Uuid), ()>,
        memory_cache_reader: ActorHandle<
            Arc<MemoryCacheState<V, HE>>,
            (EntityKind, Uuid),
            Option<Entity<V, HE>>,
        >,
        memory_cache_writer: ActorHandle<Arc<MemoryCacheState<V, HE>>, Entity<V, HE>, Uuid>,
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
    reader: ActorHandle<Handles<V, HE>, (EntityKind, Uuid), Option<Entity<V, HE>>>,
    writer: ActorHandle<Handles<V, HE>, Entity<V, HE>, Uuid>,
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
    ) -> ActorHandle<Handles<V, HE>, (EntityKind, Uuid), Option<Entity<V, HE>>> {
        ActorHandle::new(handles, &|handles, (entity_kind, uuid)| {
            async move {
                debug!("Reading with entity manager.");

                let entity_kind_copy = entity_kind.clone();

                let mut entity = handles
                    .memory_cache_reader
                    .process((entity_kind, uuid))
                    .await
                    .map_err(|_| HypergraphError::VertexInsertion)
                    .unwrap();

                // We use a read-through strategy here.
                if entity.is_none() {
                    entity = handles
                        .io_manager_reader
                        .process((entity_kind_copy, uuid))
                        .await
                        .map_err(|_| HypergraphError::VertexInsertion)
                        .unwrap();

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
    ) -> ActorHandle<Handles<V, HE>, Entity<V, HE>, Uuid> {
        ActorHandle::new(handles, &|handles, entity| {
            async move {
                debug!("Writing with entity manager.");

                let entity_copy = entity.clone();

                let uuid = handles
                    .memory_cache_writer
                    .process(entity)
                    .await
                    .map_err(|_| HypergraphError::VertexInsertion)
                    .unwrap();

                // We don't wait for the IOManager to respond since we use a
                // write-through strategy.
                handles
                    .io_manager_writer
                    .process((entity_copy, uuid))
                    .await
                    .map_err(|_| HypergraphError::VertexInsertion)
                    .unwrap();

                Ok(uuid)
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
    pub async fn add_vertex(&self, weight: V) -> Result<Uuid, HypergraphError> {
        let uuid = self
            .entity_manager
            .writer
            .process(Entity::Vertex(Vertex::new(HashSet::default(), weight)))
            .await
            .map_err(|_| HypergraphError::VertexInsertion)?;

        debug!("Vertex {} added", uuid.to_string());

        Ok(uuid)
    }

    #[instrument]
    pub async fn get_vertex(&self, uuid: Uuid) -> Result<Option<Vertex<V>>, HypergraphError> {
        let vertex = self
            .entity_manager
            .reader
            .process((EntityKind::Vertex, uuid))
            .await
            .map_err(|_| HypergraphError::VertexRetrieval)?;

        if vertex.is_some() {
            debug!("Vertex {} found", uuid.to_string());

            match vertex.unwrap() {
                Entity::Hyperedge(_) => unreachable!(),
                Entity::Vertex(vertex) => Ok(Some(vertex)),
            }
        } else {
            debug!("Vertex {} not found", uuid.to_string());

            Ok(None)
        }
    }

    #[instrument]
    pub async fn add_hyperedge(
        &self,
        weight: HE,
        vertices: Vec<Uuid>,
    ) -> Result<Uuid, HypergraphError> {
        let uuid = self
            .entity_manager
            .writer
            .process(Entity::Hyperedge(Hyperedge::new(vertices, weight)))
            .await
            .map_err(|_| HypergraphError::HyperedgeInsertion)?;

        debug!("Hyperedge {} added", uuid.to_string());

        Ok(uuid)
    }
}
