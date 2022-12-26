extern crate skimmer;

use crate::model::style::CommonStyles;
use crate::model::{
    model_issue_rope, EncodedString, Model, Node, Renderer, Rope, Tagged, TaggedValue,
};

use std::any::Any;
use std::borrow::Cow;
use std::default::Default;
use std::iter::Iterator;

pub static TAG: &'static str = "tag:yaml.org,2002:null";

#[derive(Copy, Clone, Debug)]
pub struct Null;

impl Null {
    pub fn get_tag() -> Cow<'static, str> {
        Cow::from(TAG)
    }

    fn read_null(&self, value: &[u8], ptr: usize) -> usize {
        match value.get(ptr).map(|b| *b) {
            Some(b'~') => 1,
            Some(b'n') => {
                if value[ptr..].starts_with("null".as_bytes()) {
                    4
                } else {
                    0
                }
            }
            Some(b'N') => {
                if value[ptr..].starts_with("Null".as_bytes())
                    || value[ptr..].starts_with("NULL".as_bytes())
                {
                    4
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
}

impl Model for Null {
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

    fn has_default(&self) -> bool {
        true
    }

    fn get_default(&self) -> TaggedValue {
        TaggedValue::from(NullValue::default())
    }

    fn encode(
        &self,
        _renderer: &Renderer,
        value: TaggedValue,
        tags: &mut dyn Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
    ) -> Result<Rope, TaggedValue> {
        let mut val: NullValue =
            match <TaggedValue as Into<Result<NullValue, TaggedValue>>>::into(value) {
                Ok(value) => value,
                Err(value) => return Err(value),
            };

        let issue_tag = val.issue_tag();
        let alias = val.take_alias();

        let node = Node::String(EncodedString::from("~".as_bytes()));

        Ok(model_issue_rope(self, node, issue_tag, alias, tags))
    }

    fn decode(&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        if value.len() == 0 {
            return Ok(TaggedValue::from(NullValue::default()));
        }

        let mut ptr: usize = 0;
        let mut quote_state: u8 = 0;

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

        let maybe_null = self.read_null(value, ptr);

        if maybe_null > 0 {
            ptr += maybe_null;

            if quote_state > 0 {
                match value.get(ptr).map(|b| *b) {
                    Some(b'\'') if quote_state == 1 => (),
                    Some(b'"') if quote_state == 2 => (),
                    _ => return Err(()),
                };
            }

            return Ok(TaggedValue::from(NullValue::default()));
        }

        if quote_state > 0 {
            match value.get(ptr).map(|b| *b) {
                Some(b'\'') if quote_state == 1 => Ok(TaggedValue::from(NullValue::default())),
                Some(b'"') if quote_state == 2 => Ok(TaggedValue::from(NullValue::default())),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct NullValue {
    style: u8,
    alias: Option<Cow<'static, str>>,
}

impl NullValue {
    pub fn new(styles: CommonStyles, alias: Option<Cow<'static, str>>) -> NullValue {
        NullValue {
            style: if styles.issue_tag() { 1 } else { 0 },
            alias,
        }
    }

    pub fn take_alias(&mut self) -> Option<Cow<'static, str>> {
        self.alias.take()
    }

    pub fn issue_tag(&self) -> bool {
        self.style & 1 == 1
    }

    pub fn set_issue_tag(&mut self, val: bool) {
        if val {
            self.style |= 1;
        } else {
            self.style &= !1;
        }
    }
}

impl Default for NullValue {
    fn default() -> NullValue {
        NullValue {
            style: 0,
            alias: None,
        }
    }
}

impl Tagged for NullValue {
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

impl AsRef<str> for NullValue {
    fn as_ref(&self) -> &'static str {
        "~"
    }
}

#[cfg(all(test, not(feature = "dev")))]
mod tests {
    use super::*;

    use crate::model::{Renderer, Tagged};

    use std::iter;

    #[test]
    fn tag() {
        let null = Null;

        assert_eq!(null.get_tag(), TAG);
    }

    #[test]
    fn encode() {
        let renderer = Renderer;
        let null = Null;

        if let Ok(rope) = null.encode(
            &renderer,
            TaggedValue::from(NullValue::default()),
            &mut iter::empty(),
        ) {
            let encode = rope.render(&renderer);
            assert_eq!(encode, "~".as_bytes());
        } else {
            assert!(false)
        }
    }

    #[test]
    fn decode() {
        let null = Null;

        let options = ["", "~", "null", "Null", "NULL"];

        for i in 0..options.len() {
            if let Ok(tagged) = null.decode(true, options[i].as_bytes()) {
                assert_eq!(tagged.get_tag(), Cow::from(TAG));

                if let None = tagged.as_any().downcast_ref::<NullValue>() {
                    assert!(false)
                }
            } else {
                assert!(false)
            }
        }

        let decode = null.decode(true, "nil".as_bytes());
        assert!(decode.is_err());
    }
}
