#[doc(hidden)]
pub mod errors;

use std::{fmt::Debug, path::Path, sync::Arc};

use errors::HypergraphError;
use quick_cache::sync::Cache;
use tokio::{
    fs::{create_dir_all, read_dir, read_to_string},
    sync::{mpsc, oneshot, Mutex},
};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

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
    Hyperedge(HE),
    Vertex(V),
}

#[derive(Debug)]
struct MemoryCache<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    hyperedges: Arc<Cache<Uuid, HE>>,
    vertices: Arc<Cache<Uuid, V>>,
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
        hyperedges: Arc<Cache<Uuid, HE>>,
        vertices: Arc<Cache<Uuid, V>>,
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
        hyperedges: Arc<Cache<Uuid, HE>>,
        vertices: Arc<Cache<Uuid, V>>,
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
    writer: HypergraphEntitySender<V, HE>,
}

impl<V, HE> IOManager<V, HE>
where
    V: Clone + Debug + Send + Sync + 'static,
    HE: Clone + Debug + Send + Sync + 'static,
{
    #[instrument]
    async fn start() -> Result<Self, HypergraphError> {
        info!("Starting IOManager");

        let writer = Self::get_writer().await?;

        Ok(Self { writer })
    }

    #[instrument]
    async fn get_writer() -> Result<HypergraphEntitySender<V, HE>, HypergraphError> {
        let (sender, mut receiver) = mpsc::channel::<Entity<V, HE>>(1);

        tokio::spawn(async move {
            while let Some(entity) = receiver.recv().await {
                debug!("Writing to disk.");
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
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    entity_manager: EntityManager<V, HE>,
    io_manager: IOManager<V, HE>,
    memory_cache: MemoryCache<V, HE>,
}

impl<V, HE> Hypergraph<V, HE>
where
    V: Clone + Debug + Send + Sync + 'static,
    HE: Clone + Debug + Send + Sync + 'static,
{
    #[instrument]
    pub async fn init(path: impl AsRef<Path> + std::fmt::Debug) -> Result<Self, HypergraphError> {
        info!("Init Hypergraph");

        let io_manager = IOManager::start().await?;
        let memory_cache = MemoryCache::start().await?;

        Ok(Self {
            entity_manager: EntityManager::start(Handles::new(
                io_manager.writer.clone(),
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
            .send((Entity::Vertex(weight), sender))
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
        vertices: &[Uuid],
    ) -> Result<Uuid, HypergraphError> {
        let (sender, receiver) = oneshot::channel();

        self.entity_manager
            .writer
            .send((Entity::Hyperedge(weight), sender))
            .await
            .map_err(|_| HypergraphError::HyperedgeInsertion)?;

        let uuid = receiver
            .await
            .map_err(|_| HypergraphError::HyperedgeInsertion)?;

        debug!("Hyperedge {} added", uuid.to_string());

        Ok(uuid)
    }
}
