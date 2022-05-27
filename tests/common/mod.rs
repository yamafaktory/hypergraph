#![deny(unsafe_code, nonstandard_style)]

use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Vertex<'a> {
    name: &'a str,
}

impl<'a> Vertex<'a> {
    pub fn new(name: &'a str) -> Self {
        Vertex { name }
    }
}

impl<'a> Display for Vertex<'a> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", self)
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Hyperedge<'a> {
    cost: usize,
    name: &'a str,
}

impl<'a> Hyperedge<'a> {
    pub fn new(name: &'a str, cost: usize) -> Self {
        Hyperedge { cost, name }
    }
}

impl<'a> Display for Hyperedge<'a> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", self)
    }
}

impl<'a> From<Hyperedge<'a>> for usize {
    fn from(Hyperedge { cost, .. }: Hyperedge<'a>) -> Self {
        cost
    }
}
