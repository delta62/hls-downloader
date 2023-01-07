use crate::parser::all_tags;
use nom::{error::Error, Finish};

#[derive(Debug)]
pub enum Node<'a> {
    AttributeName(&'a str),
    AttributesEnd,
    AttributesStart,
    AttributeValue(AttributeValue<'a>),
    Integer(u64),
    ManifestEnd,
    ManifestStart,
    String(&'a str),
    TagEnd,
    TagName(&'a str),
    TagStart,
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

    pub fn lines(&self) -> &[Line] {
        self.lines.as_slice()
    }

    pub fn nodes(self) -> Vec<Node<'a>> {
        let mut ret = vec![Node::ManifestStart];

        // println!("{:#?}", self.lines);

        for line in self.lines {
            match line {
                Line::Tag { name, args } => {
                    ret.push(Node::TagStart);
                    ret.push(Node::TagName(name));

                    match args {
                        Some(TagArgs::Attributes(attrs)) => {
                            ret.push(Node::AttributesStart);
                            for attr in attrs {
                                ret.push(Node::AttributeName(attr.name));
                                ret.push(Node::AttributeValue(attr.value));
                            }
                            ret.push(Node::AttributesEnd);
                        }
                        Some(TagArgs::String(s)) => ret.push(Node::String(s)),
                        Some(TagArgs::Integer(i)) => ret.push(Node::Integer(i)),
                        None => {}
                    }

                    // ret.push(Node::TagEnd);
                }
                Line::Uri(uri) => {
                    ret.push(Node::Uri(uri));
                }
            }
        }

        ret.push(Node::ManifestEnd);

        ret
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
