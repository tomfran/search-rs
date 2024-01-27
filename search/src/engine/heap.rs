use std::{cmp::Ordering, collections::BinaryHeap};

#[derive(Debug)]
pub struct Entry {
    pub id: u32,
    pub priority: f64,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for Entry {}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .priority
            .partial_cmp(&self.priority)
            .unwrap_or(Ordering::Equal)
    }
}

pub struct FixedMinHeap {
    heap: BinaryHeap<Entry>,
    capacity: usize,
}

impl FixedMinHeap {
    pub fn new(capacity: usize) -> FixedMinHeap {
        FixedMinHeap {
            heap: BinaryHeap::new(),
            capacity,
        }
    }

    pub fn push(&mut self, id: u32, score: f64) {
        self.heap.push(Entry {
            id,
            priority: score,
        });

        if self.heap.len() > self.capacity {
            self.heap.pop();
        }
    }

    pub fn get_sorted_id_priority_pairs(&mut self) -> Vec<(u32, f64)> {
        let mut res: Vec<(u32, f64)> = (0..self.capacity)
            .filter_map(|_| self.heap.pop().map(|e| (e.id, e.priority)))
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
        let mut selector = FixedMinHeap::new(2);

        selector.push(2, 0.4);
        selector.push(3, 0.3);
        selector.push(1, 0.5);
        selector.push(4, 0.2);

        assert_eq!(
            selector.get_sorted_id_priority_pairs(),
            [(1, 0.5), (2, 0.4)]
        );
    }

    #[test]
    fn test_top_less_than_k() {
        let mut selector = FixedMinHeap::new(3);

        selector.push(1, 0.5);
        selector.push(2, 0.4);

        assert_eq!(
            selector.get_sorted_id_priority_pairs(),
            [(1, 0.5), (2, 0.4)]
        );
    }
}
