mod args;
mod download_worker;
mod fs;
mod manifest_watcher;
mod work_queue;

use clap::Parser;
use crossbeam_deque::Worker;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use url::Url;

use args::Args;
use hls::Line;
use manifest_watcher::{FileAdd, ManifestWatcher};
use work_queue::FileType;

const WORKER_COUNT: usize = 4;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    let args = Args::parse();
    let base_url = Url::parse(args.base_url.as_str()).unwrap();
    let manifest = read_manifest(args.manifest_path);
    let worker = Worker::new_fifo();
    let is_done = Arc::new(AtomicBool::new(false));
    let mut worker_handles = Vec::with_capacity(WORKER_COUNT);

    for _ in 0..WORKER_COUNT {
        let stealer = worker.stealer();
        let is_done = is_done.clone();
        worker_handles.push(tokio::spawn(async move {
            while !is_done.load(Ordering::Relaxed) {
                match stealer.steal() {
                    crossbeam_deque::Steal::Empty => {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                    crossbeam_deque::Steal::Retry => {
                        log::warn!("failed to read from the download queue. retrying...");
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                    crossbeam_deque::Steal::Success(work_item) => {
                        log::debug!("Stole some work {:?}", work_item);
                    }
                }
            }
        }));
    }

    let mut watcher = ManifestWatcher::new(|message| match message {
        FileAdd::Segment(s) => {
            let work_item =
                fs::parse_path_from_url(&base_url, s.as_str(), FileType::MediaSegment).unwrap();
            worker.push(work_item);
        }
        FileAdd::Key(s) => {
            let work_item = fs::parse_path_from_url(&base_url, s.as_str(), FileType::Key).unwrap();
            worker.push(work_item);
        }
    });

    watcher.update(manifest);

    while !worker.is_empty() {
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    is_done.store(true, Ordering::Relaxed);

    for handle in worker_handles {
        handle.await.unwrap();
    }
}

fn read_manifest<P: AsRef<Path>>(path: P) -> Vec<Line> {
    let input = std::fs::read_to_string(path).unwrap();
    hls::from_str(input.as_str()).unwrap()
}
