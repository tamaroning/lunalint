use std::{ops::Range, sync::Arc};

use full_moon::{
    node::{Node, Tokens},
    tokenizer::TokenReference,
};

#[derive(Debug)]
pub struct SourceInfo {
    path: String,
    content: String,
}

impl SourceInfo {
    pub fn new(path: String, content: String) -> Self {
        Self { path, content }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    src: Arc<SourceInfo>,
    start: usize,
    end: usize,
}

impl Location {
    pub fn new(src: Arc<SourceInfo>, start: usize, end: usize) -> Self {
        Self { src, start, end }
    }

    pub fn dummy() -> Self {
        Location {
            src: Arc::new(SourceInfo::new("<null>".to_string(), "".to_string())),
            start: 0,
            end: 0,
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn src(&self) -> &Arc<SourceInfo> {
        &self.src
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

impl From<(&Arc<SourceInfo>, Tokens<'_>)> for Location {
    fn from((src, tokens): (&Arc<SourceInfo>, Tokens<'_>)) -> Self {
        tokens_to_location(src.clone(), tokens).unwrap()
    }
}

impl From<(&Arc<SourceInfo>, &TokenReference)> for Location {
    fn from((src, token): (&Arc<SourceInfo>, &TokenReference)) -> Self {
        let Some(range) = token.range() else {
            return Location::dummy();
        };
        Location::new(src.clone(), range.0.bytes(), range.1.bytes())
    }
}

fn tokens_to_location(src: Arc<SourceInfo>, mut tokens: Tokens) -> Option<Location> {
    let Some(Some(first)) = tokens.next().map(|t| t.range()) else {
        return None;
    };
    let first = Location::new(Arc::clone(&src), first.0.bytes(), first.1.bytes());
    let Some(Some(last)) = tokens.last().map(|t| t.range()) else {
        return Some(first);
    };
    let last = Location::new(Arc::clone(&src), last.0.bytes(), last.1.bytes());
    Some(Location::new(src, first.start, last.end))
}

impl std::ops::Add for Location {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        assert_eq!(self.src.path(), other.src.path());
        Self::new(self.src.clone(), self.start, other.end)
    }
}
