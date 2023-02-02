mod args;
mod fs;
mod manifest_watcher;
mod work_queue;

use clap::Parser;
use std::path::Path;
use url::Url;

use args::Args;
use hls::Line;
use manifest_watcher::{FileAdd, ManifestWatcher};
use work_queue::WorkQueue;

fn main() {
    env_logger::init();

    let args = Args::parse();
    let manifest_url = Url::parse(args.manifest_url.as_str()).unwrap();
    let path = std::env::args().nth(1).expect("Expected manifest to parse");
    let manifest = read_manifest(path);
    let mut queue = WorkQueue::new();

    let mut watcher = ManifestWatcher::new(|message| match message {
        FileAdd::Segment(s) | FileAdd::Key(s) => {
            let work_item = fs::parse_path_from_url(&manifest_url, s.as_str()).unwrap();
            queue.add(work_item);
        }
    });

    watcher.update(manifest);
}

fn read_manifest<P: AsRef<Path>>(path: P) -> Vec<Line> {
    let input = std::fs::read_to_string(path).unwrap();
    hls::from_str(input.as_str()).unwrap()
}
