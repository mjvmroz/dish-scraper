mod dish;
mod err;

use std::fs::File;

use dish::edge_filter::{analyze, CongressionalGraph};
use dish::feed::rss_channel;
use dish::site::fetch_links;
use err::LazyResult;
use indicatif::ProgressBar;
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
    println!("Fetching RSS feed");
    let resp = rss_channel().await?;
    let episodes: Vec<Episode> = resp
        .items
        .iter()
        .flat_map(|item| Episode::try_from(item.to_owned()).ok())
        .collect();

    // TODO: Look into async_iter
    let mut links = Vec::new();
    println!("Fetching links for each episode...");
    let pb = ProgressBar::new(
        episodes
            .len()
            .try_into()
            .expect("Jen's gone insane. She can't be stopped."),
    );
    for episode in &episodes {
        let ep_links = fetch_links(episode.number).await?;
        links.push((episode.number, ep_links.clone()));
        pb.set_message(format!("Episode {}: {:?}", episode.number, ep_links));
        pb.inc(1);
    }
    pb.finish();

    persist(analyze(episodes, links)).await
}
