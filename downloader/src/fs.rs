use std::path::Path;
use url::{ParseError, Url};

use crate::work_queue::WorkItem;

pub fn parse_path_from_url(manifest_url: &Url, url: &str) -> Result<WorkItem, ParseError> {
    let remote_url = Url::parse(url).or_else(|e| {
        if matches!(e, ParseError::RelativeUrlWithoutBase) {
            manifest_url.join(url)
        } else {
            Err(e)
        }
    })?;

    let local_path = Path::new(&remote_url.path()[1..]).to_path_buf();

    Ok(WorkItem::new(local_path, remote_url))
}
