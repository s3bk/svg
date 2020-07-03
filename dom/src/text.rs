use crate::prelude::*;
use std::sync::Arc;
use crate::parse_node;

#[derive(Clone, Debug)]
pub struct TagText {
    pub id: Option<String>,
    pub items: Vec<Arc<Item>>,
    pub pos: GlyphPos,
    pub attrs: Attrs,
}
impl Tag for TagText {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
    fn children(&self) -> &[Arc<Item>] {
        &self.items
    }
}

impl ParseNode for TagText {
    fn parse_node(node: &Node) -> Result<TagText, Error> {
        parse!(node => {
            var x,
            var y,
            var dx,
            var dy,
            var rotate,
            var id,
            _ => items,
        });

        Ok(TagText {
            pos: GlyphPos { x, y, dx, dy, rotate },
            attrs: Attrs::parse(node)?,
            id,
            items,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GlyphPos {
    pub x: Option<OneOrMany<LengthX>>,
    pub y: Option<OneOrMany<LengthY>>,

    pub dx: Option<OneOrMany<LengthX>>,
    pub dy: Option<OneOrMany<LengthY>>,

    pub rotate: Option<OneOrMany<f32>>,
}

#[derive(Clone, Debug)]
pub struct TagTSpan {
    pub id: Option<String>,
    pub items: Vec<Arc<Item>>,
    pub attrs: Attrs,
    pub pos: GlyphPos,
}
impl Tag for TagTSpan {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }
    fn children(&self) -> &[Arc<Item>] {
        &self.items
    }
}
impl ParseNode for TagTSpan {
    fn parse_node(node: &Node) -> Result<TagTSpan, Error> {
        parse!(node => {
            var x,
            var y,
            var dx,
            var dy,
            var rotate,
            var id,
            _ => items,
        });
        let attrs = Attrs::parse(node)?;

        Ok(TagTSpan {
            attrs,
            id,
            items,
            pos: GlyphPos { x, y, dx, dy, rotate },
        })
    }
}


#[derive(Clone, Debug)]
pub struct TagTRef {
    pub href: Option<String>,
}

impl Tag for TagTRef {
    fn id(&self) -> Option<&str> {
        None
    }
    fn children(&self) -> &[Arc<Item>] {
        &[]
    }
}
impl ParseNode for TagTRef {
    fn parse_node(node: &Node) -> Result<TagTRef, Error> {
        let href = href(node);
        Ok(TagTRef { href })
    }
}

fn one_or_many<'a, T: 'a>(f: impl Fn(Length) -> T + 'a) -> impl Fn(&str) -> Result<OneOrMany<T>, Error> + 'a {
    use svgtypes::LengthListParser;
    move |s| {
        let mut parser = LengthListParser::from(s);
        match (parser.next(), parser.next()) {
            (None, _) => Err(Error::InvalidAttributeValue(s.into())),
            (Some(Ok(a)), None) => Ok(OneOrMany::One(f(a))),
            (Some(Ok(a)), Some(Ok(b))) => {
                let mut values = vec![f(a), f(b)];
                for r in parser {
                    values.push(f(r?));
                }
                Ok(OneOrMany::Many(values))
            }
            (Some(Err(e)), _) | (_, Some(Err(e))) => Err(Error::InvalidAttributeValue(s.into())),
        }
    }
}

impl Parse for OneOrMany<LengthX> {
    fn parse(s: &str) -> Result<OneOrMany<LengthX>, Error> {
        one_or_many(LengthX)(s)
    }
}
impl Parse for OneOrMany<LengthY> {
    fn parse(s: &str) -> Result<OneOrMany<LengthY>, Error> {
        one_or_many(LengthY)(s)
    }
}