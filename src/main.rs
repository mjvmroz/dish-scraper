mod dish;
mod err;

use std::{collections::HashMap, fs::File};

use dish::feed::rss_channel;
use err::LazyResult;

use crate::dish::feed::Episode;

#[tokio::main]
async fn main() -> LazyResult<()> {
    let resp = rss_channel().await?;
    let episodes: HashMap<String, Episode> = resp
        .items
        .iter()
        .flat_map(|item| Episode::try_from(item.to_owned()).ok())
        .map(|e| (e.slug.clone(), e))
        .collect();

    let mut json_file = File::create("output/episodes.json")?;
    serde_json::to_writer(&mut json_file, &episodes)?;
    drop(json_file);

    let mut cbor_file = File::create("output/episodes.cbor")?;
    serde_cbor::ser::to_writer(&mut cbor_file, &episodes)?;
    drop(cbor_file);

    Ok(())
}
