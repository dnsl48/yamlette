extern crate skimmer;

use crate::model::{EncodedString, Model, Node, Renderer, Rope, Tagged, TaggedValue};

use std::any::Any;
use std::borrow::Cow;
use std::iter::Iterator;

pub static TAG: &'static str = "tag:yaml.org,2002:merge";

#[derive(Clone, Copy)]
pub struct Merge;

impl Merge {
    pub fn get_tag() -> Cow<'static, str> {
        Cow::from(TAG)
    }
}

impl Model for Merge {
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
        match <TaggedValue as Into<Result<MergeValue, TaggedValue>>>::into(value) {
            Ok(_) => Ok(Rope::from(Node::String(EncodedString::from(
                "<<".as_bytes(),
            )))),
            Err(value) => Err(value),
        }
    }

    fn decode(&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let mut ptr = 0;

        // let vlen = value.len ();

        let mut quote_state = 0;

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
            }
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

        if value[ptr..].starts_with("<<".as_bytes()) {
            ptr += 2;

            if quote_state > 0 {
                match value.get(ptr).map(|b| *b) {
                    Some(b'\'') if quote_state == 1 => {
                        ptr += 1;
                    }
                    Some(b'"') if quote_state == 2 => {
                        ptr += 1;
                    }
                    _ => return Err(()),
                }
            }

            loop {
                match value.get(ptr).map(|b| *b) {
                    None => break,
                    Some(b' ') | Some(b'\n') | Some(b'\t') | Some(b'\r') => {
                        ptr += 1;
                    }
                    _ => return Err(()),
                };
            }

            Ok(TaggedValue::from(MergeValue))
        } else {
            Err(())
        }
    }
}

#[derive(Debug)]
pub struct MergeValue;

impl Tagged for MergeValue {
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

impl AsRef<str> for MergeValue {
    fn as_ref(&self) -> &'static str {
        "<<"
    }
}

#[cfg(all(test, not(feature = "dev")))]
mod tests {
    use super::*;

    use crate::model::{Renderer, Tagged};

    use std::iter;

    #[test]
    fn tag() {
        let merge = Merge;

        assert_eq!(merge.get_tag(), TAG);
    }

    #[test]
    fn encode() {
        let renderer = Renderer;
        let merge = Merge;

        let rope = merge
            .encode(&renderer, TaggedValue::from(MergeValue), &mut iter::empty())
            .ok()
            .unwrap();
        assert_eq!(rope.render(&renderer), vec![b'<', b'<']);
    }

    #[test]
    fn decode() {
        let merge = Merge;

        if let Ok(tagged) = merge.decode(true, "<<".as_bytes()) {
            assert_eq!(tagged.get_tag(), Cow::from(TAG));

            if let None = tagged.as_any().downcast_ref::<MergeValue>() {
                assert!(false)
            }
        } else {
            assert!(false)
        }

        assert!(merge.decode(true, "<<~".as_bytes()).is_err());
    }
}
