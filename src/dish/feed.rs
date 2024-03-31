use std::collections::HashSet;

use chrono::serde::ts_milliseconds;

use chrono::{DateTime, Utc};
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

fn slug_from_num_str(num_str: &str) -> String {
    format!("CD-{}", num_str)
}

fn pointers(description: Html) -> HashSet<usize> {
    let link_slug_re = Regex::new(r"(?i)congressionaldish.com/cd-?(?P<num>\d+)[^a-z0-9]")
        .expect("Whoops, illegal regex");

    description
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

impl TryFrom<rss::Item> for Episode {
    type Error = ScraperError;

    fn try_from(item: rss::Item) -> Result<Self, Self::Error> {
        let link_text_re = Regex::new(r"(?i)^cd-?(?P<num_str>\d+):? (?P<title>.*)$")
            .expect("Whoops, illegal regex");
        let title = item.title.ok_or(ScraperError::MissingTitle)?;
        let captures = link_text_re
            .captures(&title)
            .ok_or(ScraperError::TitleStructure(title.to_owned()))?;

        let num_str = &captures["num_str"];
        let slug = slug_from_num_str(num_str);
        let number = num_str.parse().expect("Failed to parse number");
        let title = captures["title"].to_string();
        let published_at: DateTime<Utc> = item
            .pub_date
            .ok_or(ScraperError::MissingPublishDate)
            .map(|date| {
                DateTime::parse_from_rfc2822(&date)
                    .unwrap()
                    .with_timezone(&Utc)
            })
            .map_err(|_| ScraperError::MissingPublishDate)?;

        let preview = item.itunes_ext.and_then(|ext| ext.subtitle);

        Ok(Self {
            slug,
            number,
            title,
            published_at,
            preview,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Episode {
    pub slug: String,
    pub number: usize,
    #[serde(with = "ts_milliseconds")]
    pub published_at: DateTime<Utc>,
    pub title: String,
    pub preview: Option<String>,
}
