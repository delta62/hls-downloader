use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till, take_while},
    character::complete::{char, digit0, digit1, hex_digit1, line_ending, one_of},
    combinator::{map, opt, recognize, value},
    error::Error,
    multi::{many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Finish, IResult,
};

#[derive(Debug)]
enum Line<'a> {
    Blank,
    Tag(Tag<'a>),
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
                if remaining.len() > 0 {
                    log::error!("Remaining input:\n{}", remaining);
                }

                Ok(Self { lines })
            }
            Err(Error { input, code }) => Err(Error {
                input: input.to_string(),
                code,
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolution {
    pub width: u64,
    pub height: u64,
}

#[derive(Debug)]
enum AttributeValue<'a> {
    Integer(u64),
    Hex(Vec<u8>),
    Float(f64),
    String(&'a str),
    Keyword(&'a str),
    Resolution(Resolution),
}

#[derive(Debug)]
struct Attribute<'a> {
    name: &'a str,
    value: AttributeValue<'a>,
}

type Attributes<'a> = Vec<Attribute<'a>>;

#[derive(Debug)]
enum Tag<'a> {
    Tag {
        name: &'a str,
        attrs: Option<Attributes<'a>>,
    },
}

fn keyword_start<'a>(i: &'a str) -> IResult<&'a str, char> {
    one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ")(i)
}

fn keyword_char(c: char) -> bool {
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ-0123456789".contains(c)
}

fn keyword1<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    recognize(pair(keyword_start, take_while(keyword_char)))(i)
}

fn is_non_string(c: char) -> bool {
    "\"\r\n".contains(c)
}

fn quoted_string<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    delimited(char('"'), take_till(is_non_string), char('"'))(i)
}

fn dec_digit1<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    alt((tag("0"), recognize(pair(one_of("123456789"), digit0))))(i)
}

fn integer<'a>(i: &'a str) -> IResult<&'a str, u64> {
    dec_digit1(i).map(|(i, s)| (i, s.parse::<u64>().unwrap()))
}

fn float<'a>(i: &'a str) -> IResult<&'a str, f64> {
    recognize(tuple((opt(char('-')), opt(dec_digit1), char('.'), digit1)))(i)
        .map(|(i, s)| (i, s.parse::<f64>().unwrap()))
}

fn tag_name<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    preceded(char('#'), keyword1)(i)
}

fn hex_sequence<'a>(i: &'a str) -> IResult<&'a str, Vec<u8>> {
    preceded(alt((tag("0x"), tag("0X"))), hex_digit1)(i).map(|(i, s)| (i, hex::decode(s).unwrap()))
}

fn comment<'a>(i: &'a str) -> IResult<&'a str, ()> {
    value((), pair(char('#'), take_till(|c| "\r\n".contains(c))))(i)
}

fn enum_string<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    keyword1(i)
}

fn resolution<'a>(i: &'a str) -> IResult<&'a str, Resolution> {
    map(
        tuple((integer, char('x'), integer)),
        |(width, _, height)| Resolution { width, height },
    )(i)
}

fn attr_val<'a>(i: &'a str) -> IResult<&'a str, AttributeValue<'a>> {
    alt((
        map(resolution, AttributeValue::Resolution),
        map(float, AttributeValue::Float),
        map(integer, AttributeValue::Integer),
        map(hex_sequence, AttributeValue::Hex),
        map(quoted_string, AttributeValue::String),
        map(enum_string, AttributeValue::Keyword),
    ))(i)
}

fn attr<'a>(i: &'a str) -> IResult<&'a str, Attribute<'a>> {
    separated_pair(keyword1, char('='), attr_val)(i)
        .map(|(i, (name, value))| (i, Attribute { name, value }))
}

fn attrs<'a>(i: &'a str) -> IResult<&'a str, Attributes<'a>> {
    separated_list1(char(','), attr)(i)
}

fn playlist_tag<'a>(i: &'a str) -> IResult<&'a str, Tag> {
    terminated(pair(tag_name, opt(preceded(char(':'), attrs))), line_ending)(i)
        .map(|(i, (name, attrs))| (i, Tag::Tag { name, attrs }))
}

fn uri<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    terminated(is_not(" \t\r\n"), line_ending)(i)
}

fn playlist_line<'a>(i: &'a str) -> IResult<&'a str, Line<'a>> {
    alt((
        map(line_ending, |t| Line::Blank),
        map(playlist_tag, |t| Line::Tag(t)),
        map(comment, |c| Line::Comment),
        map(uri, |u| Line::Uri(u)),
    ))(i)
}

fn all_tags<'a>(i: &'a str) -> IResult<&'a str, Vec<Line>> {
    many1(playlist_line)(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_header_tag() {
        let input = "#EXTM3U";
        assert_eq!(Ok(("", "EXTM3U")), tag_name(input));
    }

    #[test]
    fn parses_comment() {
        let input = "# EXTM3U\r\n";
        assert_eq!(Ok(("\r\n", ())), comment(input));
    }

    #[test]
    fn parses_quoted_string() {
        assert_eq!(Ok(("", "")), quoted_string(r#""""#));
        assert_eq!(Ok(("", "cool input")), quoted_string(r#""cool input""#));
    }

    #[test]
    fn parses_decimal_integer() {
        assert_eq!(Ok(("", 42)), integer("42"));
        assert_eq!(Ok(("", 0)), integer("0"));
        assert_eq!(Ok(("07", 0)), integer("007"));
        assert_eq!(Ok(("123", 0)), integer("0123"));
        // assert!(integer("184467440737095516151").is_err());
        assert!(integer("").is_err());
    }

    #[test]
    fn parses_float() {
        assert_eq!(Ok(("", 0.42)), float(".42"));
        assert_eq!(Ok(("", 0.0)), float("0.0"));
        assert_eq!(Ok(("", 0.0)), float(".0"));
        assert_eq!(Ok(("", 0.07)), float("0.07"));
        assert_eq!(Ok(("", 1.23)), float("1.23"));
        assert_eq!(Ok(("", -1.23)), float("-1.23"));
        assert_eq!(Ok(("", -0.42)), float("-.42"));
        // assert!(integer("184467440737095516151").is_err());
        assert!(integer("").is_err());
    }

    #[test]
    fn parses_hex_sequence() {
        assert_eq!(Ok(("", vec![0x00])), hex_sequence("0x00"));
        assert_eq!(Ok(("", vec![0x42])), hex_sequence("0x42"));
        assert_eq!(Ok(("", vec![0x42])), hex_sequence("0X42"));
        assert_eq!(Ok(("", vec![0x00, 0x01, 0x02])), hex_sequence("0x000102"));
    }

    #[test]
    fn parses_enum_string() {
        assert_eq!(Ok(("", "BANANA")), enum_string("BANANA"));
        assert_eq!(Ok(("", "AES-128")), enum_string("AES-128"));
        assert!(enum_string("999").is_err());
    }

    #[test]
    fn parses_resolution() {
        assert_eq!(
            Ok((
                "",
                Resolution {
                    width: 1024,
                    height: 768
                }
            )),
            resolution("1024x768")
        );

        assert_eq!(
            Ok((
                "",
                Resolution {
                    width: 0,
                    height: 0
                }
            )),
            resolution("0x0")
        );
    }
}