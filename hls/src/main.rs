use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
enum MediaType {
    Audio,
    Video,
    Subtitles,
    ClosedCaptions,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
enum PlaylistType {
    Event,
    Vod,
}

#[derive(Debug, Deserialize)]
struct MediaAttributes<'a> {
    #[serde(rename = "type")]
    media_type: MediaType,
    uri: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
enum EncryptionMethod {
    #[serde(rename = "AES-128")]
    Aes128,
    None,
    SampleAes,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
struct KeyAttributes {
    method: EncryptionMethod,
    uri: Option<String>,
    #[serde(with = "serde_bytes")]
    iv: Option<Vec<u8>>,
    keyformat: Option<String>,
    keyformatversions: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
enum Tag<'a> {
    M3u,
    IndependentSegments,
    Inf(f64),
    Key(KeyAttributes),
    #[serde(borrow)]
    Media(MediaAttributes<'a>),
    MediaSequence(u64),
    Targetduration(u64),
    Version(u64),
    PlaylistType(PlaylistType),
    ProgramDateTime(String),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
enum Line<'a> {
    #[serde(borrow)]
    Tag(Tag<'a>),
    Uri(String),
}

fn main() {
    env_logger::init();

    let path = std::env::args().nth(1).expect("Expected manifest to parse");
    let input = std::fs::read_to_string(path).unwrap();
    let manifest: Vec<Line> = serde_hls::from_str(input.as_str()).unwrap();
    println!("{:#?}", manifest);
}
