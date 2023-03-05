pub(crate) mod feed;

use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub(crate) enum ScraperError {
    MissingTitle,
    MissingPublishDate,
    TitleStructure(String),
}

impl Display for ScraperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScraperError::MissingTitle => write!(f, "Missing title"),
            ScraperError::MissingPublishDate => write!(f, "Missing date"),
            ScraperError::TitleStructure(t) => write!(f, "Unexpected title structure: '{}'", t),
        }
    }
}

impl Error for ScraperError {}
