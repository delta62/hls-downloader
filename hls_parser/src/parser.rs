use crate::models::{Attribute, AttributeValue, Attributes, HexSequence, Line, TagArgs};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till, take_while},
    character::complete::{char, digit0, digit1, hex_digit1, line_ending, one_of},
    combinator::{map, map_res, not, opt, peek, recognize, value},
    multi::{fold_many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

const WHITESPACE: &str = " \t\r\n";

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
    map_res(dec_digit1, |s| s.parse::<u64>())(i)
}

fn float(i: &str) -> IResult<&str, f64> {
    map_res(
        recognize(tuple((opt(char('-')), opt(dec_digit1), char('.'), digit1))),
        |s| s.parse::<f64>(),
    )(i)
}

fn tag_name(i: &str) -> IResult<&str, &str> {
    preceded(
        char('#'),
        preceded(
            tag("EXT"),
            preceded(opt(tag("-X-")), take_while(keyword_char)),
        ),
    )(i)
}

fn hex_sequence(i: &str) -> IResult<&str, &str> {
    preceded(alt((tag("0x"), tag("0X"))), hex_digit1)(i)
}

fn duration_name(i: &str) -> IResult<&str, f64> {
    terminated(
        terminated(float, char(',')),
        take_till(|c| "\r\n".contains(c)),
    )(i)
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

fn resolution(i: &str) -> IResult<&str, AttributeValue> {
    map(
        separated_pair(integer, char('x'), integer),
        |(width, height)| AttributeValue::Resolution { width, height },
    )(i)
}

fn attr_val(i: &str) -> IResult<&str, AttributeValue> {
    alt((
        map(hex_sequence, |s| AttributeValue::Hex(HexSequence::new(s))),
        resolution,
        map(float, AttributeValue::Float),
        map(integer, AttributeValue::Integer),
        map(quoted_string, AttributeValue::String),
        map(keyword1, AttributeValue::Keyword),
    ))(i)
}

fn attr(i: &str) -> IResult<&str, Attribute> {
    map(
        separated_pair(keyword1, char('='), attr_val),
        |(name, value)| Attribute { name, value },
    )(i)
}

fn attrs(i: &str) -> IResult<&str, Attributes> {
    separated_list1(char(','), attr)(i)
}

fn maybe_tag_args(i: &str) -> IResult<&str, Option<TagArgs>> {
    opt(preceded(char(':'), tag_args))(i)
}

fn tag_args(i: &str) -> IResult<&str, TagArgs> {
    alt((
        map(duration_name, TagArgs::Float),
        map(attrs, TagArgs::Attributes),
        map(terminated(integer, peek(line_ending)), TagArgs::Integer),
        map(is_not(WHITESPACE), TagArgs::String),
    ))(i)
}

fn playlist_tag(i: &str) -> IResult<&str, Line> {
    map(
        terminated(pair(tag_name, maybe_tag_args), line_ending),
        |(name, args)| Line::Tag { name, args },
    )(i)
}

fn uri(i: &str) -> IResult<&str, &str> {
    preceded(not(char('#')), terminated(is_not(WHITESPACE), line_ending))(i)
}

fn playlist_line(i: &str) -> IResult<&str, Option<Line>> {
    alt((
        map(line_ending, |_| None),
        map(playlist_tag, Some),
        map(comment, |_| None),
        map(uri, |u| Some(Line::Uri(u))),
    ))(i)
}

pub fn all_tags(i: &str) -> IResult<&str, Vec<Line>> {
    fold_many1(playlist_line, Vec::new, |mut acc, line| {
        if let Some(line) = line {
            acc.push(line);
        }
        acc
    })(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_header_tag() {
        let input = "#EXTM3U";
        assert_eq!(Ok(("", "M3U")), tag_name(input));
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
    fn parses_duration_name() {
        // This is a special case for EXTINF, which has an unusal arg format of <float>,[name]
        assert_eq!(Ok(("", 12.345)), duration_name("12.345,"));
        assert_eq!(Ok(("", 12.345)), duration_name("12.345,SegmentName"));
        assert_eq!(Ok(("", 12.345)), duration_name("12.345,The rain in Spain"));
        // Trailing comma is required
        assert!(duration_name("12.345").is_err());
    }

    #[test]
    fn parses_decimal_integer() {
        assert_eq!(Ok(("", 42)), integer("42"));
        assert_eq!(Ok(("", 0)), integer("0"));
        assert_eq!(Ok(("07", 0)), integer("007"));
        assert_eq!(Ok(("123", 0)), integer("0123"));
        // overflow
        assert!(integer("184467440737095516151").is_err());
        assert!(integer("-1").is_err());
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
