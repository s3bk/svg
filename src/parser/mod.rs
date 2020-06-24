pub mod color;
//mod time;

use nom::{
    sequence::{preceded, delimited},
    character::complete::none_of,
    bytes::complete::{tag},
    branch::alt,
    multi::many1_count,
    combinator::{map, map_res, recognize},
    IResult,
};
use crate::prelude::*;

type R<'i, T> = IResult<&'i str, T, ()>;

fn id(i: &str) -> R<&str> {
    recognize(many1_count(none_of(" \")")))(i)
}
#[test]
fn test_id() {
    assert_eq!(id("foobar123"), Ok(("", "foobar123")));
}

fn iri(i: &str) -> R<&str> {
    preceded(tag("#"), id)(i)
}

#[test]
fn test_iri() {
    assert_eq!(iri("#foobar123"), Ok(("", "foobar123")));
}

pub fn func_iri(i: &str) -> R<&str> {
    delimited(tag("url("), iri, tag(")"))(i)
}

#[test]
fn test_func_iri() {
    assert_eq!(func_iri("url(#foobar123)"), Ok(("", "foobar123")));
}

pub fn parse_color(s: &str) -> Result<Color, Error> {
    match color::color(s) {
        Ok((_, color)) => Ok(color),
        Err(e) => {
            debug!("parse_color({:?}): {:?}", s, e);
            Err(Error::InvalidAttributeValue(s.into()))
        }
    }
}
pub fn parse_paint(s: &str) -> Result<Paint, Error> {
    match alt((
        map(tag("none"), |_| Paint::None),
        map(func_iri, |s| Paint::Ref(s.into())),
        map(color::color, Paint::Color),
    ))(s) {
        Ok((_, paint)) => Ok(paint),
        Err(e) => {
            debug!("parse_paint({:?}): {:?}", s, e);
            Err(Error::InvalidAttributeValue(s.into()))
        }
    }
}

#[test]
fn test_paint() {
    assert_eq!(parse_paint("url(#radialGradient862)").unwrap(), Paint::Ref("radialGradient862".into()));
}