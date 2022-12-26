extern crate skimmer;

use crate::model::{EncodedString, Model, Node, Renderer, Rope, Tagged, TaggedValue};

use std::any::Any;
use std::borrow::Cow;
use std::iter::Iterator;

pub static TAG: &'static str = "tag:yaml.org,2002:value";

#[derive(Clone, Copy)]
pub struct Value;

impl Value {
    pub fn get_tag() -> Cow<'static, str> {
        Cow::from(TAG)
    }
}

impl Model for Value {
    fn get_tag(&self) -> Cow<'static, str> {
        Self::get_tag()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }

    fn is_decodable(&self) -> bool {
        true
    }

    fn is_encodable(&self) -> bool {
        true
    }

    fn encode(
        &self,
        _renderer: &Renderer,
        value: TaggedValue,
        _tags: &mut dyn Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
    ) -> Result<Rope, TaggedValue> {
        match <TaggedValue as Into<Result<ValueValue, TaggedValue>>>::into(value) {
            Ok(_) => Ok(Rope::from(Node::String(EncodedString::from(
                "=".as_bytes(),
            )))),
            Err(value) => Err(value),
        }
    }

    fn decode(&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        // let vlen = value.len ();

        let mut ptr = 0;
        let mut quote_state = 0; // 1 - single, 2 - double

        if explicit {
            match value.get(ptr).map(|b| *b) {
                Some(b'\'') => {
                    ptr += 1;
                    quote_state = 1;
                }
                Some(b'"') => {
                    ptr += 1;
                    quote_state = 2;
                }
                _ => (),
            };
        }

        match value.get(ptr).map(|b| *b) {
            Some(b'=') => {
                ptr += 1;
            }
            _ => return Err(()),
        }

        if quote_state > 0 {
            match value.get(ptr).map(|b| *b) {
                Some(b'\'') if quote_state == 1 => {
                    ptr += 1;
                }
                Some(b'"') if quote_state == 2 => {
                    ptr += 1;
                }
                _ => return Err(()),
            };
        }

        if value.len() > ptr {
            return Err(());
        }

        Ok(TaggedValue::from(ValueValue))
    }
}

#[derive(Debug)]
pub struct ValueValue;

impl Tagged for ValueValue {
    fn get_tag(&self) -> Cow<'static, str> {
        Cow::from(TAG)
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}

impl AsRef<str> for ValueValue {
    fn as_ref(&self) -> &'static str {
        "="
    }
}

#[cfg(all(test, not(feature = "dev")))]
mod tests {
    use super::*;

    use crate::model::{Renderer, Tagged};

    use std::iter;

    #[test]
    fn tag() {
        let value = Value; // ::new (&get_charset_utf8 ());

        assert_eq!(value.get_tag(), TAG);
    }

    #[test]
    fn encode() {
        let renderer = Renderer; // ::new (&get_charset_utf8 ());
        let value = Value; // ::new (&get_charset_utf8 ());

        if let Ok(rope) = value.encode(&renderer, TaggedValue::from(ValueValue), &mut iter::empty())
        {
            let vec = rope.render(&renderer);
            assert_eq!(vec, vec![b'=']);
        } else {
            assert!(false)
        }
    }

    #[test]
    fn decode() {
        let value = Value; // ::new (&get_charset_utf8 ());

        if let Ok(tagged) = value.decode(true, "=".as_bytes()) {
            assert_eq!(tagged.get_tag(), Cow::from(TAG));

            if let None = tagged.as_any().downcast_ref::<ValueValue>() {
                assert!(false)
            }
        } else {
            assert!(false)
        }

        assert!(value.decode(true, "=:".as_bytes()).is_err());
    }
}
