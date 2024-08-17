use std::ops::Range;

use full_moon::{
    ast::{self, Expression},
    node::{Node, Tokens},
    tokenizer::{Symbol, TokenType},
};

pub struct Location {
    start: usize,
    end: usize,
}

impl Location {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn dummy() -> Self {
        Self::new(0, 0)
    }

    pub fn start(&self) -> usize {
        self.start
    }
    /*
    pub fn end(&self) -> usize {
        self.end
    }
    */
}

impl From<Tokens<'_>> for Location {
    fn from(tokens: Tokens) -> Self {
        tokens_to_location(tokens).unwrap_or(Location::dummy())
    }
}

impl Into<Range<usize>> for Location {
    fn into(self) -> Range<usize> {
        self.start..self.end
    }
}

fn tokens_to_location(mut tokens: Tokens) -> Option<Location> {
    let Some(Some(first)) = tokens.next().map(|t| t.range()) else {
        return None;
    };
    let first = Location::new(first.0.bytes(), first.1.bytes());
    let Some(Some(last)) = tokens.last().map(|t| t.range()) else {
        return Some(first);
    };
    let last = Location::new(last.0.bytes(), last.1.bytes());
    Some(Location::new(first.start, last.end))
}

impl std::ops::Add for Location {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.start, other.end)
    }
}
