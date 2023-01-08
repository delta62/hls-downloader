use hls_parser::{AttributeValue, Manifest, Node};
use serde::de::{self, Deserialize, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::forward_to_deserialize_any;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Syntax,
    TrailingCharacters,
    UnexpectedEof,
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

#[derive(Clone, Copy, Debug)]
enum Context {
    Attributes,
    EnumAttribute,
    Manifest,
    Tag,
    TagName,
    IntAttribute,
    StringAttribute,
    Uri,
}

impl Default for Context {
    fn default() -> Self {
        Self::Manifest
    }
}

pub struct Deserializer<'de> {
    nodes: Vec<Node<'de>>,
    context: Context,
    next_index: usize,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        let manifest = Manifest::parse(input).unwrap();
        let nodes = manifest.nodes();
        let next_index = 0;

        Self {
            next_index,
            nodes,
            context: Default::default(),
        }
    }

    fn peek(&self) -> Result<&Node> {
        self.nodes.get(self.next_index).ok_or(Error::UnexpectedEof)
    }

    fn take1(&mut self) -> Result<&Node> {
        let ret = self
            .nodes
            .get(self.next_index)
            .ok_or(Error::UnexpectedEof)?;
        self.next_index += 1;
        Ok(ret)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        log::debug!("[{:?}] {:?}", self.context, self.peek().unwrap());

        match (self.context, self.peek().unwrap()) {
            (Context::Manifest, Node::ManifestStart) => {
                self.take1()?;
                visitor.visit_seq(Lines::new(self))
            }
            (Context::Manifest, Node::TagStart) => visitor.visit_enum(TagLine::new(self)),
            (Context::Tag, Node::TagStart) => {
                self.take1()?;
                visitor.visit_str("Tag")
            }
            (Context::Tag, Node::TagName(_)) => {
                self.context = Context::TagName;
                visitor.visit_enum(TagName::new(self))
            }
            (Context::TagName, Node::TagName(_)) => {
                if let Node::TagName(s) = self.peek().unwrap() {
                    let res = visitor.visit_str(s);
                    self.take1().unwrap();
                    let next = self.peek().unwrap();
                    match next {
                        Node::Integer(_) => self.context = Context::IntAttribute,
                        Node::String(_) => self.context = Context::StringAttribute,
                        Node::AttributesStart => self.context = Context::Attributes,
                        _ => self.context = Context::Manifest,
                    }
                    res
                } else {
                    unreachable!()
                }
            }
            (Context::IntAttribute, Node::Integer(i)) => {
                let res = visitor.visit_u64(*i)?;
                self.context = Context::Manifest;
                self.take1()?;
                Ok(res)
            }
            (Context::StringAttribute, Node::String(s)) => {
                let res = visitor.visit_str(s)?;
                self.context = Context::Manifest;
                self.take1()?;
                Ok(res)
            }
            (Context::EnumAttribute, Node::String(s)) => {
                let res = visitor.visit_str(s)?;
                self.context = Context::Manifest;
                self.take1()?;
                Ok(res)
            }
            (Context::Attributes, Node::AttributesStart) => {
                self.take1()?;
                visitor.visit_map(Attributes::new(self))
            }
            (Context::Attributes, Node::AttributeName(s)) => {
                let res = visitor.visit_str(s)?;
                self.take1()?;
                Ok(res)
            }
            (Context::Attributes, Node::AttributeValue(v)) => {
                let res = match v {
                    AttributeValue::Integer(i) => {
                        let res = visitor.visit_u64(*i)?;
                        self.take1()?;
                        Ok(res)
                    }
                    AttributeValue::String(s) => {
                        let res = visitor.visit_str(s)?;
                        self.take1()?;
                        Ok(res)
                    }
                    AttributeValue::Keyword(_) => visitor.visit_enum(AttrEnum::new(self)),
                    AttributeValue::Hex(_) => {
                        let res = visitor.visit_unit()?;
                        self.take1()?;
                        Ok(res)
                    }
                    _ => todo!("no match for attr"),
                };

                res
            }
            (Context::EnumAttribute, Node::AttributeValue(v)) => {
                if let AttributeValue::Keyword(s) = v {
                    let res = visitor.visit_str(s)?;
                    self.take1()?;
                    self.context = Context::Attributes;
                    Ok(res)
                } else {
                    Err(Error::Message("invalid state".to_string()))
                }
            }
            (Context::Manifest, Node::Uri(_)) => visitor.visit_enum(UriLine::new(self)),
            (Context::Tag, Node::Uri(_)) => {
                let res = visitor.visit_borrowed_str("Uri")?;
                self.context = Context::Uri;
                Ok(res)
            }
            (Context::Uri, Node::Uri(u)) => {
                let res = visitor.visit_str(u)?;
                self.take1()?;
                self.context = Context::Manifest;
                Ok(res)
            }
            _ => todo!("ctx, next pair not implemented"),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek().unwrap() {
            Node::String(_) => {
                self.context = Context::EnumAttribute;
                visitor.visit_enum(AttrEnum::new(self))
            }
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Node::AttributeValue(_) = self.peek()? {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
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
        if let Node::ManifestEnd = self.de.peek()? {
            log::info!("done with seq");
            Ok(None)
        } else {
            log::info!("continuing thru seq");
            seed.deserialize(&mut *self.de).map(Some)
        }
    }
}

struct TagLine<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> TagLine<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for TagLine<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.context = Context::Tag;
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for TagLine<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        todo!();
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("tuple variant");
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
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
        self.de.context = Context::TagName;
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
        seed.deserialize(self.de)
    }

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("tuple variant");
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("struct variant");
    }
}

struct AttrEnum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> AttrEnum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for AttrEnum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.context = Context::EnumAttribute;
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for AttrEnum<'a, 'de> {
    type Error = Error;

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("tuple variant");
    }

    fn struct_variant<V>(self, __fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("struct variant");
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    Ok(T::deserialize(&mut deserializer)?)
}

struct Attributes<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Attributes<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'a, 'de> MapAccess<'de> for Attributes<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.de.peek()? {
            Node::AttributesEnd => {
                self.de.take1()?;
                self.de.context = Context::Manifest;
                Ok(None)
            }
            _ => seed.deserialize(&mut *self.de).map(Some),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct UriLine<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> UriLine<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for UriLine<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        log::info!("uri variant seed");
        self.de.context = Context::Tag;
        //self.de.take1()?;
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for UriLine<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        todo!();
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("tuple variant");
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        log::info!("uri newtype variant seed");
        seed.deserialize(self.de)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("struct variant");
    }
}
