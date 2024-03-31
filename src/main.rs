mod dish;
mod err;

use std::fs::File;

use dish::edge_filter::{analyze, CongressionalGraph};
use dish::feed::rss_channel;
use err::LazyResult;
use tokio::fs;

use crate::dish::feed::Episode;

async fn persist(data: CongressionalGraph) -> LazyResult<()> {
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
    let episodes: Vec<Episode> = resp
        .items
        .iter()
        .flat_map(|item| Episode::try_from(item.to_owned()).ok())
        .collect();

    // TODO: Actually fetch the links
    let links: Vec<(usize, Vec<usize>)> = episodes
        .iter()
        .map(|ep| (ep.number, [ep.number].into()))
        .collect();

    persist(analyze(episodes, links)).await
}
