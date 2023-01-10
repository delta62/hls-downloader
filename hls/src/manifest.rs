use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum MediaType {
    Audio,
    Video,
    Subtitles,
    ClosedCaptions,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum PlaylistType {
    Event,
    Vod,
}

#[derive(Debug, Deserialize)]
pub struct MediaAttributes<'a> {
    #[serde(rename = "type")]
    pub media_type: MediaType,
    pub uri: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum EncryptionMethod {
    #[serde(rename = "AES-128")]
    Aes128,
    None,
    SampleAes,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub struct KeyAttributes {
    pub method: EncryptionMethod,
    pub uri: Option<String>,
    #[serde(with = "serde_bytes")]
    pub iv: Option<Vec<u8>>,
    pub keyformat: Option<String>,
    pub keyformatversions: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Tag<'a> {
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
pub enum Line<'a> {
    #[serde(borrow)]
    Tag(Tag<'a>),
    Uri(String),
}
