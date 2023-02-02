use std::collections::VecDeque;
use std::path::PathBuf;

use url::Url;

pub struct WorkItem {
    local_path: PathBuf,
    remote_url: Url,
}

impl WorkItem {
    pub fn new(local_path: PathBuf, remote_url: Url) -> Self {
        Self {
            local_path,
            remote_url,
        }
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
