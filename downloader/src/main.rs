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
use work_queue::{FileType, WorkQueue};

fn main() {
    env_logger::init();

    let args = Args::parse();
    let base_url = Url::parse(args.base_url.as_str()).unwrap();
    let manifest = read_manifest(args.manifest_path);
    let mut queue = WorkQueue::new();

    let mut watcher = ManifestWatcher::new(|message| match message {
        FileAdd::Segment(s) => {
            let work_item =
                fs::parse_path_from_url(&base_url, s.as_str(), FileType::MediaSegment).unwrap();
            queue.add(work_item);
        }
        FileAdd::Key(s) => {
            let work_item = fs::parse_path_from_url(&base_url, s.as_str(), FileType::Key).unwrap();
            queue.add(work_item);
        }
    });

    watcher.update(manifest);

    while let Some(work_item) = queue.take() {
        fs::mkdirp(args.output_dir.as_str(), &work_item).unwrap();
        // log::debug!("{:?}", work_item);
    }
}

fn read_manifest<P: AsRef<Path>>(path: P) -> Vec<Line> {
    let input = std::fs::read_to_string(path).unwrap();
    hls::from_str(input.as_str()).unwrap()
}
