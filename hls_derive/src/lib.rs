use hls_parser::{Manifest, Node};
use serde::de::{
    self, Deserialize, EnumAccess, IntoDeserializer, SeqAccess, VariantAccess, Visitor,
};
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
    next_index: usize,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        let manifest = Manifest::parse(input).unwrap();
        let nodes = manifest.nodes();
        let next_index = 0;
        Self { next_index, nodes }
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
        match self.peek().unwrap() {
            Node::ManifestStart => visitor.visit_seq(Lines::new(self)),
            Node::TagStart => visitor.visit_enum(Line::new(self)),
            // Node::TagStart => visitor.visit_enum("Tag".into_deserializer()),
            Node::TagName(s) => {
                let xyz = visitor.visit_str(s)?;

                Ok(xyz)
            }
            _ => todo!(),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
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
        seed.deserialize(&mut *self.de).map(Some)
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
        println!("tuple variant");
        todo!();
    }

    fn newtype_variant<T>(self) -> Result<T>
    where
        T: Deserialize<'de>,
    {
        println!("newtype variant");
        todo!();
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        println!("newtype variant seed");
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
