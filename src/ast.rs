#[derive(Debug, Eq, PartialEq)]
pub struct Resolution {
    pub width: u64,
    pub height: u64,
}

#[derive(Debug)]
pub struct Manifest<'input> {
    tags: Vec<Tag<'input>>,
}

impl<'input> Manifest<'input> {
    pub fn tags(&self) -> ManifestTags {
        ManifestTags {
            tags: &self.tags,
            next_index: 0,
        }
    }
}

pub struct ManifestTags<'input> {
    tags: &'input Vec<Tag<'input>>,
    next_index: usize,
}

impl<'input> Iterator for ManifestTags<'input> {
    type Item = &'input Tag<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index == self.tags.len() {
            return None;
        }

        let tag = &self.tags[self.next_index];
        self.next_index += 1;
        Some(tag)
    }
}

impl<'input> Manifest<'input> {
    pub fn new(tags: Vec<Tag<'input>>) -> Self {
        Self { tags }
    }
}

#[derive(Debug)]
pub enum AttrValue<'input> {
    EnumString(&'input str),
    Float(f64),
    HexSequence(Vec<u8>),
    Integer(u64),
    QuotedString(&'input str),
    Resolution { width: u64, height: u64 },
}

#[derive(Debug)]
pub struct Attr<'input> {
    pub key: &'input str,
    pub value: AttrValue<'input>,
}

impl<'input> Attr<'input> {
    pub fn new(key: &'input str, value: AttrValue<'input>) -> Self {
        Self { key, value }
    }
}

pub type AttrList<'input> = Vec<Attr<'input>>;

#[derive(Debug)]
pub enum Tag<'input> {
    Header,
    Version(u64),
    Inf {
        duration: f64,
        title: Option<&'input str>,
    },
    Byterange {
        n: u64,
        o: Option<u64>,
    },
    Discontinuity,
    Key(AttrList<'input>),
    Map(AttrList<'input>),
    ProgramDateTime(&'input str),
    Daterange(AttrList<'input>),
    TargetDuration(u64),
    MediaSequence(u64),
    DiscontinuitySequence(u64),
    EndList,
    PlaylistType(&'input str),
    IFramesOnly,
    Media(AttrList<'input>),
    StreamInf {
        attrs: AttrList<'input>,
        uri: &'input str,
    },
    IFrameStreamInf(AttrList<'input>),
    SessionData(AttrList<'input>),
    SessionKey(AttrList<'input>),
    IndependentSegments,
    Start(AttrList<'input>),
    Unknown(&'input str),
    Comment,
    Uri(&'input str),
}
