mod manifest_watcher;

use std::path::Path;

use hls::Line;
use manifest_watcher::{FileAdd, ManifestWatcher};
use url::Url;

fn main() {
    env_logger::init();

    let path = std::env::args().nth(1).expect("Expected manifest to parse");
    let manifest = read_manifest(path);

    let mut watcher = ManifestWatcher::new(|message| match message {
        FileAdd::Segment(s) => {
            let base_url = Url::parse("http://example.com").unwrap();
            let x = base_url.join(s.as_str()).unwrap();
            let x = base_url.make_relative(&x).unwrap();
            let p = Path::new(x.as_str());
            let dir = p.parent().unwrap();
            log::debug!("{:?}", dir);
        }
        FileAdd::Key(k) => {
            let x = Url::parse(k.as_str()).unwrap();
            let base_url = x.origin().ascii_serialization();
            let x = Url::parse(base_url.as_str())
                .unwrap()
                .make_relative(&x)
                .unwrap();
            let p = Path::new(x.as_str());
            let dir = p.parent().unwrap();
            log::debug!("{:?}", dir);
        }
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
