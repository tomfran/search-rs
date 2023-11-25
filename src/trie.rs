use std::collections::HashMap;

#[derive(Default)]
struct Node {
    value: Option<i32>,
    children: HashMap<char, Node>,
}

pub struct Trie {
    root: Node,
}

impl Trie {
    pub fn new() -> Trie {
        Trie {
            root: Node::default(),
        }
    }

    pub fn insert(&mut self, word: &str, value: i32) {
        let mut node = &mut self.root;

        for c in word.chars() {
            node = node.children.entry(c).or_default()
        }

        node.value = Some(value);
    }

    pub fn get(&self, word: &str) -> Option<i32> {
        self.get_internal(word)
            .filter(|n| n.value.is_some())
            .map(|n| n.value.unwrap())
    }

    pub fn get_by_prefix(&self, prefix: &str) -> Vec<i32> {
        match self.get_internal(prefix) {
            Some(node) => return self.visit(node),
            None => Vec::new(),
        }
    }

    fn get_internal(&self, word: &str) -> Option<&Node> {
        let mut node = &self.root;

        for c in word.chars() {
            match node.children.get(&c) {
                Some(next_node) => node = next_node,
                None => return None,
            }
        }

        Some(node)
    }

    fn visit(&self, node: &Node) -> Vec<i32> {
        vec![1, 2, 3]
    }
}
