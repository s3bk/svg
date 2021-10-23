pub mod color;
//mod time;

use nom::{
    sequence::{preceded, delimited, tuple},
    character::complete::{none_of, space0, space1},
    bytes::complete::{tag},
    branch::alt,
    multi::{many1_count, separated_nonempty_list},
    combinator::{map, map_res, recognize, opt, all_consuming},
    number::complete::float,
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
        map(tag("transparent"), |_| Paint::None),
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

fn list_sep(i: &str) -> IResult<&str, &str, ()> {
    recognize(alt((
        recognize(tuple((tag(","), space0))),
        space1
    )))(i)
}
fn number_list_4_(i: &str) -> IResult<&str, [f32; 4], ()> {
    let (i, a) = float(i)?;
    let (i, _) = list_sep(i)?;
    let (i, b) = float(i)?;
    let (i, _) = list_sep(i)?;
    let (i, c) = float(i)?;
    let (i, _) = list_sep(i)?;
    let (i, d) = float(i)?;
    Ok((i, [a, b, c, d]))
}

pub fn number_list_4(s: &str) -> Result<[f32; 4], Error> {
    match number_list_4_(s) {
        Ok((_, list)) => Ok(list),
        Err(_) => Err(Error::InvalidAttributeValue(s.into()))
    }
}

fn one_or_two_numbers_(i: &str) -> IResult<&str, (f32, Option<f32>), ()> {
    tuple((float, opt(preceded(list_sep, float))))(i)
}

pub fn one_or_two_numbers(s: &str) -> Result<(f32, Option<f32>), Error> {
    match one_or_two_numbers_(s) {
        Ok((_, val)) => Ok(val),
        Err(_) => Err(Error::InvalidAttributeValue(s.into()))
    }
}

fn one_or_three_numbers_(i: &str) -> IResult<&str, (f32, Option<(f32, f32)>), ()> {
    tuple((float, opt(tuple((preceded(list_sep, float), preceded(list_sep, float))))))(i)
}
pub fn one_or_three_numbers(s: &str) -> Result<(f32, Option<(f32, f32)>), Error> {
    match one_or_three_numbers_(s) {
        Ok((_, val)) => Ok(val),
        Err(_) => Err(Error::InvalidAttributeValue(s.into()))
    }
}

fn one_or_many<'a, O>(f: impl Fn(&'a str) -> IResult<&'a str, O, ()> + Copy) -> impl Fn(&'a str) -> IResult<&'a str, OneOrMany<O>, ()> {
    alt((
        map(all_consuming(f), |v| OneOrMany::One(v)),
        map(separated_nonempty_list(list_sep, f), |v| OneOrMany::Many(v))
    ))
}
pub fn one_or_many_f32(s: &str) -> Result<OneOrMany<f32>, Error> {
    match (one_or_many(float))(s) {
        Ok((_, val)) => Ok(val),
        Err(_) => Err(Error::InvalidAttributeValue(s.into()))
    }
}
