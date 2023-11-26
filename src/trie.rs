use std::collections::{HashMap, VecDeque};

#[derive(Default)]
struct Node<T> {
    value: Option<T>,
    children: HashMap<char, Node<T>>,
}

#[allow(dead_code)]
pub struct Trie<T>
where
    T: Default + Clone + Copy,
{
    root: Node<T>,
}

#[allow(dead_code)]
impl<T> Trie<T>
where
    T: Default + Clone + Copy,
{
    pub fn new() -> Trie<T> {
        Trie {
            root: Node::default(),
        }
    }

    pub fn insert(&mut self, word: &str, value: T) {
        let mut node = &mut self.root;

        for c in word.chars() {
            node = node.children.entry(c).or_default()
        }

        node.value = Some(value);
    }

    pub fn get(&self, word: &str) -> Option<T> {
        self.get_internal(word).and_then(|n| n.value)
    }

    pub fn get_by_prefix(&self, prefix: &str) -> Vec<T> {
        self.get_internal(prefix)
            .map_or_else(Vec::new, |n| self.visit(n))
    }

    fn get_internal(&self, word: &str) -> Option<&Node<T>> {
        let mut node = &self.root;

        for c in word.chars() {
            match node.children.get(&c) {
                Some(next_node) => node = next_node,
                None => return None,
            }
        }

        Some(node)
    }

    fn visit(&self, node: &Node<T>) -> Vec<T> {
        let mut res: Vec<T> = Vec::new();
        let mut queue: VecDeque<&Node<T>> = VecDeque::new();
        queue.push_back(node);

        while let Some(el) = queue.pop_front() {
            for adj in &el.children {
                queue.push_back(adj.1)
            }

            if let Some(v) = el.value {
                res.push(v)
            }
        }

        res
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut trie = Trie::new();

        trie.insert("hello", 42);

        assert_eq!(trie.get("hello"), Some(42));
        assert_eq!(trie.get("world"), None);
    }

    #[test]
    fn test_get_by_prefix() {
        let mut trie = Trie::new();

        trie.insert("hello", 42);
        trie.insert("help", 99);
        trie.insert("world", 123);

        let a = vec![42, 99];
        assert!(trie.get_by_prefix("hel").iter().all(|e| a.contains(e)))
    }
}
