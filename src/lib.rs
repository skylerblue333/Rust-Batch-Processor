use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct BatchItem {
    pub id: u64,
    pub payload: String,
}

pub struct BatchProcessor {
    queue: Arc<Mutex<VecDeque<BatchItem>>>,
    batch_size: usize,
    processed: Arc<Mutex<Vec<BatchItem>>>,
}

impl BatchProcessor {
    pub fn new(batch_size: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            batch_size,
            processed: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn enqueue(&self, item: BatchItem) {
        let mut q = self.queue.lock().unwrap();
        q.push_back(item);
    }

    pub fn process_batch(&self) -> usize {
        let mut q = self.queue.lock().unwrap();
        let mut processed = self.processed.lock().unwrap();
        let count = q.len().min(self.batch_size);
        for _ in 0..count {
            if let Some(item) = q.pop_front() {
                processed.push(item);
            }
        }
        count
    }

    pub fn processed_count(&self) -> usize {
        self.processed.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_processing() {
        let processor = BatchProcessor::new(3);
        for i in 0..10 {
            processor.enqueue(BatchItem { id: i, payload: format!("item-{}", i) });
        }
        let processed = processor.process_batch();
        assert_eq!(processed, 3);
        assert_eq!(processor.processed_count(), 3);
    }

    #[test]
    fn test_empty_batch() {
        let processor = BatchProcessor::new(5);
        let processed = processor.process_batch();
        assert_eq!(processed, 0);
    }
}
