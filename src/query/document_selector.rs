use std::{cmp::Ordering, collections::BinaryHeap};

#[derive(Debug)]
struct DocumentScore {
    id: u32,
    score: f32,
}

impl PartialEq for DocumentScore {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for DocumentScore {}

impl PartialOrd for DocumentScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.score.partial_cmp(&self.score)
    }
}

impl Ord for DocumentScore {
    fn cmp(&self, other: &Self) -> Ordering {
        other.partial_cmp(self).unwrap()
    }
}

pub struct DocumentSelector {
    heap: BinaryHeap<DocumentScore>,
    capacity: usize,
}

impl DocumentSelector {
    pub fn new(capacity: usize) -> DocumentSelector {
        DocumentSelector {
            heap: BinaryHeap::new(),
            capacity,
        }
    }

    pub fn push(&mut self, id: u32, score: f32) {
        self.heap.push(DocumentScore { id, score });

        if self.heap.len() > self.capacity {
            self.heap.pop();
        }
    }

    pub fn get_sorted_ids(&mut self) -> Vec<u32> {
        let mut res: Vec<u32> = (0..self.capacity)
            .map(|_| self.heap.pop())
            .filter(|e| e.is_some())
            .map(|e: Option<DocumentScore>| e.unwrap().id)
            .collect();

        res.reverse();
        res
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_top_k() {
        let mut selector = DocumentSelector::new(2);

        selector.push(2, 0.4);
        selector.push(3, 0.3);
        selector.push(1, 0.5);
        selector.push(4, 0.2);

        assert_eq!(selector.get_sorted_ids(), [1, 2]);
    }

    #[test]
    fn test_top_less_than_k() {
        let mut selector = DocumentSelector::new(3);

        selector.push(1, 0.5);
        selector.push(2, 0.4);

        assert_eq!(selector.get_sorted_ids(), [1, 2]);
    }
}
