use crate::prelude::*;
use std::collections::VecDeque;

struct Node<T> {
    data: T,
    next: Vec<usize>,
    deg: usize,
}

pub struct Dag<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Dag<T> {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
        }
    }

    /// Pushes a new node into the graph
    pub fn push(&mut self, data: T) -> usize {
        self.nodes.push(Node {
            data,
            next: Default::default(),
            deg: 0,
        });
        self.nodes.len() - 1
    }

    /// Adds an edge from a node to another node
    pub fn add(&mut self, from: usize, to: usize) -> bool {
        if let Some(to) = self.nodes.get_mut(to) {
            to.deg += 1;
        } else {
            return false;
        }
        if let Some(from) = self.nodes.get_mut(from) {
            from.next.push(to);
            true
        } else {
            false
        }
    }

    pub fn topo_sort(mut self) -> Option<Vec<T>> {
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        for (index, node) in self.nodes.iter().enumerate() {
            if node.deg == 0 {
                queue.push_back(index);
            }
        }
        while let Some(from) = queue.pop_front() {
            result.push(from);
            if let Some(from) = self.nodes.get_mut(from) {
                let next = std::mem::take(&mut from.next);
                for to in next.into_iter() {
                    if let Some(next) = self.nodes.get_mut(to) {
                        next.deg = next.deg.saturating_sub(1);
                        if next.deg == 0 {
                            queue.push_back(to);
                        }
                    }
                }
            }
        }
        if result.len() == self.nodes.len() {
            let mut output = self
                .nodes
                .into_iter()
                .map(|node| node.data)
                .collect::<Vec<_>>();
            for (prev, next) in result.into_iter().enumerate() {
                output.swap(prev, next);
            }
            Some(output)
        } else {
            None
        }
    }
}
