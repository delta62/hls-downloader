use crate::error::{Error, Result};
use crate::models::{AttributeValue, Manifest, Node};
use serde::de::{self, Deserialize, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::{self, forward_to_deserialize_any};

#[derive(Clone, Copy, Debug)]
enum Context {
    AttributeName,
    Attributes,
    EnumAttribute,
    FloatAttribute,
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
    pub fn from_str(input: &'de str) -> Result<Self> {
        let manifest = Manifest::parse(input).map_err(|_| Error::Syntax)?;
        let nodes = manifest.nodes();
        let next_index = 0;

        Ok(Self {
            next_index,
            nodes,
            context: Default::default(),
        })
    }

    fn peek(&self) -> Result<&Node> {
        self.nodes.get(self.next_index).ok_or(Error::UnexpectedEof)
    }

    fn next(&mut self) -> Result<()> {
        log::debug!(" --- next --- ");
        self.nodes
            .get(self.next_index)
            .ok_or(Error::UnexpectedEof)?;
        self.next_index += 1;
        Ok(())
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        log::debug!("[{:?}] {:?}", self.context, self.peek()?);

        match (self.context, self.peek()?) {
            (Context::Manifest, Node::TagStart) => visitor.visit_enum(TagLine::new(self)),
            (Context::Tag, Node::TagStart) => {
                self.next()?;
                visitor.visit_borrowed_str("Tag")
            }
            (Context::Tag, Node::TagName(_)) => {
                self.context = Context::TagName;
                visitor.visit_enum(TagName::new(self))
            }
            (Context::TagName, Node::TagName(s)) => {
                let res = visitor.visit_str(s)?;
                self.next()?;
                match self.peek()? {
                    Node::Integer(_) => self.context = Context::IntAttribute,
                    Node::Float(_) => self.context = Context::FloatAttribute,
                    Node::String(_) => self.context = Context::StringAttribute,
                    Node::AttributesStart => self.context = Context::Attributes,
                    _ => self.context = Context::Manifest,
                }
                Ok(res)
            }
            (Context::IntAttribute, Node::Integer(i)) => {
                let res = visitor.visit_u64(*i)?;
                self.context = Context::Manifest;
                self.next()?;
                Ok(res)
            }
            (Context::FloatAttribute, Node::Float(f)) => {
                let res = visitor.visit_f64(*f)?;
                self.context = Context::Manifest;
                self.next()?;
                Ok(res)
            }
            (Context::StringAttribute, Node::String(s)) => {
                let res = visitor.visit_str(s)?;
                self.context = Context::Manifest;
                self.next()?;
                Ok(res)
            }
            (Context::EnumAttribute, Node::String(s)) => {
                let res = visitor.visit_str(s)?;
                self.context = Context::Manifest;
                self.next()?;
                Ok(res)
            }
            (Context::AttributeName, Node::AttributeName(s)) => {
                let res = visitor.visit_str(s)?;
                self.next()?;
                Ok(res)
            }
            (Context::Attributes, Node::AttributesStart) => {
                self.next()?;
                visitor.visit_map(Attributes::new(self))
            }
            (Context::Attributes, Node::AttributeValue(v)) => match v {
                AttributeValue::Integer(i) => {
                    let res = visitor.visit_u64(*i)?;
                    self.next()?;
                    Ok(res)
                }
                AttributeValue::String(s) => {
                    let res = visitor.visit_str(s)?;
                    self.next()?;
                    Ok(res)
                }
                AttributeValue::Keyword(k) => match *k {
                    "YES" => {
                        self.next()?;
                        visitor.visit_bool(true)
                    }
                    "NO" => {
                        self.next()?;
                        visitor.visit_bool(false)
                    }
                    s => {
                        let res = visitor.visit_str(s);
                        self.next()?;
                        res
                    }
                },
                AttributeValue::Hex(s) => {
                    let bytes = s.bytes().map_err(|_| Error::InvalidHex)?;
                    let res = visitor.visit_byte_buf(bytes)?;
                    self.next()?;
                    Ok(res)
                }
                AttributeValue::Float(f) => {
                    let res = visitor.visit_f64(*f)?;
                    self.next()?;
                    Ok(res)
                }
                AttributeValue::Resolution { width, height } => {
                    let res = visitor.visit_string(format!("{}x{}", width, height))?;
                    self.next()?;
                    Ok(res)
                }
            },
            (Context::EnumAttribute, Node::AttributeValue(v)) => {
                if let AttributeValue::Keyword(s) = v {
                    let res = visitor.visit_str(s)?;
                    self.next()?;
                    Ok(res)
                } else {
                    unreachable!()
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
                self.next()?;
                self.context = Context::Manifest;
                Ok(res)
            }
            _ => unreachable!(),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek()? {
            Node::AttributeValue(AttributeValue::Integer(i)) => {
                let res = visitor.visit_u64(*i)?;
                self.next()?;
                Ok(res)
            }
            Node::Integer(_) => self.deserialize_any(visitor),
            _ => todo!(),
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
        log::debug!("deserialize_enum {:?}", self.peek()?);
        match self.peek()? {
            Node::String(_) | Node::AttributeValue(AttributeValue::Keyword(_)) => {
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
        log::debug!("option");
        if let Node::AttributeValue(_) = self.peek()? {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Node::ManifestStart = self.peek()? {
            self.next()?;
            visitor.visit_seq(Lines::new(self))
        } else {
            unreachable!("Only manifests support sequential access");
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct tuple
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
            Ok(None)
        } else {
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
        log::debug!("newtype_variant_seed {:?}", self.de.peek()?);
        seed.deserialize(self.de)
    }

    fn unit_variant(self) -> Result<()> {
        log::debug!("unit_variant");
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
    let mut deserializer = Deserializer::from_str(s)?;
    T::deserialize(&mut deserializer)
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
        log::debug!("next_key_seed ({:?})", self.de.peek()?);
        match self.de.peek()? {
            Node::AttributesEnd => {
                self.de.next()?;
                self.de.context = Context::Manifest;
                Ok(None)
            }
            _ => {
                self.de.context = Context::AttributeName;
                let res = seed.deserialize(&mut *self.de).map(Some);
                self.de.context = Context::Attributes;
                res
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        log::debug!("next_value_seed ({:?})", self.de.peek()?);
        let res = seed.deserialize(&mut *self.de)?;
        self.de.context = Context::AttributeName;
        Ok(res)
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
        self.de.context = Context::Tag;
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
        seed.deserialize(self.de)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("struct variant");
    }
}
