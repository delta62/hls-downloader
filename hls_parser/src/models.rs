use crate::parser::all_tags;
use nom::{error::Error, Finish};

#[derive(Debug)]
pub struct Manifest<'a> {
    lines: Vec<Line<'a>>,
}

impl<'a> Manifest<'a> {
    pub fn parse(s: &'a str) -> Result<Self, Error<String>> {
        match all_tags(s).finish() {
            Ok((remaining, lines)) => {
                if !remaining.is_empty() {
                    log::error!("Failed to parse! Next 3 lines:");
                    for i in 0..3 {
                        log::error!("{:?}", remaining.lines().nth(i));
                    }
                }

                Ok(Self { lines })
            }
            Err(Error { input, code }) => Err(Error {
                input: input.to_string(),
                code,
            }),
        }
    }

    pub fn lines(&'a self) -> impl Iterator<Item = &'a Line<'a>> {
        self.lines
            .iter()
            .filter(|line| matches!(line, Line::Tag { .. } | Line::Uri(_)))
    }
}

#[derive(Debug)]
pub struct HexSequence<'a>(&'a str);

impl<'a> HexSequence<'a> {
    pub fn new(data: &'a str) -> Self {
        Self(data)
    }

    pub fn bytes(&self) -> Result<Vec<u8>, hex::FromHexError> {
        hex::decode(self.0)
    }
}

#[derive(Debug)]
pub enum Line<'a> {
    Tag {
        name: &'a str,
        args: Option<TagArgs<'a>>,
    },
    Uri(&'a str),
}

pub type Attributes<'a> = Vec<Attribute<'a>>;

#[derive(Debug)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub value: AttributeValue<'a>,
}

#[derive(Debug)]
pub enum AttributeValue<'a> {
    Integer(u64),
    Hex(HexSequence<'a>),
    Float(f64),
    String(&'a str),
    Keyword(&'a str),
    Resolution { width: u64, height: u64 },
}

#[derive(Debug)]
pub enum TagArgs<'a> {
    Attributes(Attributes<'a>),
    Integer(u64),
    String(&'a str),
}