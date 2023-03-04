use std::collections::HashSet;

use regex::Regex;
use rss::Channel;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::err::LazyResult;

use super::ScraperError;

pub(crate) async fn rss_channel() -> LazyResult<Channel> {
    let content = reqwest::get("https://congressionaldish.libsyn.com/rss")
        .await?
        .bytes()
        .await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

fn format_slug(slug: &str) -> String {
    slug.replace("-", "").to_uppercase()
}

fn pointers(description: Html) -> HashSet<String> {
    let link_slug_re = Regex::new(r"(?i)congressionaldish.com/(?P<slug>cd-?\d+)[^a-z0-9]")
        .expect("Whoops, illegal regex");
    description
        .select(&Selector::parse("a").unwrap())
        .filter_map(|a| a.value().attr("href"))
        .filter_map(|href| {
            link_slug_re
                .captures(href)
                .map(|captures| format_slug(&captures["slug"]))
        })
        .collect::<HashSet<String>>()
}

impl TryFrom<rss::Item> for Episode {
    type Error = ScraperError;

    fn try_from(item: rss::Item) -> Result<Self, Self::Error> {
        let link_text_re =
            Regex::new(r"(?i)^(?P<slug>cd-?\d+):? (?P<title>.*)$").expect("Whoops, illegal regex");
        let title = item.title.ok_or(ScraperError::MissingTitle)?;
        let captures = link_text_re
            .captures(&title)
            .ok_or(ScraperError::TitleStructure(title.to_owned()))?;

        let slug = format_slug(&captures["slug"]);
        let title = captures["title"].to_string();
        let pointers = item
            .description
            .as_ref()
            .map(|html| pointers(Html::parse_fragment(html)))
            .unwrap_or_default();

        Ok(Self {
            slug,
            title,
            pointers,
            description: item.description,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Episode {
    pub slug: String,
    pub title: String,
    pub pointers: HashSet<String>,
    pub description: Option<String>,
}

impl Episode {
    pub(crate) fn page_url(&self) -> String {
        format!("https://congressionaldish.com/{}", self.slug)
    }
}
