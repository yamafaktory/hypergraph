use std::{
    io::Error,
    path::{Path, PathBuf},
    time::Duration,
};

use futures::future::{BoxFuture, FutureExt};
use hypergraph::{errors::HypergraphError, Hypergraph};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{read_dir, remove_dir_all},
    time::{sleep, timeout},
};

const PATH: &str = "./test";
const SLEEP_INTERVAL: u64 = 100;
const TIMEOUT_IN_MS: u64 = 2_000;

#[derive(Clone, Copy, Debug, Eq, Deserialize, PartialEq, Serialize)]
pub(crate) struct Vertex {}

#[derive(Clone, Copy, Debug, Eq, Deserialize, PartialEq, Serialize)]
pub(crate) struct Hyperedge {}

type Handler<'a, A, R, E> = dyn Fn(A) -> BoxFuture<'a, Result<R, E>> + Send + Sync;

pub(crate) fn get_tracing_subscriber() {
    tracing_subscriber::fmt::fmt()
        .pretty()
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .init();
}

async fn wait<P>(path: P, expected: usize) -> Result<impl Iterator<Item = PathBuf>, HypergraphError>
where
    P: AsRef<Path>,
{
    let mut files = vec![];
    let path_buf = path.as_ref().to_path_buf();

    while files.len() != expected {
        files.clear();

        let mut dir = read_dir(path_buf.clone())
            .await
            .map_err(|_| HypergraphError::Processing)?;

        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|_| HypergraphError::Processing)?
        {
            if entry
                .metadata()
                .await
                .map_err(|_| HypergraphError::Processing)?
                .is_file()
            {
                files.push(entry.path());
            }
        }

        sleep(Duration::from_millis(SLEEP_INTERVAL)).await;
    }

    Ok(files.into_iter())
}

async fn wait_for_files<P>(
    path: P,
    expected: usize,
) -> Result<impl Iterator<Item = PathBuf>, HypergraphError>
where
    P: AsRef<Path>,
{
    timeout(Duration::from_millis(TIMEOUT_IN_MS), wait(path, expected))
        .await
        .map_err(|_| HypergraphError::Processing)?
}

pub(crate) async fn prepare<'a>(
    bypass_memory_cache: bool,
) -> Result<
    (
        Hypergraph<Vertex, Hyperedge>,
        &'a Handler<'a, (), (), Error>,
        &'a Handler<'a, usize, impl Iterator<Item = PathBuf>, HypergraphError>,
    ),
    HypergraphError,
> {
    let graph = if bypass_memory_cache {
        Hypergraph::<Vertex, Hyperedge>::init_with_config(PATH, 0, 0).await?
    } else {
        Hypergraph::<Vertex, Hyperedge>::init(PATH).await?
    };

    let clear = &|_| async { remove_dir_all(PATH).await }.boxed();

    let wait = &|expected: usize| async move { wait_for_files(PATH, expected).await }.boxed();

    Ok((graph, clear, wait))
}
