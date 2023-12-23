#[doc(hidden)]
pub mod errors;
// #[doc(hidden)]
// pub mod hyperedges;
// mod indexes;
// #[doc(hidden)]
// pub mod iterator;
// mod shared;
#[doc(hidden)]
mod types;
// mod utils;
// #[doc(hidden)]
// pub mod vertices;

use std::{
    fmt::{Debug, Display, Formatter},
    hash::Hash,
    marker::PhantomData,
    ops::Deref,
    sync::Arc,
};

use quick_cache::sync::Cache;
use tokio::{
    fs::{create_dir_all, read_dir, read_to_string},
    sync::{mpsc, oneshot, Mutex},
};
use tracing::{debug, error, info, span, warn, Level};
//
// use bi_hash_map::BiHashMap;
use types::{AIndexMap, AIndexSet, ARandomState};
use uuid::Uuid;
//
// // Reexport indexes at this level.
// pub use crate::core::indexes::{HyperedgeIndex, VertexIndex};
//

// /// Shared Trait for the vertices.
// /// Must be implemented to use the library.
// pub trait VertexTrait: Clone + Debug + Display {}
//
// impl<T> VertexTrait for T where T: Clone + Debug + Display {}
//
// /// Shared Trait for the hyperedges.
// /// Must be implemented to use the library.
// pub trait HyperedgeTrait: VertexTrait + Into<usize> {}
//
// impl<T> HyperedgeTrait for T where T: VertexTrait + Into<usize> {}

//
// #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
// pub struct StorageContainer<A> {
//     data: PhantomData<A>,
//     index: Uuid,
// }
//
// impl<A> StorageContainer<A> {
//     pub fn new() -> Self {
//         StorageContainer {
//             data: PhantomData,
//             index: Uuid::new_v7(),
//         }
//     }
// }

enum Entity<V, HE>
where
    V: Copy + Clone + Debug + Send + Sync,
    HE: Copy + Clone + Debug + Send + Sync,
{
    Hyperedge(HE),
    Vertex(V),
}

#[derive(Debug)]
struct MemoryCache<V, HE>
where
    V: Copy + Clone + Debug + Send + Sync,
    HE: Copy + Clone + Debug + Send + Sync,
{
    hyperedges: Arc<Cache<Uuid, HE>>,
    vertices: Arc<Cache<Uuid, V>>,
    writer: mpsc::Sender<(Entity<V, HE>, oneshot::Sender<String>)>,
}

impl<V, HE> MemoryCache<V, HE>
where
    V: Copy + Clone + Debug + Send + Sync + 'static,
    HE: Copy + Clone + Debug + Send + Sync + 'static,
{
    async fn new() -> Result<Self, ()> {
        info!("Creating IOManager");

        let sender = Self::get_writer().await?;

        Ok(Self {
            hyperedges: Arc::new(Cache::new(10_000)),
            vertices: Arc::new(Cache::new(10_000)),
            writer: sender,
        })
    }

    #[tracing::instrument]
    async fn get_reader() -> Result<mpsc::Sender<(Entity<V, HE>, oneshot::Sender<String>)>, ()> {
        let (sender, mut rx) = mpsc::channel::<(Entity<V, HE>, oneshot::Sender<String>)>(1);

        tokio::spawn(async move {
            while let Some((todo, response)) = rx.recv().await {
                debug!("Reading from in-memory cache.");

                response.send(String::from("hello")).unwrap();
            }
        });

        Ok(sender)
    }

    #[tracing::instrument]
    async fn get_writer() -> Result<mpsc::Sender<(Entity<V, HE>, oneshot::Sender<String>)>, ()> {
        let (sender, mut rx) = mpsc::channel::<(Entity<V, HE>, oneshot::Sender<String>)>(1);

        tokio::spawn(async move {
            while let Some((todo, response)) = rx.recv().await {
                debug!("Writing to in-memory cache.");

                response.send(String::from("hello")).unwrap();
            }
        });

        Ok(sender)
    }
}

#[derive(Debug)]
struct IOManager {
    sender: mpsc::Sender<(String, oneshot::Sender<String>)>,
}

impl IOManager {
    #[tracing::instrument]
    async fn new() -> Result<Self, ()> {
        info!("Creating IOManager");

        let sender = Self::get_writer().await?;

        Ok(Self { sender })
    }

    #[tracing::instrument]
    async fn get_writer() -> Result<mpsc::Sender<(String, oneshot::Sender<String>)>, ()> {
        let (sender, mut rx) = mpsc::channel::<(String, oneshot::Sender<String>)>(1);

        tokio::spawn(async move {
            while let Some((todo, response)) = rx.recv().await {
                debug!("Writing to disk.");
                response.send(String::from("hello")).unwrap();
            }
        });

        Ok(sender)
    }
}

/// A directed hypergraph composed of generic vertices and hyperedges.
#[derive(Debug)]
pub struct Hypergraph<V, HE>
where
    V: Copy + Clone + Debug + Send + Sync,
    HE: Copy + Clone + Debug + Send + Sync,
{
    cache: MemoryCache<V, HE>,
    writer: IOManager,
}

impl<V, HE> Hypergraph<V, HE>
where
    V: Copy + Clone + Debug + Send + Sync + 'static,
    HE: Copy + Clone + Debug + Send + Sync + 'static,
{
    #[tracing::instrument]
    pub async fn init() -> Result<Self, ()> {
        info!("Init Hypergraph");

        Ok(Self {
            cache: MemoryCache::new().await?,
            writer: IOManager::new().await?,
        })
    }

    #[tracing::instrument]
    pub async fn add_vertex(self, vertex: V) -> Result<(), ()> {
        let (tx, rx) = oneshot::channel();

        self.cache
            .writer
            .send((Entity::Vertex(vertex), tx))
            .await
            .unwrap();

        let response = rx.await.unwrap();

        debug!(response);

        Ok(())
    }
}
