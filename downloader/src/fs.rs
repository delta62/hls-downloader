use lazy_static::lazy_static;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;
use url::{ParseError, Url};

use crate::work_queue::FileType;
use crate::work_queue::WorkItem;

pub fn parse_path_from_url(
    manifest_url: &Url,
    url: &str,
    file_type: FileType,
) -> Result<WorkItem, ParseError> {
    let remote_url = Url::parse(url).or_else(|e| {
        if matches!(e, ParseError::RelativeUrlWithoutBase) {
            manifest_url.join(url)
        } else {
            Err(e)
        }
    })?;

    // Skip leading '/' of URL path
    let local_path = Path::new(&remote_url.path()[1..]).to_path_buf();

    Ok(WorkItem::new(local_path, remote_url, file_type))
}

pub fn mkdirp(output_dir: &str, work_item: &WorkItem) -> std::io::Result<()> {
    lazy_static! {
        static ref MKDIR_CACHE: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    }

    let local_base = local_base_dir(work_item);
    let path = Path::new(output_dir)
        .join(local_base)
        .join(work_item.local_path.as_path().parent().unwrap());

    debug_assert!(path.starts_with(output_dir));

    let mut cache = MKDIR_CACHE.lock().unwrap();

    if !cache.contains(path.to_str().unwrap()) {
        log::debug!("mkdirp {:?}", path);
        std::fs::create_dir_all(path.clone())?;
        cache.insert(path.to_str().unwrap().to_owned());
    }

    Ok(())
}

fn local_base_dir(work_item: &WorkItem) -> &'static str {
    match work_item.file_type {
        FileType::Key => "keys",
        FileType::MediaSegment => "segments",
    }
}
