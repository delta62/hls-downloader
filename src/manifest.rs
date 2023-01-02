use crate::ast::{self, Attr, AttrList, AttrValue, Tag};
use chrono::{DateTime, FixedOffset};
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ParseError {
    MissingAttribute(&'static str),
    MissingExtInf,
    MissingHeader,
    MultipleHeaders,
    InvalidAttrType(String),
    InvalidAttrEnum,
    InvalidDateTime(String),
    UnknownEncryptionType(String),
    UnknownHdcpLevel(String),
    UnknownMediaType(String),
    UnknownPlaylistType(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParseError {}

#[derive(Debug)]
enum PlaylistType {
    Event,
    Vod,
}

#[derive(Clone, Copy, Debug)]
pub enum TrackType {
    Audio,
    ClosedCaptions,
    Subtitles,
    Video,
}

#[derive(Clone, Copy, Debug)]
pub enum ManfiestType {
    Master,
    Media,
}

#[derive(Debug)]
struct Track {
    assoc_language: Option<String>,
    autoselect: bool,
    characteristics: Option<String>,
    default: bool,
    forced: bool,
    group_id: String,
    instream_id: Option<String>,
    language: Option<String>,
    name: String,
    track_type: TrackType,
    uri: Option<String>,
}

#[derive(Debug)]
struct Resolution {
    width: u64,
    height: u64,
}

#[derive(Debug)]
enum HdcpLevel {
    Type0,
    None,
}

#[derive(Debug)]
enum ClosedCaptions {
    None,
    GroupId(String),
}

#[derive(Debug)]
struct Variant {
    audio: Option<String>,
    average_bandwidth: Option<u64>,
    bandwidth: u64,
    closed_captions: Option<ClosedCaptions>,
    codecs: Vec<String>,
    frame_rate: Option<f64>,
    hdcp_level: Option<HdcpLevel>,
    resolution: Option<Resolution>,
    subtitles: Option<String>,
    uri: String,
    video: Option<String>,
}

#[derive(Debug)]
struct IframeVariant {
    average_bandwidth: Option<u64>,
    bandwidth: u64,
    codecs: Vec<String>,
    hdcp_level: Option<HdcpLevel>,
    resolution: Option<Resolution>,
    uri: String,
    video: Option<String>,
}

#[derive(Debug)]
pub struct MasterManfiest {
    audio: Vec<Track>,
    iframe_variants: Vec<IframeVariant>,
    independent_segments: bool,
    variants: Vec<Variant>,
    version: u64,
    video: Vec<Track>,
}

#[derive(Debug)]
struct Segment {
    byte_length: Option<u64>,
    byte_offset: Option<u64>,
    date_time: Option<DateTime<FixedOffset>>,
    discontinuity: bool,
    duration: f64,
    init_byte_range: Option<String>,
    init_uri: Option<String>,
    keys: Vec<EncryptionKey>,
    title: Option<String>,
    uri: String,
}

impl Segment {
    fn from_uri(context: &SegmentContext, uri: &str) -> Result<Self, ParseError> {
        Ok(Self {
            byte_length: context.byte_length,
            byte_offset: context.byte_offset,
            date_time: context.date_time,
            discontinuity: context.discontinuity,
            duration: context.duration.ok_or(ParseError::MissingExtInf)?,
            init_byte_range: context.init_byte_range.clone(),
            init_uri: context.init_uri.clone(),
            keys: context.keys.clone(),
            title: context.title.clone(),
            uri: uri.to_string(),
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum EncryptionMethod {
    None,
    Aes128,
    SampleAes,
}

#[derive(Clone, Debug)]
struct EncryptionKey {
    method: EncryptionMethod,
    uri: Option<String>,
    iv: Option<u128>,
    key_format: String,
    key_format_versions: String,
}

#[derive(Debug, Default)]
pub struct SegmentContext {
    byte_length: Option<u64>,
    byte_offset: Option<u64>,
    date_time: Option<DateTime<FixedOffset>>,
    discontinuity: bool,
    duration: Option<f64>,
    init_byte_range: Option<String>,
    init_uri: Option<String>,
    keys: Vec<EncryptionKey>,
    title: Option<String>,
}

#[derive(Debug)]
struct InitMap {
    uri: String,
    byte_range: Option<String>,
}

impl SegmentContext {
    fn add_key(&mut self, key: EncryptionKey) {
        if let Some(i) = self
            .keys
            .iter()
            .position(|k| k.key_format == key.key_format)
        {
            self.keys.remove(i);
        }

        self.keys.push(key);
    }

    fn set_date_time(&mut self, date_time: DateTime<FixedOffset>) {
        self.date_time = Some(date_time);
    }

    fn set_init_map(&mut self, map: InitMap) {
        self.init_uri = Some(map.uri);
        self.init_byte_range = map.byte_range;
    }

    fn set_duration(&mut self, duration: f64) {
        self.duration = Some(duration);
    }

    fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    fn set_byte_offset(&mut self, offset: Option<u64>) {
        self.byte_offset = offset;
    }

    fn set_byte_length(&mut self, length: u64) {
        self.byte_length = Some(length);
    }

    fn set_discontinuity(&mut self) {
        self.discontinuity = true;
    }

    fn reset(&mut self) {
        self.duration = Default::default();
        self.title = Default::default();
        self.date_time = None;
        self.discontinuity = false;
        self.byte_offset = None;
        self.byte_length = None;
    }
}

#[derive(Debug)]
pub struct MediaManifest {
    discontinuity_sequence: u64,
    end_list: bool,
    first_segment_sequence: u64,
    iframes_only: bool,
    playlist_type: Option<PlaylistType>,
    segments: Vec<Segment>,
    target_duration: u64,
    version: u64,
}

impl MediaManifest {
    pub fn from_ast(ast: &ast::Manifest) -> Result<Self, ParseError> {
        let mut tags = ast.tags();

        match tags.next() {
            Some(Tag::Header) => {}
            _ => Err(ParseError::MissingHeader)?,
        }

        let mut segments = Vec::new();
        let mut version = 1;
        let mut segment_context = SegmentContext::default();
        let mut discontinuity_sequence = 0;
        let mut first_segment_sequence = 0;
        let mut end_list = false;
        let mut target_duration = None;
        let mut playlist_type = None;
        let mut iframes_only = false;

        for tag in tags {
            match tag {
                // Any manifest type
                Tag::Header => Err(ParseError::MultipleHeaders)?,
                Tag::Version(v) => version = *v,

                // Media segment tags
                Tag::Inf { duration, title } => {
                    segment_context.set_duration(*duration);
                    segment_context.set_title(title.to_owned().map(|s| s.to_string()));
                }
                Tag::Map(attrs) => segment_context.set_init_map(parse_map(attrs)?),
                Tag::Byterange { n, o } => {
                    segment_context.set_byte_length(*n);
                    segment_context.set_byte_offset(*o);
                }
                Tag::Uri(uri) => {
                    segments.push(Segment::from_uri(&segment_context, uri)?);
                    segment_context.reset();
                }
                Tag::Discontinuity => {
                    segment_context.set_discontinuity();
                }
                Tag::Key(attrs) => {
                    segment_context.add_key(parse_key(attrs)?);
                }
                Tag::ProgramDateTime(string) => segment_context.set_date_time(
                    DateTime::parse_from_rfc3339(string)
                        .map_err(|_| ParseError::InvalidDateTime(string.to_string()))?,
                ),
                Tag::Daterange(_) => log::warn!("skipping EXT-X-DATERANGE; not implemented"),
                Tag::DiscontinuitySequence(n) => discontinuity_sequence = *n,
                Tag::MediaSequence(n) => first_segment_sequence = *n,
                Tag::EndList => end_list = true,
                Tag::TargetDuration(n) => target_duration = Some(*n),
                Tag::PlaylistType(s) => {
                    playlist_type = match *s {
                        "EVENT" => Some(PlaylistType::Event),
                        "VOD" => Some(PlaylistType::Vod),
                        s => Err(ParseError::UnknownPlaylistType(s.to_string()))?,
                    }
                }
                Tag::IFramesOnly => iframes_only = true,

                _ => {
                    log::warn!("Encountered unimplemented tag {:?}", tag);
                    panic!()
                }
            }
        }

        Ok(Self {
            discontinuity_sequence,
            end_list,
            first_segment_sequence,
            iframes_only,
            playlist_type,
            segments,
            target_duration: target_duration
                .ok_or(ParseError::MissingAttribute("TARGET_DURATION"))?,
            version,
        })
    }
}

impl MasterManfiest {
    pub fn from_ast(ast: &ast::Manifest) -> Result<Self, ParseError> {
        let mut tags = ast.tags();

        match tags.next() {
            Some(Tag::Header) => {}
            _ => Err(ParseError::MissingHeader)?,
        }

        let mut audio = Vec::new();
        let mut video = Vec::new();
        let mut independent_segments = false;
        let mut variants = Vec::new();
        let mut iframe_variants = Vec::new();
        let mut version = 1;

        for tag in tags {
            match tag {
                // Any manifest type
                Tag::Header => Err(ParseError::MultipleHeaders)?,
                Tag::Version(v) => version = *v,
                _ => {
                    log::warn!("Encountered unimplemented tag {:?}", tag);
                    panic!()
                }
                // Master playlist tags
                Tag::IndependentSegments => independent_segments = true,
                Tag::IFrameStreamInf(attrs) => iframe_variants.push(parse_iframe_variant(attrs)?),
                Tag::Media(attrs) => {
                    let track = parse_media(attrs)?;
                    match track.track_type {
                        TrackType::Audio => audio.push(track),
                        TrackType::Video => video.push(track),
                        _ => {
                            log::warn!("Ignoring unused track type {:?}", track.track_type);
                        }
                    }
                }
                Tag::StreamInf { attrs, uri } => {
                    variants.push(parse_variant(attrs, uri)?);
                }
            }
        }

        Ok(Self {
            audio,
            iframe_variants,
            independent_segments,
            variants,
            version,
            video,
        })
    }
}

fn parse_media<'input>(attrs: &'input AttrList) -> Result<Track, ParseError> {
    let mut assoc_language = None;
    let mut autoselect = None;
    let mut characteristics = None;
    let mut default = None;
    let mut forced = None;
    let mut group_id = None;
    let mut instream_id = None;
    let mut language = None;
    let mut name = None;
    let mut track_type = None;
    let mut uri = None;

    for attr in attrs {
        match attr.key {
            "ASSOC-LANGUAGE" => assoc_language = Some(expect_string(attr)?),
            "AUTOSELECT" => autoselect = Some(parse_bool(attr)?),
            "CHARACTERISTICS" => characteristics = Some(expect_string(attr)?),
            "DEFAULT" => default = Some(parse_bool(attr)?),
            "FORCED" => forced = Some(parse_bool(attr)?),
            "GROUP-ID" => group_id = Some(expect_string(attr)?),
            "INSTREAM-ID" => instream_id = Some(expect_string(attr)?),
            "LANGUAGE" => language = Some(expect_string(attr)?),
            "NAME" => name = Some(expect_string(attr)?),
            "TYPE" => track_type = Some(parse_media_type(attr)?),
            "URI" => uri = Some(expect_string(attr)?),
            key => log::warn!("Skipping unimplemented attribute {}", key),
        }
    }

    Ok(Track {
        assoc_language,
        autoselect: autoselect.unwrap_or(false),
        characteristics,
        default: default.unwrap_or(false),
        forced: forced.unwrap_or(false),
        group_id: group_id.ok_or(ParseError::MissingAttribute("GROUP-ID"))?,
        instream_id,
        language,
        name: name.ok_or(ParseError::MissingAttribute("NAME"))?,
        track_type: track_type.ok_or(ParseError::MissingAttribute("TYPE"))?,
        uri,
    })
}

fn parse_variant<'input>(attrs: &'input AttrList, uri: &'input str) -> Result<Variant, ParseError> {
    let mut average_bandwidth = None;
    let mut bandwidth = None;
    let mut closed_captions = None;
    let mut audio = None;
    let mut codecs = None;
    let mut video = None;
    let mut frame_rate = None;
    let mut subtitles = None;
    let mut hdcp_level = None;
    let mut resolution = None;

    for attr in attrs {
        match attr.key {
            "AUDIO" => audio = Some(expect_string(attr)?),
            "AVERAGE-BANDWIDTH" => average_bandwidth = Some(expect_int(attr)?),
            "BANDWIDTH" => bandwidth = Some(expect_int(attr)?),
            "CLOSED-CAPTIONS" => closed_captions = Some(parse_closed_captions(attr)?),
            "CODECS" => codecs = Some(parse_codecs(attr)?),
            "FRAME-RATE" => frame_rate = Some(expect_float(attr)?),
            "HDCP-LEVEL" => hdcp_level = Some(parse_hdcp_level(attr)?),
            "RESOLUTION" => resolution = Some(expect_resolution(attr)?),
            "SUBTITLES" => subtitles = Some(expect_string(attr)?),
            "VIDEO" => video = Some(expect_string(attr)?),
            key => log::warn!("Skipping unimplemented attribute {}", key),
        }
    }

    Ok(Variant {
        audio,
        average_bandwidth,
        bandwidth: bandwidth.ok_or(ParseError::MissingAttribute("BANDWIDTH"))?,
        closed_captions,
        codecs: codecs.ok_or(ParseError::MissingAttribute("CODECS"))?,
        frame_rate,
        hdcp_level,
        resolution,
        subtitles,
        uri: uri.to_string(),
        video,
    })
}

fn parse_iframe_variant<'input>(attrs: &'input AttrList) -> Result<IframeVariant, ParseError> {
    let mut average_bandwidth = None;
    let mut bandwidth = None;
    let mut codecs = None;
    let mut video = None;
    let mut hdcp_level = None;
    let mut resolution = None;
    let mut uri = None;

    for attr in attrs {
        match attr.key {
            "AVERAGE-BANDWIDTH" => average_bandwidth = Some(expect_int(attr)?),
            "BANDWIDTH" => bandwidth = Some(expect_int(attr)?),
            "CODECS" => codecs = Some(parse_codecs(attr)?),
            "HDCP-LEVEL" => hdcp_level = Some(parse_hdcp_level(attr)?),
            "RESOLUTION" => resolution = Some(expect_resolution(attr)?),
            "VIDEO" => video = Some(expect_string(attr)?),
            "URI" => uri = Some(expect_string(attr)?),
            key => log::warn!("Skipping unimplemented attribute {}", key),
        }
    }

    Ok(IframeVariant {
        average_bandwidth,
        bandwidth: bandwidth.ok_or(ParseError::MissingAttribute("BANDWIDTH"))?,
        codecs: codecs.ok_or(ParseError::MissingAttribute("CODECS"))?,
        hdcp_level,
        resolution,
        uri: uri.ok_or(ParseError::MissingAttribute("URI"))?,
        video,
    })
}

fn parse_map<'input>(attrs: &'input AttrList) -> Result<InitMap, ParseError> {
    let mut uri = None;
    let mut byte_range = None;

    for attr in attrs {
        match attr.key {
            "URI" => uri = Some(expect_string(attr)?),
            "BYTERANGE" => byte_range = Some(expect_string(attr)?),
            key => log::warn!("Skipping unimplemented attribute {}", key),
        }
    }

    Ok(InitMap {
        uri: uri.ok_or(ParseError::MissingAttribute("URI"))?,
        byte_range,
    })
}

fn parse_key<'input>(attrs: &'input AttrList) -> Result<EncryptionKey, ParseError> {
    let mut iv = None;
    let mut method = None;
    let mut uri = None;
    let mut key_format = None;
    let mut key_format_versions = None;

    for attr in attrs {
        match attr.key {
            "IV" => {
                let bytes = expect_bytes(attr)?;
                let (bytes, _) = bytes.as_slice().split_at(std::mem::size_of::<u128>());
                let num = u128::from_be_bytes(bytes.try_into().unwrap());
                iv = Some(num)
            }
            "KEYFORMAT" => key_format = Some(expect_string(attr)?),
            "KEYFORMATVERSIONS" => key_format_versions = Some(expect_string(attr)?),
            "METHOD" => {
                method = Some(expect_enum(attr, |m| match m {
                    "NONE" => Ok(EncryptionMethod::None),
                    "AES-128" => Ok(EncryptionMethod::Aes128),
                    "SAMPLE-AES" => Ok(EncryptionMethod::SampleAes),
                    _ => Err(ParseError::UnknownEncryptionType(m.to_string())),
                })?);
            }
            "URI" => uri = Some(expect_string(attr)?),
            key => log::warn!("Skipping unimplemented attribute {}", key),
        }
    }

    Ok(EncryptionKey {
        iv,
        key_format: key_format.unwrap_or("identity".to_string()),
        key_format_versions: key_format_versions.unwrap_or("1".to_string()),
        method: method.ok_or(ParseError::MissingAttribute("METHOD"))?,
        uri,
    })
}

fn parse_closed_captions<'input>(attr: &Attr<'input>) -> Result<ClosedCaptions, ParseError> {
    match attr.value {
        AttrValue::EnumString("NONE") => Ok(ClosedCaptions::None),
        AttrValue::QuotedString(s) => Ok(ClosedCaptions::GroupId(s.to_string())),
        _ => Err(ParseError::InvalidAttrType(attr.key.to_string())),
    }
}

fn parse_codecs<'input>(attr: &Attr<'input>) -> Result<Vec<String>, ParseError> {
    let string = expect_string(attr)?;
    Ok(string.split(',').map(|s| s.to_string()).collect())
}

fn parse_bool<'input>(attr: &Attr<'input>) -> Result<bool, ParseError> {
    expect_enum(attr, |s| match s {
        "YES" => Ok(true),
        "NO" => Ok(false),
        _ => Err(ParseError::InvalidAttrEnum),
    })
}

fn parse_hdcp_level<'input>(attr: &Attr<'input>) -> Result<HdcpLevel, ParseError> {
    expect_enum(attr, |s| match s {
        "NONE" => Ok(HdcpLevel::None),
        "TYPE-0" => Ok(HdcpLevel::Type0),
        _ => Err(ParseError::UnknownHdcpLevel(s.to_string())),
    })
}

fn parse_media_type<'input>(attr: &Attr<'input>) -> Result<TrackType, ParseError> {
    expect_enum(attr, |s| match s {
        "AUDIO" => Ok(TrackType::Audio),
        "CLOSED-CAPTIONS" => Ok(TrackType::ClosedCaptions),
        "SUBTITLES" => Ok(TrackType::Subtitles),
        "VIDEO" => Ok(TrackType::Video),
        _ => Err(ParseError::UnknownMediaType(s.to_string())),
    })
}

fn expect_resolution<'input>(attr: &Attr<'input>) -> Result<Resolution, ParseError> {
    if let AttrValue::Resolution { width, height } = attr.value {
        Ok(Resolution { width, height })
    } else {
        Err(ParseError::InvalidAttrType(attr.key.to_string()))
    }
}

fn expect_int<'input>(attr: &Attr<'input>) -> Result<u64, ParseError> {
    if let AttrValue::Integer(i) = attr.value {
        Ok(i)
    } else {
        Err(ParseError::InvalidAttrType(attr.key.to_string()))
    }
}

fn expect_float<'input>(attr: &Attr<'input>) -> Result<f64, ParseError> {
    if let AttrValue::Float(f) = attr.value {
        Ok(f)
    } else {
        Err(ParseError::InvalidAttrType(attr.key.to_string()))
    }
}

fn expect_enum<'input, T, F>(attr: &Attr<'input>, f: F) -> Result<T, ParseError>
where
    F: Fn(&'input str) -> Result<T, ParseError>,
{
    if let AttrValue::EnumString(string) = attr.value {
        f(string)
    } else {
        Err(ParseError::InvalidAttrType(attr.key.to_string()))
    }
}

fn expect_string<'input>(attr: &Attr<'input>) -> Result<String, ParseError> {
    if let AttrValue::QuotedString(string) = attr.value {
        Ok(string.to_string())
    } else {
        Err(ParseError::InvalidAttrType(attr.key.to_string()))
    }
}

fn expect_bytes<'input>(attr: &'input Attr<'input>) -> Result<&'input Vec<u8>, ParseError> {
    if let AttrValue::HexSequence(hex) = &attr.value {
        Ok(hex)
    } else {
        Err(ParseError::InvalidAttrType(attr.key.to_string()))
    }
}
