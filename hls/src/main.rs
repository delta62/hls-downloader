use serde::Deserialize;

#[derive(Debug, Deserialize)]
enum MediaType {
    Audio,
    Video,
    Subtitles,
    ClosedCaptions,
}

#[derive(Debug, Deserialize)]
struct MediaAttributes<'a> {
    #[serde(rename = "type")]
    media_type: MediaType,
    uri: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
enum Tag<'a> {
    M3u,
    IndependentSegments,
    #[serde(borrow)]
    Media(MediaAttributes<'a>),
    Version(u64),
}

#[derive(Debug, Deserialize)]
enum Line<'a> {
    Tag(Tag<'a>),
    Uri(&'a str),
}

fn main() {
    let path = std::env::args().nth(1).expect("Expected manifest to parse");
    let input = std::fs::read_to_string(path).unwrap();
    let manifest: Vec<Line> = serde_hls::from_str(input.as_str()).unwrap();
    println!("{:#?}", manifest);
}
