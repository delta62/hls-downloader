use std::collections::VecDeque;
use std::path::PathBuf;

use url::Url;

#[derive(Debug)]
pub enum FileType {
    Key,
    MediaSegment,
}

#[derive(Debug)]
pub struct WorkItem {
    pub local_path: PathBuf,
    pub remote_url: Url,
    pub file_type: FileType,
}

impl WorkItem {
    pub fn new(local_path: PathBuf, remote_url: Url, file_type: FileType) -> Self {
        Self {
            file_type,
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
