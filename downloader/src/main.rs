mod manifest_watcher;

use std::path::Path;

use hls::Line;
use manifest_watcher::ManifestWatcher;

fn main() {
    env_logger::init();

    let path = std::env::args().nth(1).expect("Expected manifest to parse");
    let manifest = read_manifest(path);

    let mut watcher = ManifestWatcher::new(|message| {
        println!("{}", message);
    });

    watcher.update(manifest);

    let path = std::env::args()
        .nth(2)
        .expect("Expected manifest to diff against");
    let next_manifest = read_manifest(path);
    watcher.update(next_manifest);
}

fn read_manifest<P: AsRef<Path>>(path: P) -> Vec<Line> {
    let input = std::fs::read_to_string(path).unwrap();
    hls::from_str(input.as_str()).unwrap()
}
