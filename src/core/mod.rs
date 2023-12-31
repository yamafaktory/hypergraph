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
use quick_cache::sync::Cache;
use serde::Serialize;
use tokio::{
    fs::{create_dir_all, try_exists, File},
    io::AsyncReadExt,
    sync::{mpsc, oneshot},
};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

static VERTICES_DB: &str = "vertices.db";
static HYPEREDGES_DB: &str = "hyperedges.db";

type HypergraphEntitySender<V, HE> = mpsc::Sender<Entity<V, HE>>;
type HypergraphEntitySenderWithResponse<V, HE, R> =
    mpsc::Sender<(Entity<V, HE>, oneshot::Sender<R>)>;
type HypergraphUuidSender<V, HE> = mpsc::Sender<(Uuid, oneshot::Sender<Entity<V, HE>>)>;

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

#[derive(Clone, Debug, Serialize)]
struct Vertex<V> {
    relations: HashSet<Uuid>,
    weight: V,
}

impl<V> Vertex<V> {
    fn new(relations: HashSet<Uuid>, weight: V) -> Self {
        Self { relations, weight }
    }
}

#[derive(Clone, Debug)]
struct Hyperedge<HE> {
    relations: Vec<Uuid>,
    weight: HE,
}

impl<HE> Hyperedge<HE> {
    fn new(relations: Vec<Uuid>, weight: HE) -> Self {
        Self { relations, weight }
    }
}

struct Vertices<V>(HashMap<Uuid, Vertex<V>>);

struct Hyperedges<HE>(HashMap<Uuid, Hyperedge<HE>>);

#[derive(Debug)]
struct MemoryCache<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    hyperedges: Arc<Cache<Uuid, Hyperedge<HE>>>,
    vertices: Arc<Cache<Uuid, Vertex<V>>>,
    reader: HypergraphUuidSender<V, HE>,
    writer: HypergraphEntitySenderWithResponse<V, HE, Uuid>,
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
        let reader = Self::get_reader(hyperedges.clone(), vertices.clone()).await?;
        let writer = Self::get_writer(hyperedges.clone(), vertices.clone()).await?;

        Ok(Self {
            hyperedges,
            vertices,
            reader,
            writer,
        })
    }

    #[tracing::instrument]
    async fn get_reader(
        hyperedges: Arc<Cache<Uuid, Hyperedge<HE>>>,
        vertices: Arc<Cache<Uuid, Vertex<V>>>,
    ) -> Result<HypergraphUuidSender<V, HE>, HypergraphError> {
        let (sender, mut receiver) = mpsc::channel::<(Uuid, oneshot::Sender<Entity<V, HE>>)>(1);

        tokio::spawn(async move {
            while let Some((todo, response)) = receiver.recv().await {
                debug!("Reading from in-memory cache.");

                // response.send(Entity::Vertex(123)).unwrap();
            }
        });

        Ok(sender)
    }

    #[instrument]
    async fn get_writer(
        hyperedges: Arc<Cache<Uuid, Hyperedge<HE>>>,
        vertices: Arc<Cache<Uuid, Vertex<V>>>,
    ) -> Result<HypergraphEntitySenderWithResponse<V, HE, Uuid>, HypergraphError> {
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
    writer: Option<HypergraphEntitySender<V, HE>>,
}

impl<V, HE> IOManager<V, HE>
where
    V: Clone + Debug + Send + Sync + Serialize + 'static,
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
            writer: None,
        })
    }

    async fn start(&mut self) -> Result<(), HypergraphError> {
        let writer = self.get_writer().await?;

        self.writer = Some(writer);

        let path = self.path.clone();

        match try_exists(path.clone()).await {
            Ok(true) => {
                debug!("Path already exists");

                if let Ok(exists) = try_exists(self.vertices_db_path.clone()).await {
                    if !exists {
                        self.create_entity_db(self.vertices_db_path.clone()).await?;
                    }
                } else {
                    return Err(HypergraphError::DatabasesCreation);
                }

                if let Ok(exists) = try_exists(self.hyperedges_db_path.clone()).await {
                    if !exists {
                        self.create_entity_db(self.hyperedges_db_path.clone())
                            .await?;
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
            .sync_all()
            .await
            .map_err(|_| HypergraphError::DatabasesCreation)?;

        Ok(())
    }

    #[instrument]
    async fn get_writer(&self) -> Result<HypergraphEntitySender<V, HE>, HypergraphError> {
        let (sender, mut receiver) = mpsc::channel::<Entity<V, HE>>(1);
        let vertices_db_path = self.vertices_db_path.clone();

        tokio::spawn(async move {
            while let Some(entity) = receiver.recv().await {
                debug!("Writing to disk.");

                match entity {
                    Entity::Hyperedge(hyperedge) => {}
                    Entity::Vertex(vertex) => {
                        let mut file = File::open(vertices_db_path.clone()).await.unwrap();
                        let metadata = file.metadata().await.unwrap();

                        if metadata.len() == 0 {
                            let mut data: HashMap<Uuid, Vertex<V>> = HashMap::default();
                            data.insert(Uuid::now_v7(), vertex);
                            let t = serialize(&data).unwrap();
                            dbg!(t);
                        } else {
                            let mut contents = vec![];
                            file.read_to_end(&mut contents).await.unwrap();
                            let decompressed: Vec<u8> = deserialize(&contents).unwrap();
                        }
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
    io_manager_writer: HypergraphEntitySender<V, HE>,
    memory_cache_writer: HypergraphEntitySenderWithResponse<V, HE, Uuid>,
}

impl<V, HE> Handles<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    fn new(
        io_manager_writer: HypergraphEntitySender<V, HE>,
        memory_cache_writer: HypergraphEntitySenderWithResponse<V, HE, Uuid>,
    ) -> Self {
        Self {
            io_manager_writer,
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
    writer: mpsc::Sender<(Entity<V, HE>, oneshot::Sender<Uuid>)>,
}

impl<V, HE> EntityManager<V, HE>
where
    V: Clone + Debug + Send + Sync + 'static,
    HE: Clone + Debug + Send + Sync + 'static,
{
    async fn start(handles: Handles<V, HE>) -> Result<Self, HypergraphError> {
        info!("Starting EntityManager");

        let writer = Self::get_writer(handles.clone()).await?;

        Ok(Self { writer })
    }

    #[instrument]
    async fn get_writer(
        handles: Handles<V, HE>,
    ) -> Result<HypergraphEntitySenderWithResponse<V, HE, Uuid>, HypergraphError> {
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

                handles
                    .io_manager_writer
                    .send(entity_copy)
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
    V: Clone + Debug + Send + Serialize + Sync,
    HE: Clone + Debug + Send + Sync,
{
    entity_manager: EntityManager<V, HE>,
    io_manager: IOManager<V, HE>,
    memory_cache: MemoryCache<V, HE>,
}

impl<V, HE> Hypergraph<V, HE>
where
    V: Clone + Debug + Send + Sync + Serialize + 'static,
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
                io_manager.writer.clone().unwrap(),
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
