use roxmltree::{Error as XmlError};
use svgtypes::Error as SvgError;

#[derive(Debug)]
pub enum Error<'a> {
    Xml(XmlError),
    Svg(SvgError),
    TooShort,
    Unimplemented(&'a str),
    InvalidAttributeValue(&'a str),
    MissingAttribute(&'static str),
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