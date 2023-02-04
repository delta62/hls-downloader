use crossbeam_deque::Worker;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::work_queue::WorkItem;

const RETRY_WAIT_MS: u64 = 500;

pub struct DownloadWorker {
    output_dir: String,
    worker_count: usize,
}

impl DownloadWorker {
    pub fn new(output_dir: String, worker_count: usize) -> Self {
        Self {
            output_dir,
            worker_count,
        }
    }

    pub async fn run(&mut self, worker: &Worker<WorkItem>, stop: Arc<AtomicBool>) {
        let mut worker_handles = Vec::with_capacity(self.worker_count);

        for _ in 0..self.worker_count {
            let stealer = worker.stealer();
            let stop = stop.clone();
            let output_dir = self.output_dir.clone();

            let task = tokio::spawn(async move {
                loop {
                    match stealer.steal() {
                        crossbeam_deque::Steal::Empty => {
                            tokio::time::sleep(Duration::from_millis(RETRY_WAIT_MS)).await;
                            if stop.load(Ordering::Relaxed) {
                                break;
                            }
                        }
                        crossbeam_deque::Steal::Retry => {
                            log::warn!("failed to read from the download queue. retrying...");
                            tokio::time::sleep(Duration::from_millis(RETRY_WAIT_MS)).await;
                        }
                        crossbeam_deque::Steal::Success(work_item) => {
                            let res = reqwest::get(work_item.remote_url.as_str()).await.unwrap();

                            if !res.status().is_success() {
                                panic!("oh noes {} -> {:?}", res.url(), res.status());
                            }

                            let body = res.bytes().await.unwrap();
                            log::debug!("{:?}", body);

                            crate::fs::mkdirp(output_dir.as_str(), &work_item).unwrap();
                            std::fs::write(work_item.local_path, body).unwrap();
                        }
                    }
                }
            });

            worker_handles.push(task);
        }

        for handle in worker_handles {
            handle.await.unwrap();
        }
    }
}
