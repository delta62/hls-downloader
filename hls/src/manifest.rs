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
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum HdcpLevel {
    None,
    Type0,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub struct MediaAttributes {
    #[serde(rename = "TYPE")]
    pub media_type: MediaType,
    pub uri: Option<String>,
    pub group_id: String,
    pub language: Option<String>,
    pub assoc_language: Option<String>,
    pub name: String,
    pub default: Option<bool>,
    pub autoselect: Option<bool>,
    pub forced: Option<bool>,
    pub instream_id: Option<String>,
    pub characteristics: Option<String>,
    pub channels: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub struct StreamInfAttributes {
    pub bandwidth: u64,
    pub average_bandwidth: Option<u64>,
    pub codecs: Option<String>,
    pub resolution: Option<String>,
    pub frame_rate: Option<f64>,
    pub hdcp_level: Option<HdcpLevel>,
    pub audio: Option<String>,
    pub video: Option<String>,
    pub subtitles: Option<String>,
    pub closed_captions: Option<String>,
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
pub enum Tag {
    M3u,
    IndependentSegments,
    Inf(f64),
    Key(KeyAttributes),
    Media(MediaAttributes),
    MediaSequence(u64),
    Targetduration(u64),
    Version(u64),
    PlaylistType(PlaylistType),
    ProgramDateTime(String),
    StreamInf(StreamInfAttributes),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub enum Line {
    Tag(Tag),
    Uri(String),
}
