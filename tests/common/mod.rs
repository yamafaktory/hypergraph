use std::io::Error;

use futures::future::{BoxFuture, FutureExt};
use hypergraph::{errors::HypergraphError, Hypergraph};
use serde::{Deserialize, Serialize};
use tokio::fs::remove_dir_all;

static PATH: &str = "./test";

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub(crate) struct Vertex {}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Hyperedge {}

type Handler<'a> = dyn Fn() -> BoxFuture<'a, Result<(), Error>> + Send + Sync;

pub(crate) async fn prepare<'a>()
-> Result<(Hypergraph<Vertex, Hyperedge>, &'a Handler<'a>), HypergraphError> {
    tracing_subscriber::fmt::fmt()
        .pretty()
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .init();

    let graph = Hypergraph::<Vertex, Hyperedge>::init(PATH).await?;

    let clear = &|| async { remove_dir_all(PATH).await }.boxed();

    Ok((graph, clear))
}
