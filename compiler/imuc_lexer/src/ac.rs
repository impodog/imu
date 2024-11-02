use std::collections::{btree_map::Entry, BTreeMap};

/// A node of the trie used in [`AhoCorasick`]
struct TrieNode<T> {
    node: BTreeMap<char, usize>,
    fail: usize,
    finish: Option<T>,
}

impl<T> Default for TrieNode<T> {
    fn default() -> Self {
        Self {
            node: BTreeMap::new(),
            fail: 0,
            finish: None,
        }
    }
}

/// A raw trie used to insert new elements
/// Call [`Self::build`] to generate a [`AhoCorasick`] automaton
pub struct AhoCorasickBuilder<T> {
    nodes: Vec<TrieNode<T>>,
}

/// A built aho-corasick automaton that can be used for querying
/// You need to build this from [`AhoCorasickBuilder`]
///
/// ```
/// use imuc_lexer::{AhoCorasickBuilder, AhoCorasick};
///
/// // Build the automaton
/// let mut ac = AhoCorasickBuilder::<&'static str>::default();
/// let dog_pos = ac.insert("dog", "cute");
/// let cat_pos = ac.insert("cat", "also cute");
/// let ac = ac.build();
/// // Query the automaton
/// let mut pos = 0;
/// for ch in "cat".chars() {
///     pos = ac.query(pos, ch);
/// }
/// assert_eq!(pos, cat_pos);
/// assert_eq!(Some("also cute"), ac.finish(pos).cloned());
/// ```
pub struct AhoCorasick<T> {
    nodes: Vec<TrieNode<T>>,
}

impl<T> Default for AhoCorasickBuilder<T> {
    fn default() -> Self {
        Self {
            nodes: vec![TrieNode::<T>::default()],
        }
    }
}

impl<T> AhoCorasickBuilder<T> {
    fn create_node(&mut self) -> usize {
        self.nodes.push(TrieNode::default());
        self.nodes.len() - 1
    }

    fn forward(&mut self, pos: usize, ch: char) -> usize {
        let len = self.nodes.len();
        let value = if let Some(node) = self.nodes.get_mut(pos) {
            match node.node.entry(ch) {
                Entry::Vacant(vacant) => {
                    vacant.insert(len);
                    None
                }
                Entry::Occupied(occupied) => Some(*occupied.get()),
            }
        } else {
            unreachable!("Invalid position");
        };
        value.unwrap_or_else(|| self.create_node())
    }

    fn mark_finished(&mut self, pos: usize, value: T) {
        if let Some(node) = self.nodes.get_mut(pos) {
            node.finish = Some(value);
        } else {
            unreachable!("Invalid position");
        }
    }

    /// Insert a string into the automaton trie, assigning a value
    ///
    /// The return value is the end position of the inserted string
    /// If the same string is inserted for many times, the value will be overridden each insertion
    pub fn insert(&mut self, s: &str, value: T) -> usize {
        let mut pos = 0;
        for ch in s.chars() {
            pos = self.forward(pos, ch);
        }
        self.mark_finished(pos, value);
        pos
    }

    fn build_pos(&mut self, pos: usize, parent: usize) {
        if let Some(node) = self.nodes.get_mut(pos) {
            let list = node
                .node
                .iter()
                .map(|(ch, next)| (*ch, *next))
                .collect::<Vec<_>>();
            for (ch, next) in list.into_iter() {
                let mut fail = parent;
                while let Some(fail_node) = self.nodes.get(fail) {
                    if let Some(&next_fail) = fail_node.node.get(&ch) {
                        self.nodes[next].fail = next_fail;
                        break;
                    }
                    fail = fail_node.fail;
                }
                self.build_pos(next, pos);
            }
        } else {
            unreachable!("Invalid position");
        }
    }

    /// Consume the builder, and get a [`AhoCorasick`] with data moved
    pub fn build(mut self) -> AhoCorasick<T> {
        self.nodes.get_mut(0).expect("Nodes are empty").fail = usize::MAX;
        self.build_pos(0, usize::MAX);
        AhoCorasick::<T> { nodes: self.nodes }
    }
}

impl<T> AhoCorasick<T> {
    /// Query the next position of the automaton, or return usize::MAX if no more matches are
    /// possible, see [`AhoCorasick`] for example of usage
    ///
    /// It is safe to give unavailable pos(usually usize::MAX) to this function,
    /// and it will return the same pos again.
    pub fn query(&self, mut pos: usize, ch: char) -> usize {
        while let Some(next_node) = self.nodes.get(pos) {
            if let Some(&next_pos) = next_node.node.get(&ch) {
                return next_pos;
            }
            pos = next_node.fail;
        }
        pos
    }

    /// Get the finish value(if any) of the current position, after querying the whole string
    pub fn finish(&self, pos: usize) -> Option<&T> {
        self.nodes.get(pos).and_then(|node| node.finish.as_ref())
    }
}
