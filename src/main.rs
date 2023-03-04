mod dish;
mod err;

use std::collections::HashMap;

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
    let json = serde_json::to_string(&episodes)?;
    println!("{}", json);
    Ok(())
}
