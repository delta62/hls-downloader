use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till, take_while},
    character::complete::{char, digit0, digit1, hex_digit1, line_ending, one_of},
    combinator::{map, not, opt, peek, recognize, success, value},
    error::Error,
    multi::{many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Finish, IResult,
};

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
    name: &'a str,
    value: AttributeValue<'a>,
}

pub type Attributes<'a> = Vec<Attribute<'a>>;

#[derive(Debug)]
pub enum TagArgs<'a> {
    Attributes(Attributes<'a>),
    Integer(u64),
    String(&'a str),
    None,
}

#[derive(Debug)]
pub struct Tag<'a> {
    name: &'a str,
    args: TagArgs<'a>,
}

fn keyword_start(i: &str) -> IResult<&str, char> {
    one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ")(i)
}

fn keyword_char(c: char) -> bool {
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ-0123456789".contains(c)
}

fn keyword1(i: &str) -> IResult<&str, &str> {
    recognize(pair(keyword_start, take_while(keyword_char)))(i)
}

fn is_non_string(c: char) -> bool {
    "\"\r\n".contains(c)
}

fn quoted_string(i: &str) -> IResult<&str, &str> {
    delimited(char('"'), take_till(is_non_string), char('"'))(i)
}

fn dec_digit1(i: &str) -> IResult<&str, &str> {
    alt((tag("0"), recognize(pair(one_of("123456789"), digit0))))(i)
}

fn integer(i: &str) -> IResult<&str, u64> {
    dec_digit1(i).map(|(i, s)| (i, s.parse::<u64>().unwrap()))
}

fn float(i: &str) -> IResult<&str, f64> {
    recognize(tuple((opt(char('-')), opt(dec_digit1), char('.'), digit1)))(i)
        .map(|(i, s)| (i, s.parse::<f64>().unwrap()))
}

fn tag_name(i: &str) -> IResult<&str, &str> {
    preceded(char('#'), keyword1)(i)
}

fn hex_sequence(i: &str) -> IResult<&str, &str> {
    preceded(alt((tag("0x"), tag("0X"))), hex_digit1)(i)
}

fn comment(i: &str) -> IResult<&str, ()> {
    value(
        (),
        tuple((
            char('#'),
            not(tag("EXT")),
            take_till(|c| "\r\n".contains(c)),
            line_ending,
        )),
    )(i)
}

fn enum_string(i: &str) -> IResult<&str, &str> {
    keyword1(i)
}

fn resolution(i: &str) -> IResult<&str, AttributeValue> {
    map(
        tuple((integer, char('x'), integer)),
        |(width, _, height)| AttributeValue::Resolution { width, height },
    )(i)
}

fn attr_val(i: &str) -> IResult<&str, AttributeValue> {
    alt((
        map(hex_sequence, AttributeValue::Hex),
        resolution,
        map(float, AttributeValue::Float),
        map(integer, AttributeValue::Integer),
        map(quoted_string, AttributeValue::String),
        map(enum_string, AttributeValue::Keyword),
    ))(i)
}

fn attr(i: &str) -> IResult<&str, Attribute> {
    separated_pair(keyword1, char('='), attr_val)(i)
        .map(|(i, (name, value))| (i, Attribute { name, value }))
}

fn attrs(i: &str) -> IResult<&str, Attributes> {
    separated_list1(char(','), attr)(i)
}

fn tag_args(i: &str) -> IResult<&str, TagArgs> {
    alt((
        map(preceded(char(':'), attrs), TagArgs::Attributes),
        map(
            tuple((char(':'), integer, peek(line_ending))),
            |(_, i, _)| TagArgs::Integer(i),
        ),
        map(preceded(char(':'), is_not("\r\n")), TagArgs::String),
        map(success(()), |()| TagArgs::None),
    ))(i)
}

fn playlist_tag(i: &str) -> IResult<&str, Line> {
    map(
        terminated(pair(tag_name, tag_args), line_ending),
        |(name, args)| Line::Tag { name, args },
    )(i)
}

fn uri(i: &str) -> IResult<&str, &str> {
    map(
        tuple((not(char('#')), is_not(" \t\r\n"), line_ending)),
        |((), uri, _crlf)| uri,
    )(i)
}

fn playlist_line(i: &str) -> IResult<&str, Line> {
    alt((
        map(line_ending, |_| Line::Blank),
        playlist_tag,
        map(comment, |_| Line::Comment),
        map(uri, Line::Uri),
    ))(i)
}

fn all_tags(i: &str) -> IResult<&str, Vec<Line>> {
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
        assert_eq!(Ok(("", ())), comment(input));
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
        assert_eq!(Ok(("", "00")), hex_sequence("0x00"));
        assert_eq!(Ok(("", "42")), hex_sequence("0x42"));
        assert_eq!(Ok(("", "42")), hex_sequence("0X42"));
        assert_eq!(Ok(("", "000102")), hex_sequence("0x000102"));
    }

    #[test]
    fn parses_enum_string() {
        assert_eq!(Ok(("", "BANANA")), enum_string("BANANA"));
        assert_eq!(Ok(("", "AES-128")), enum_string("AES-128"));
        assert!(enum_string("999").is_err());
    }

    #[test]
    fn parses_resolution() {
        assert!(matches!(
            resolution("1024x768").unwrap().1,
            AttributeValue::Resolution {
                width: 1024,
                height: 768
            }
        ));

        assert!(matches!(
            resolution("0x0").unwrap().1,
            AttributeValue::Resolution {
                width: 0,
                height: 0
            }
        ));
    }
}
