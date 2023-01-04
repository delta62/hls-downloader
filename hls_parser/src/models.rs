use crate::parser::all_tags;
use nom::{error::Error, Finish};

#[derive(Debug)]
pub enum Line<'a> {
    Blank,
    Tag { name: &'a str, args: TagArgs<'a> },
    Comment,
    Uri(&'a str),
}

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
pub enum AttributeValue<'a> {
    Integer(u64),
    Hex(&'a str),
    Float(f64),
    String(&'a str),
    Keyword(&'a str),
    Resolution { width: u64, height: u64 },
}

#[derive(Debug)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub value: AttributeValue<'a>,
}

pub type Attributes<'a> = Vec<Attribute<'a>>;

#[derive(Debug)]
pub enum TagArgs<'a> {
    Attributes(Attributes<'a>),
    Integer(u64),
    String(&'a str),
    None,
}
