use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashSet;

use crate::err::LazyResult;

fn scrape_links(html_content: Html) -> HashSet<usize> {
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
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36")
        .build()?;

    let response = client.get(url).send().await?;
    let content = response.text().await?;
    Ok(Html::parse_document(&content))
}

pub(crate) async fn fetch_links(number: usize) -> LazyResult<HashSet<usize>> {
    let url = format!("https://congressionaldish.com/CD{}", number);
    let scraped_links = fetch_content(&url).await.map(scrape_links)?;
    let backward_links = scraped_links
        .into_iter()
        .filter(|&target| target < number)
        .collect();
    Ok(backward_links)
}
