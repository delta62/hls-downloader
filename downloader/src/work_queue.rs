use std::collections::VecDeque;
use std::path::PathBuf;

pub struct WorkItem {
    path: PathBuf,
}

impl WorkItem {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

pub struct WorkQueue {
    work: VecDeque<WorkItem>,
}

impl WorkQueue {
    pub fn new() -> Self {
        let work = VecDeque::new();
        Self { work }
    }

    pub fn add(&mut self, item: WorkItem) {
        self.work.push_back(item);
    }

    pub fn is_empty(&self) -> bool {
        self.work.is_empty()
    }

    pub fn take(&mut self) -> Option<WorkItem> {
        self.work.pop_front()
    }
}
