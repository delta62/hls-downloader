use hls_parser::Attributes;

pub trait FromAttrs {
    fn from_attrs(input: &Attributes) -> Self;
}
