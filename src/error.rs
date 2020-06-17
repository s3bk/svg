use roxmltree::{Error as XmlError};
use svgtypes::Error as SvgError;
use std::num::ParseFloatError;

#[derive(Debug)]
pub enum Error<'a> {
    Xml(XmlError),
    Svg(SvgError),
    TooShort,
    Unimplemented(&'a str),
    InvalidAttributeValue(&'a str),
    MissingAttribute(&'static str),
    ParseFloat(ParseFloatError),
}
impl<'a> From<XmlError> for Error<'a> {
    fn from(e: XmlError) -> Self {
        Error::Xml(e)
    }
}
impl<'a> From<SvgError> for Error<'a> {
    fn from(e: SvgError) -> Self {
        Error::Svg(e)
    }
}
impl<'a> From<ParseFloatError> for Error<'a> {
    fn from(e: ParseFloatError) -> Self {
        Error::ParseFloat(e)
    }
}