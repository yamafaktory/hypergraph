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
use futures::{future::BoxFuture, FutureExt};
use quick_cache::sync::Cache;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{create_dir_all, try_exists, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{mpsc, oneshot, Mutex},
};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::actors::ActorHandle;

static VERTICES_DB: &str = "vertices.db";
static HYPEREDGES_DB: &str = "hyperedges.db";

type EntityWithUuidSender<V, HE> = mpsc::Sender<(Entity<V, HE>, Uuid)>;
type EntityKindWithUuidSenderWithResponse<V, HE> =
    mpsc::Sender<((EntityKind, Uuid), oneshot::Sender<Option<Entity<V, HE>>>)>;
type EntitySenderWithResponse<V, HE, R> = mpsc::Sender<(Entity<V, HE>, oneshot::Sender<R>)>;
type UuidSender<V, HE> = mpsc::Sender<(Uuid, oneshot::Sender<Entity<V, HE>>)>;

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
struct MemoryCache<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    hyperedges: Arc<Cache<Uuid, Hyperedge<HE>>>,
    vertices: Arc<Cache<Uuid, Vertex<V>>>,
    reader: ActorHandle<(EntityKind, Uuid), Option<Entity<V, HE>>>,
    writer: EntitySenderWithResponse<V, HE, Uuid>,
}

impl<V, HE> MemoryCache<V, HE>
where
    V: Clone + Debug + Send + Sync + 'static,
    HE: Clone + Debug + Send + Sync + 'static,
{
    async fn start() -> Result<Self, HypergraphError> {
        info!("Starting MemoryCache");

        let hyperedges = Arc::new(Cache::new(10_000));
        let vertices = Arc::new(Cache::new(10_000));
        let reader = Self::get_reader(hyperedges.clone(), vertices.clone()).await;
        let writer = Self::get_writer(hyperedges.clone(), vertices.clone()).await?;

        Ok(Self {
            hyperedges,
            vertices,
            reader,
            writer,
        })
    }

    async fn test(
        self,
        entity_kind: EntityKind,
        uuid: Uuid,
    ) -> Result<Option<Entity<V, HE>>, HypergraphError> {
        Ok(None)
    }

    #[tracing::instrument]
    async fn get_reader(
        hyperedges: Arc<Cache<Uuid, Hyperedge<HE>>>,
        vertices: Arc<Cache<Uuid, Vertex<V>>>,
    ) -> ActorHandle<(EntityKind, Uuid), Option<Entity<V, HE>>> {
        ActorHandle::<(EntityKind, Uuid), Option<Entity<V, HE>>>::new(&|(entity_kind, uuid)| {
            async move {
                debug!("Reading from in-memory cache.");
                // let entity = match entity_kind {
                //     EntityKind::Hyperedge => hyperedges.get(&uuid).map(Entity::Hyperedge),
                //     EntityKind::Vertex => vertices.get(&uuid).map(Entity::Vertex),
                // };

                // Ok(entity)
                Ok(None)
            }
            .boxed()
        })

        // let t = a.process(12).await?;
        // debug!(t);

        // let (sender, mut receiver) =
        //     mpsc::channel::<((EntityKind, Uuid), oneshot::Sender<Option<Entity<V, HE>>>)>(1);
        //
        // tokio::spawn(async move {
        //     while let Some(((entity_kind, uuid), response)) = receiver.recv().await {
        //         debug!("Reading from in-memory cache.");
        //
        //         let entity = match entity_kind {
        //             EntityKind::Hyperedge => hyperedges.get(&uuid).map(Entity::Hyperedge),
        //             EntityKind::Vertex => vertices.get(&uuid).map(Entity::Vertex),
        //         };
        //
        //         response.send(entity).unwrap();
        //     }
        // });
        //
        // Ok(sender)
    }

    #[instrument]
    async fn get_writer(
        hyperedges: Arc<Cache<Uuid, Hyperedge<HE>>>,
        vertices: Arc<Cache<Uuid, Vertex<V>>>,
    ) -> Result<EntitySenderWithResponse<V, HE, Uuid>, HypergraphError> {
        let (sender, mut receiver) = mpsc::channel::<(Entity<V, HE>, oneshot::Sender<Uuid>)>(1);

        tokio::spawn(async move {
            while let Some((entity, response)) = receiver.recv().await {
                debug!("Writing to in-memory cache.");

                let uuid = Uuid::now_v7();

                match entity {
                    Entity::Hyperedge(hyperedge) => hyperedges.insert(uuid, hyperedge),
                    Entity::Vertex(vertex) => vertices.insert(uuid, vertex),
                }

                response.send(uuid).unwrap();
            }
        });

        Ok(sender)
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
    reader: Option<EntityKindWithUuidSenderWithResponse<V, HE>>,
    writer: Option<EntityWithUuidSender<V, HE>>,
}

impl<V, HE> IOManager<V, HE>
where
    V: Clone + Debug + for<'a> Deserialize<'a> + Send + Sync + Serialize + 'static,
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
        let reader = self.get_reader().await?;
        let writer = self.get_writer().await?;

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
    ) -> Result<EntityKindWithUuidSenderWithResponse<V, HE>, HypergraphError> {
        let (sender, mut receiver) =
            mpsc::channel::<((EntityKind, Uuid), oneshot::Sender<Option<Entity<V, HE>>>)>(1);
        let vertices_db_path = self.vertices_db_path.clone();

        tokio::spawn(async move {
            while let Some(((entity_kind, uuid), response)) = receiver.recv().await {
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

                response.send(entity).unwrap();
            }
        });

        Ok(sender)
    }

    #[instrument]
    async fn get_writer(&self) -> Result<EntityWithUuidSender<V, HE>, HypergraphError> {
        let (sender, mut receiver) = mpsc::channel::<(Entity<V, HE>, Uuid)>(1);
        let vertices_db_path = self.vertices_db_path.clone();

        tokio::spawn(async move {
            while let Some((entity, uuid)) = receiver.recv().await {
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
                }
            }
        });

        Ok(sender)
    }
}

#[derive(Clone, Debug)]
struct Handles<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    io_manager_reader: EntityKindWithUuidSenderWithResponse<V, HE>,
    io_manager_writer: EntityWithUuidSender<V, HE>,
    memory_cache_reader: ActorHandle<(EntityKind, Uuid), Option<Entity<V, HE>>>,
    memory_cache_writer: EntitySenderWithResponse<V, HE, Uuid>,
}

impl<V, HE> Handles<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    fn new(
        io_manager_reader: EntityKindWithUuidSenderWithResponse<V, HE>,
        io_manager_writer: EntityWithUuidSender<V, HE>,
        memory_cache_reader: ActorHandle<(EntityKind, Uuid), Option<Entity<V, HE>>>,
        memory_cache_writer: EntitySenderWithResponse<V, HE, Uuid>,
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
    reader: EntityKindWithUuidSenderWithResponse<V, HE>,
    writer: EntitySenderWithResponse<V, HE, Uuid>,
}

impl<V, HE> EntityManager<V, HE>
where
    V: Clone + Debug + Send + Sync + 'static,
    HE: Clone + Debug + Send + Sync + 'static,
{
    async fn start(handles: Handles<V, HE>) -> Result<Self, HypergraphError> {
        info!("Starting EntityManager");

        let reader = Self::get_reader(handles.clone()).await?;
        let writer = Self::get_writer(handles.clone()).await?;

        Ok(Self { reader, writer })
    }

    #[instrument]
    async fn get_reader(
        handles: Handles<V, HE>,
    ) -> Result<EntityKindWithUuidSenderWithResponse<V, HE>, HypergraphError> {
        let (sender, mut receiver) =
            mpsc::channel::<((EntityKind, Uuid), oneshot::Sender<Option<Entity<V, HE>>>)>(1);

        tokio::spawn(async move {
            while let Some(((entity_kind, uuid), response)) = receiver.recv().await {
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
                    let (sender, receiver) = oneshot::channel();

                    handles
                        .io_manager_reader
                        .send(((entity_kind_copy, uuid), sender))
                        .await
                        .map_err(|_| HypergraphError::VertexInsertion)
                        .unwrap();

                    entity = receiver
                        .await
                        .map_err(|_| HypergraphError::VertexInsertion)
                        .unwrap();

                    // TODO: cache miss but on disk -> sync cache
                }

                response.send(entity).unwrap();
            }
        });

        Ok(sender)
    }

    #[instrument]
    async fn get_writer(
        handles: Handles<V, HE>,
    ) -> Result<EntitySenderWithResponse<V, HE, Uuid>, HypergraphError> {
        let (sender, mut receiver) = mpsc::channel::<(Entity<V, HE>, oneshot::Sender<Uuid>)>(1);

        tokio::spawn(async move {
            while let Some((entity, response)) = receiver.recv().await {
                debug!("Writing with entity manager.");

                let entity_copy = entity.clone();
                let (sender, receiver) = oneshot::channel();

                handles
                    .memory_cache_writer
                    .send((entity, sender))
                    .await
                    .map_err(|_| HypergraphError::VertexInsertion)
                    .unwrap();

                let uuid = receiver
                    .await
                    .map_err(|_| HypergraphError::VertexInsertion)
                    .unwrap();

                // We don't wait for the IOManager to respond since we use a
                // write-through strategy.
                handles
                    .io_manager_writer
                    .send((entity_copy, uuid))
                    .await
                    .map_err(|_| HypergraphError::VertexInsertion)
                    .unwrap();

                response.send(uuid).unwrap();
            }
        });

        Ok(sender)
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
    #[instrument]
    pub async fn init<P>(path: P) -> Result<Self, HypergraphError>
    where
        P: AsRef<Path> + Copy + Debug,
    {
        info!("Init Hypergraph");

        let mut io_manager = IOManager::new(path).await?;
        let memory_cache = MemoryCache::start().await?;

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
        let (sender, receiver) = oneshot::channel();

        self.entity_manager
            .writer
            .send((
                Entity::Vertex(Vertex::new(HashSet::default(), weight)),
                sender,
            ))
            .await
            .map_err(|_| HypergraphError::VertexInsertion)?;

        let uuid = receiver
            .await
            .map_err(|_| HypergraphError::VertexInsertion)?;

        debug!("Vertex {} added", uuid.to_string());

        Ok(uuid)
    }

    #[instrument]
    pub async fn get_vertex(&self, uuid: Uuid) -> Result<Option<Vertex<V>>, HypergraphError> {
        let (sender, receiver) = oneshot::channel();

        self.entity_manager
            .reader
            .send(((EntityKind::Vertex, uuid), sender))
            .await
            .map_err(|_| HypergraphError::VertexRetrieval)?;

        let vertex = receiver
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
        let (sender, receiver) = oneshot::channel();

        self.entity_manager
            .writer
            .send((Entity::Hyperedge(Hyperedge::new(vertices, weight)), sender))
            .await
            .map_err(|_| HypergraphError::HyperedgeInsertion)?;

        let uuid = receiver
            .await
            .map_err(|_| HypergraphError::HyperedgeInsertion)?;

        debug!("Hyperedge {} added", uuid.to_string());

        Ok(uuid)
    }
}
