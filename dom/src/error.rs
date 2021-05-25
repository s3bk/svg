use roxmltree::{Error as XmlError};
use svgtypes::Error as SvgError;
use std::num::ParseFloatError;
use std::str::Utf8Error;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum Error {
    Xml(XmlError),
    Svg(SvgError),
    TooShort,
    Unimplemented(String),
    InvalidAttributeValue(String),
    MissingAttribute(String),
    ParseFloat(ParseFloatError),
    Color,
    Paint,
    Utf8(Utf8Error),
    Gzip(IoError),
    NotSvg,
}
impl From<XmlError> for Error {
    fn from(e: XmlError) -> Self {
        Error::Xml(e)
    }
}
impl From<SvgError> for Error {
    fn from(e: SvgError) -> Self {
        dbg!(e);
        panic!();
        Error::Svg(e)
    }
}
impl From<ParseFloatError> for Error {
    fn from(e: ParseFloatError) -> Self {
        Error::ParseFloat(e)
    }
}
impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Self {
        Error::Utf8(e)
    }
}