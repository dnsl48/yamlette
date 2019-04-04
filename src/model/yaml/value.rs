extern crate skimmer;

use model::{EncodedString, Model, Node, Renderer, Rope, Tagged, TaggedValue};

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

    fn as_any(&self) -> &Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut Any {
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
        _tags: &mut Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
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
            /*
            if self.s_quote.contained_at (value, 0) {
                ptr += self.s_quote.len ();
                quote_state = 1;
            } else if self.d_quote.contained_at (value, 0) {
                ptr += self.d_quote.len ();
                quote_state = 2;
            }
            */
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
            /*
                        if quote_state == 1 && self.s_quote.contained_at (value, ptr) {
                            ptr += self.s_quote.len ();
                        } else if quote_state == 2 && self.d_quote.contained_at (value, ptr) {
                            ptr += self.d_quote.len ();
                        } else { return Err ( () ) }
            */
        }

        if value.len() > ptr {
            return Err(());
        }

        Ok(TaggedValue::from(ValueValue))

        /*
                if self.marker.contained_at (value, ptr) {
                    ptr += self.marker.len ();

                    if quote_state > 0 {
                        if quote_state == 1 && self.s_quote.contained_at (value, ptr) {
                            ptr += self.s_quote.len ();
                        } else if quote_state == 2 && self.d_quote.contained_at (value, ptr) {
                            ptr += self.d_quote.len ();
                        } else { return Err ( () ) }
                    }

                    if vlen > ptr { return Err ( () ) }

                    Ok ( TaggedValue::from (ValueValue) )
                } else { Err ( () ) }
        */
    }
}

#[derive(Debug)]
pub struct ValueValue;

impl Tagged for ValueValue {
    fn get_tag(&self) -> Cow<'static, str> {
        Cow::from(TAG)
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_mut_any(&mut self) -> &mut Any {
        self as &mut Any
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

    use model::{Renderer, Tagged};
    // use txt::get_charset_utf8;

    use std::iter;

    #[test]
    fn tag() {
        // let value = ValueFactory.build_model (&get_charset_utf8 ());
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
