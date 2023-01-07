use hls_parser::{Manifest, Node};
use serde::de::{self, Deserialize, EnumAccess, SeqAccess, VariantAccess, Visitor};
use serde::forward_to_deserialize_any;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Syntax,
    TrailingCharacters,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Deserializer<'de> {
    nodes: Vec<Node<'de>>,
    line_enum: bool,
    tag_enum: bool,
    next_index: usize,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        let manifest = Manifest::parse(input).unwrap();
        let nodes = manifest.nodes();
        let next_index = 0;
        let line_enum = false;
        let tag_enum = false;
        Self {
            next_index,
            tag_enum,
            line_enum,
            nodes,
        }
    }

    fn peek(&self) -> Option<&Node> {
        self.nodes.get(self.next_index)
    }

    fn take1(&mut self) -> Option<&Node> {
        let ret = self.nodes.get(self.next_index);
        self.next_index += 1;
        ret
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("Called {:?}", self.peek());

        if self.line_enum {
            self.line_enum = false;
            println!("do line enum");
            return visitor.visit_str("Tag");
        }

        if self.tag_enum {
            self.tag_enum = false;
            if let Some(Node::TagName(s)) = self.peek() {
                println!("do tag name {}", s);
                return visitor.visit_str(s);
            }
        }

        match self.peek().unwrap() {
            Node::ManifestStart => {
                self.take1(); // skip manifest start tag
                visitor.visit_seq(Lines::new(self))
            }
            Node::TagStart => visitor.visit_enum(Line::new(self)),
            Node::TagName(_) => visitor.visit_enum(TagName::new(self)),
            Node::Integer(i) => {
                let i = *i;
                self.take1();
                visitor.visit_u64(i)
            }
            _ => todo!(),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct Lines<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Lines<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de, 'a> SeqAccess<'de> for Lines<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let Some(Node::ManifestEnd) = self.de.peek() {
            Ok(None)
        } else {
            seed.deserialize(&mut *self.de).map(Some)
        }
    }
}

struct Line<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Line<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for Line<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.line_enum = true;
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for Line<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        todo!();
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("tuple variant");
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.de.take1(); // skip over TagStart
        seed.deserialize(self.de)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("struct variant");
    }
}

struct TagName<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> TagName<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for TagName<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.tag_enum = true;
        println!("visit tag enum");
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for TagName<'a, 'de> {
    type Error = Error;

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.de.take1();
        println!("################################newtype variant seed (tag)");
        seed.deserialize(self.de)
    }

    fn unit_variant(self) -> Result<()> {
        println!("Unit variant {:?}", self.de.peek());
        self.de.take1(); // skip over tag start
        Ok(())
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("tuple variant");
        todo!();
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("struct variant");
        todo!();
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    Ok(T::deserialize(&mut deserializer)?)
}
