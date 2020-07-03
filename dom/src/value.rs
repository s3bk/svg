use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct ValueVector {
    pub x: Value<LengthX>,
    pub y: Value<LengthY>
}
impl ValueVector {
    pub fn new(x: Value<LengthX>, y: Value<LengthY>) -> ValueVector {
        ValueVector { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct Value<T> {
    pub value: T,
    pub animations: Vec<Animate<T>>,
}
impl<T> Value<T> {
    pub fn new(value: T) -> Value<T> {
        Value { value, animations: Vec::new() }
    }
}
impl<T> Value<T> where T: Parse + Clone {
    pub fn parse_animate_node(&mut self, node: &Node) -> Result<(), Error> {
        self.animations.push(Animate::parse_animate(node, &self.value)?);
        Ok(())
    }
}
impl<T: Parse> Parse for Value<T> {
    fn parse(s: &str) -> Result<Self, Error> {
        T::parse(s).map(Value::new)
    }
}
impl<T: Parse + Default> Value<T> {
    pub fn parse_or_default(s: Option<&str>) -> Result<Self, Error> {
        Ok(Value::new(s.map(T::parse).transpose()?.unwrap_or_default()))
    }
}
impl<T: Default> Default for Value<T> {
    fn default() -> Value<T> {
        Value::new(T::default())
    }
}