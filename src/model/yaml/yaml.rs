extern crate skimmer;

use crate::model::{EncodedString, Model, Node, Renderer, Rope, Tagged, TaggedValue};

use std::any::Any;
use std::borrow::Cow;
use std::iter::Iterator;

pub static TAG: &'static str = "tag:yaml.org,2002:yaml";

#[derive(Clone, Copy)]
pub struct Yaml;

impl Yaml {
    pub fn get_tag() -> Cow<'static, str> {
        Cow::from(TAG)
    }
}

impl Model for Yaml {
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
        match <TaggedValue as Into<Result<YamlValue, TaggedValue>>>::into(value) {
            Ok(yp) => Ok(Rope::from(Node::String(EncodedString::from(match yp {
                YamlValue::Alias => "*".as_bytes(), // self.marker_alias.new_vec (),
                YamlValue::Anchor => "&".as_bytes(), // self.marker_anchor.new_vec (),
                YamlValue::Tag => "!".as_bytes(),   // self.marker_tag.new_vec ()
            })))),
            Err(value) => Err(value),
        }
    }

    fn decode(&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
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
            }
        }

        let val = match value.get(ptr).map(|b| *b) {
            Some(b'*') => {
                ptr += 1;
                YamlValue::Alias
            }
            Some(b'&') => {
                ptr += 1;
                YamlValue::Anchor
            }
            Some(b'!') => {
                ptr += 1;
                YamlValue::Tag
            }
            _ => return Err(()),
        };

        if quote_state > 0 {
            match value.get(ptr).map(|b| *b) {
                Some(b'\'') | Some(b'"') => {
                    ptr += 1;
                }
                _ => return Err(()),
            }
        }

        if value.len() > ptr {
            return Err(());
        }

        Ok(TaggedValue::from(val))
    }
}

#[derive(Debug)]
pub enum YamlValue {
    Alias,
    Anchor,
    Tag,
}

impl Tagged for YamlValue {
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

impl AsRef<str> for YamlValue {
    fn as_ref(&self) -> &'static str {
        match *self {
            YamlValue::Alias => "*",
            YamlValue::Anchor => "&",
            YamlValue::Tag => "!",
        }
    }
}

#[cfg(all(test, not(feature = "dev")))]
mod tests {
    use super::*;

    use crate::model::{Renderer, Tagged};

    use std::iter;

    #[test]
    fn tag() {
        let yaml = Yaml;

        assert_eq!(yaml.get_tag(), TAG);
    }

    #[test]
    fn encode() {
        let renderer = Renderer;
        let yaml = Yaml;

        assert_eq!(
            yaml.encode(
                &renderer,
                TaggedValue::from(YamlValue::Tag),
                &mut iter::empty()
            )
            .ok()
            .unwrap()
            .render(&renderer),
            vec![b'!']
        );
        assert_eq!(
            yaml.encode(
                &renderer,
                TaggedValue::from(YamlValue::Alias),
                &mut iter::empty()
            )
            .ok()
            .unwrap()
            .render(&renderer),
            vec![b'*']
        );
        assert_eq!(
            yaml.encode(
                &renderer,
                TaggedValue::from(YamlValue::Anchor),
                &mut iter::empty()
            )
            .ok()
            .unwrap()
            .render(&renderer),
            vec![b'&']
        );
    }

    #[test]
    fn decode() {
        let yaml = Yaml;

        if let Ok(tagged) = yaml.decode(true, "!".as_bytes()) {
            assert_eq!(tagged.get_tag(), Cow::from(TAG));
            if let Some(&YamlValue::Tag) = tagged.as_any().downcast_ref::<YamlValue>() {
            } else {
                assert!(false)
            }
        } else {
            assert!(false)
        }

        if let Ok(tagged) = yaml.decode(true, "*".as_bytes()) {
            assert_eq!(tagged.get_tag(), Cow::from(TAG));
            if let Some(&YamlValue::Alias) = tagged.as_any().downcast_ref::<YamlValue>() {
            } else {
                assert!(false)
            }
        } else {
            assert!(false)
        }

        if let Ok(tagged) = yaml.decode(true, "&".as_bytes()) {
            assert_eq!(tagged.get_tag(), Cow::from(TAG));
            if let Some(&YamlValue::Anchor) = tagged.as_any().downcast_ref::<YamlValue>() {
            } else {
                assert!(false)
            }
        } else {
            assert!(false)
        }

        assert!(yaml.decode(true, "=".as_bytes()).is_err());
    }
}
