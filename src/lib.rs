use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::fmt::{Display, Formatter};
use std::sync::Mutex;

pub const MAX_BATCH_SIZE: usize = 1_000;
pub const MAX_PAYLOAD_BYTES: usize = 16 * 1024;
pub const MAX_TRACKED_IDS: usize = 100_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchItem {
    pub id: u64,
    pub payload: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProcessorStats {
    pub queued: usize,
    pub processed: u64,
    pub tracked_ids: usize,
    pub batch_size: usize,
    pub max_queue: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessorError {
    InvalidConfig(&'static str),
    InvalidItem(&'static str),
    DuplicateId(u64),
    CapacityExceeded(&'static str),
    StateUnavailable,
}

impl Display for ProcessorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConfig(message)
            | Self::InvalidItem(message)
            | Self::CapacityExceeded(message) => {
                write!(f, "{message}")
            }
            Self::DuplicateId(id) => write!(f, "duplicate item id: {id}"),
            Self::StateUnavailable => write!(f, "processor state is unavailable"),
        }
    }
}

impl std::error::Error for ProcessorError {}

#[derive(Debug, Default)]
struct State {
    queue: VecDeque<BatchItem>,
    seen_ids: HashSet<u64>,
    processed_count: u64,
}

#[derive(Debug)]
pub struct BatchProcessor {
    state: Mutex<State>,
    batch_size: usize,
    max_queue: usize,
}

impl BatchProcessor {
    pub fn new(batch_size: usize, max_queue: usize) -> Result<Self, ProcessorError> {
        if !(1..=MAX_BATCH_SIZE).contains(&batch_size) {
            return Err(ProcessorError::InvalidConfig(
                "batch_size must be between 1 and 1000",
            ));
        }
        if max_queue < batch_size || max_queue > MAX_TRACKED_IDS {
            return Err(ProcessorError::InvalidConfig(
                "max_queue must be at least batch_size and no more than 100000",
            ));
        }
        Ok(Self {
            state: Mutex::new(State::default()),
            batch_size,
            max_queue,
        })
    }

    pub fn enqueue_many(&self, items: Vec<BatchItem>) -> Result<usize, ProcessorError> {
        if items.is_empty() || items.len() > MAX_BATCH_SIZE {
            return Err(ProcessorError::InvalidItem(
                "enqueue request must contain 1-1000 items",
            ));
        }

        let mut request_ids = HashSet::with_capacity(items.len());
        for item in &items {
            if item.payload.is_empty() || item.payload.len() > MAX_PAYLOAD_BYTES {
                return Err(ProcessorError::InvalidItem(
                    "payload must contain 1-16384 UTF-8 bytes",
                ));
            }
            if !request_ids.insert(item.id) {
                return Err(ProcessorError::DuplicateId(item.id));
            }
        }

        let mut state = self
            .state
            .lock()
            .map_err(|_| ProcessorError::StateUnavailable)?;
        if state.queue.len().saturating_add(items.len()) > self.max_queue {
            return Err(ProcessorError::CapacityExceeded(
                "queue capacity would be exceeded",
            ));
        }
        if state.seen_ids.len().saturating_add(items.len()) > MAX_TRACKED_IDS {
            return Err(ProcessorError::CapacityExceeded(
                "tracked id capacity would be exceeded",
            ));
        }
        if let Some(id) = items
            .iter()
            .map(|item| item.id)
            .find(|id| state.seen_ids.contains(id))
        {
            return Err(ProcessorError::DuplicateId(id));
        }

        for item in items {
            state.seen_ids.insert(item.id);
            state.queue.push_back(item);
        }
        Ok(state.queue.len())
    }

    pub fn process_batch(&self) -> Result<Vec<BatchItem>, ProcessorError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| ProcessorError::StateUnavailable)?;
        let count = state.queue.len().min(self.batch_size);
        let processed: Vec<_> = state.queue.drain(..count).collect();
        state.processed_count = state.processed_count.saturating_add(processed.len() as u64);
        Ok(processed)
    }

    pub fn stats(&self) -> Result<ProcessorStats, ProcessorError> {
        let state = self
            .state
            .lock()
            .map_err(|_| ProcessorError::StateUnavailable)?;
        Ok(ProcessorStats {
            queued: state.queue.len(),
            processed: state.processed_count,
            tracked_ids: state.seen_ids.len(),
            batch_size: self.batch_size,
            max_queue: self.max_queue,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u64) -> BatchItem {
        BatchItem {
            id,
            payload: format!("item-{id}"),
        }
    }

    #[test]
    fn processes_fifo_batches_and_tracks_stats() {
        let processor = BatchProcessor::new(2, 10).unwrap();
        assert_eq!(
            processor
                .enqueue_many(vec![item(1), item(2), item(3)])
                .unwrap(),
            3
        );
        assert_eq!(processor.process_batch().unwrap(), vec![item(1), item(2)]);
        assert_eq!(
            processor.stats().unwrap(),
            ProcessorStats {
                queued: 1,
                processed: 2,
                tracked_ids: 3,
                batch_size: 2,
                max_queue: 10,
            }
        );
    }

    #[test]
    fn rejects_invalid_configuration() {
        assert!(matches!(
            BatchProcessor::new(0, 10),
            Err(ProcessorError::InvalidConfig(_))
        ));
        assert!(matches!(
            BatchProcessor::new(10, 5),
            Err(ProcessorError::InvalidConfig(_))
        ));
    }

    #[test]
    fn duplicate_requests_are_rejected_atomically() {
        let processor = BatchProcessor::new(10, 10).unwrap();
        processor.enqueue_many(vec![item(1)]).unwrap();
        assert_eq!(
            processor.enqueue_many(vec![item(2), item(1)]),
            Err(ProcessorError::DuplicateId(1))
        );
        assert_eq!(processor.stats().unwrap().queued, 1);
    }

    #[test]
    fn enforces_payload_and_queue_bounds() {
        let processor = BatchProcessor::new(1, 1).unwrap();
        assert!(matches!(
            processor.enqueue_many(vec![BatchItem {
                id: 1,
                payload: String::new()
            }]),
            Err(ProcessorError::InvalidItem(_))
        ));
        processor.enqueue_many(vec![item(1)]).unwrap();
        assert!(matches!(
            processor.enqueue_many(vec![item(2)]),
            Err(ProcessorError::CapacityExceeded(_))
        ));
    }
}
