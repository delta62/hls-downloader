use lazy_static::lazy_static;
use std::path::{Path, PathBuf};
use url::{ParseError, Url};

lazy_static! {
    static ref BASE_URL: Url = Url::parse("http://example.org").unwrap();
}

pub fn parse_path_from_url(url: &str) -> Result<impl AsRef<Path>, ParseError> {
    let complete_result = parse_path_from_complete_url(url);

    if let Err(ParseError::RelativeUrlWithoutBase) = complete_result {
        parse_path_from_relative_url(url)
    } else {
        complete_result
    }
}

fn parse_path_from_complete_url(url: &str) -> Result<PathBuf, ParseError> {
    let url = Url::parse(url)?;
    let path = Path::new(url.path()).to_path_buf();

    Ok(path)
}

fn parse_path_from_relative_url(url: &str) -> Result<PathBuf, ParseError> {
    let url = BASE_URL.join(url)?;
    let path = Path::new(url.path()).to_path_buf();

    Ok(path)
}
