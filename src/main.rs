mod dish;
mod err;

use std::{collections::HashMap, fs::File};

use dish::feed::rss_channel;
use err::LazyResult;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::dish::edge_filter::adjacency_reduced_edges;
use crate::dish::feed::Episode;

#[derive(Debug, Serialize, Deserialize)]
struct EpisodeData {
    episodes: Vec<Episode>,
    adjacency_reduced_edges: HashMap<usize, usize>,
}

async fn persist_episodes(data: EpisodeData) -> LazyResult<()> {
    if !fs::metadata("output").await.is_ok() {
        fs::create_dir("output").await?;
    }

    let mut json_file = File::create("output/episodes.json")?;
    serde_json::to_writer(&mut json_file, &data)?;
    drop(json_file);

    let mut cbor_file = File::create("output/episodes.cbor")?;
    serde_cbor::ser::to_writer(&mut cbor_file, &data)?;
    drop(cbor_file);
    Ok(())
}

#[tokio::main]
async fn main() -> LazyResult<()> {
    let resp = rss_channel().await?;
    let mut episodes: Vec<Episode> = resp
        .items
        .iter()
        .flat_map(|item| Episode::try_from(item.to_owned()).ok())
        .collect();

    episodes.sort_by_key(|ep| ep.number);

    let adjacency_reduced_edges = adjacency_reduced_edges(&episodes);

    let data = EpisodeData {
        episodes,
        adjacency_reduced_edges,
    };

    persist_episodes(data).await
}
