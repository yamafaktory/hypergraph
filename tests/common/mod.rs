#![deny(unsafe_code, nonstandard_style)]

use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Vertex<'a> {
    cost: usize,
    name: &'a str,
}

impl<'a> Vertex<'a> {
    pub fn new(name: &'a str, cost: usize) -> Self {
        Vertex { cost, name }
    }
}

impl<'a> Display for Vertex<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self)
    }
}

impl<'a> Into<usize> for Vertex<'a> {
    fn into(self) -> usize {
        self.cost
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct HyperEdge<'a> {
    cost: usize,
    name: &'a str,
}

impl<'a> HyperEdge<'a> {
    pub fn new(name: &'a str, cost: usize) -> Self {
        HyperEdge { cost, name }
    }
}

impl<'a> Display for HyperEdge<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self)
    }
}

impl<'a> Into<usize> for HyperEdge<'a> {
    fn into(self) -> usize {
        self.cost
    }
}
