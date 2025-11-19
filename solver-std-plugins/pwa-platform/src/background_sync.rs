use std::collections::VecDeque;

/// Background Sync
///
/// Queue actions to be performed when online
pub struct BackgroundSync {
    queue: VecDeque<SyncTask>,
}

#[derive(Debug, Clone)]
pub struct SyncTask {
    pub id: String,
    pub action: String,
    pub data: String,
    pub retries: u32,
}

impl BackgroundSync {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Queue a task for background sync
    pub fn queue(&mut self, action: &str) {
        let task = SyncTask {
            id: format!("sync-{}", self.queue.len()),
            action: action.to_string(),
            data: String::new(),
            retries: 0,
        };

        self.queue.push_back(task);
    }

    /// Queue a task with data
    pub fn queue_with_data(&mut self, action: &str, data: &str) {
        let task = SyncTask {
            id: format!("sync-{}", self.queue.len()),
            action: action.to_string(),
            data: data.to_string(),
            retries: 0,
        };

        self.queue.push_back(task);
    }

    /// Get next task
    pub fn next(&mut self) -> Option<SyncTask> {
        self.queue.pop_front()
    }

    /// Requeue failed task
    pub fn requeue(&mut self, mut task: SyncTask) {
        task.retries += 1;
        if task.retries < 3 {
            self.queue.push_back(task);
        } else {
            eprintln!("[Background Sync] Task failed after 3 retries: {}", task.id);
        }
    }

    /// Get queue size
    pub fn queue_size(&self) -> usize {
        self.queue.len()
    }

    /// Clear queue
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    /// Process all queued tasks
    pub async fn process_all(&mut self) -> Vec<SyncTask> {
        let mut processed = Vec::new();

        while let Some(task) = self.next() {
            eprintln!("[Background Sync] Processing: {}", task.action);
            processed.push(task);
        }

        processed
    }
}

impl Default for BackgroundSync {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_sync() {
        let mut sync = BackgroundSync::new();

        // Queue tasks
        sync.queue("send-message");
        sync.queue("upload-photo");
        assert_eq!(sync.queue_size(), 2);

        // Get next
        let task = sync.next();
        assert!(task.is_some());
        assert_eq!(task.unwrap().action, "send-message");
        assert_eq!(sync.queue_size(), 1);

        // Requeue
        let task = SyncTask {
            id: "test".to_string(),
            action: "retry".to_string(),
            data: String::new(),
            retries: 0,
        };
        sync.requeue(task);
        assert_eq!(sync.queue_size(), 2);
    }
}
