use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashSet;

use crate::err::LazyResult;

fn pointers(html_content: Html) -> HashSet<usize> {
    let link_slug_re = Regex::new(r"(?i)congressionaldish.com/cd-?(?P<num>\d+)[^a-z0-9]")
        .expect("Whoops, illegal regex");

    html_content
        .select(&Selector::parse("a").unwrap())
        .filter(|a| {
            a.value()
                .attr("rel")
                .filter(|rel| (**rel).eq("prev"))
                .is_none()
        })
        .filter_map(|a| a.value().attr("href"))
        .filter_map(|href| {
            link_slug_re
                .captures(href)
                .map(|captures| captures["num"].parse().expect("Failed to parse number"))
        })
        .collect()
}

async fn fetch_content(url: &str) -> LazyResult<Html> {
    let content = reqwest::get(url).await?.text().await?;
    Ok(Html::parse_document(&content))
}

pub(crate) async fn fetch_links(slug: &str) -> LazyResult<HashSet<usize>> {
    let url = format!("https://congressionaldish.com/{}", slug);
    fetch_content(&url).await.map(pointers)
}
