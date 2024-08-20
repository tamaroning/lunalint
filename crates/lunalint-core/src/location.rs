use std::ops::Range;

use full_moon::{
    node::{Node, Tokens},
    tokenizer::TokenReference,
};

#[derive(Debug, Clone, Copy)]
pub struct Location {
    start: usize,
    end: usize,
}

pub const DUMMY_LOCATION: Location = Location { start: 0, end: 0 };

impl Location {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> usize {
        self.start
    }
}

impl From<Tokens<'_>> for Location {
    fn from(tokens: Tokens) -> Self {
        tokens_to_location(tokens).unwrap()
    }
}

impl From<&TokenReference> for Location {
    fn from(token: &TokenReference) -> Self {
        let Some(range) = token.range() else {
            return DUMMY_LOCATION;
        };
        Location::new(range.0.bytes(), range.1.bytes())
    }
}

impl From<Location> for Range<usize> {
    fn from(val: Location) -> Self {
        val.start..val.end
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
