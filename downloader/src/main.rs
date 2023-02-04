mod args;
mod downloader;
mod fs;
mod manifest_watcher;
mod work_queue;

use clap::Parser;
use crossbeam_deque::Worker;
use downloader::DownloadWorker;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
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
    let mut downloader = DownloadWorker::new(args.output_dir, WORKER_COUNT);
    let is_done = Arc::new(AtomicBool::new(false));
    let downloads_complete = downloader.run(&worker, is_done.clone());

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

    is_done.store(true, Ordering::Relaxed);
    downloads_complete.await;
}

fn read_manifest<P: AsRef<Path>>(path: P) -> Vec<Line> {
    let input = std::fs::read_to_string(path).unwrap();
    hls::from_str(input.as_str()).unwrap()
}
