use hls_parser::Attributes;

pub trait FromAttrs {
    fn from_attrs<'a>(input: &Attributes<'a>) -> Self;
}
