use std::{ops::Range, sync::Arc};

use full_moon::{
    node::{Node, Tokens},
    tokenizer::{self, TokenReference},
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
    start: Position,
    end: Position,
}

impl Location {
    pub(crate) fn new(src: Arc<SourceInfo>, start: Position, end: Position) -> Self {
        Self { src, start, end }
    }

    pub(crate) fn from_position(src: Arc<SourceInfo>, pos: Position) -> Self {
        Self {
            src,
            start: pos,
            end: pos,
        }
    }

    pub fn dummy() -> Self {
        Location {
            src: Arc::new(SourceInfo::new("<null>".to_string(), "".to_string())),
            start: Position::default(),
            end: Position::default(),
        }
    }

    pub fn start(&self) -> Position {
        self.start
    }

    pub fn end(&self) -> Position {
        self.end
    }

    pub fn src(&self) -> &Arc<SourceInfo> {
        &self.src
    }

    pub fn range(&self) -> Range<usize> {
        self.start.bytes..self.end.bytes
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
        let lo = Position::new(range.0.bytes(), range.0.line(), range.0.character());
        let hi = Position::new(range.1.bytes(), range.1.line(), range.1.character());
        Location::new(src.clone(), lo, hi)
    }
}

fn tokens_to_location(src: Arc<SourceInfo>, mut tokens: Tokens) -> Option<Location> {
    let Some(Some(first)) = tokens.next().map(|t| t.range()) else {
        return None;
    };
    let first = Location::new(
        Arc::clone(&src),
        Position::from_fullmoon_position(first.0),
        Position::from_fullmoon_position(first.1),
    );
    let Some(Some(last)) = tokens.last().map(|t| t.range()) else {
        return Some(first);
    };
    let last = Location::new(
        Arc::clone(&src),
        Position::from_fullmoon_position(last.0),
        Position::from_fullmoon_position(last.1),
    );
    Some(Location::new(src, first.start, last.end))
}

impl std::ops::Add for Location {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        assert_eq!(self.src.path(), other.src.path());
        Self::new(self.src.clone(), self.start, other.end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    bytes: usize,
    line: usize,
    character: usize,
}

impl Position {
    pub fn new(bytes: usize, line: usize, character: usize) -> Self {
        Self {
            bytes,
            line,
            character,
        }
    }

    pub fn from_fullmoon_position(pos: tokenizer::Position) -> Self {
        Self {
            bytes: pos.bytes(),
            line: pos.line(),
            character: pos.character(),
        }
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn character(&self) -> usize {
        self.character
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            bytes: 0,
            line: 1,
            character: 0,
        }
    }
}
