extern crate skimmer;

use model::{self, EncodedString, Model, Node, Renderer, Rope, Tagged, TaggedValue};

use std::any::Any;
use std::borrow::Cow;
use std::iter::Iterator;

pub static TAG: &'static str = "tag:yamlette.org,1:incognitum";

#[derive(Clone, Copy)]
pub struct Incognitum;

impl Incognitum {
    pub fn get_tag() -> Cow<'static, str> {
        Cow::from(TAG)
    }
}

impl Model for Incognitum {
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

    fn is_metamodel(&self) -> bool {
        true
    }

    fn encode(
        &self,
        _renderer: &Renderer,
        value: TaggedValue,
        tags: &mut Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
    ) -> Result<Rope, TaggedValue> {
        let value: IncognitumValue =
            match <TaggedValue as Into<Result<IncognitumValue, TaggedValue>>>::into(value) {
                Ok(value) => value,
                Err(value) => return Err(value),
            };

        let mut rope = Vec::with_capacity(5);

        if let Some(ref t) = value.get_tag() {
            rope.push(model::custom_tag(Cow::from(t.clone()), tags));
            rope.push(Node::Space);
        };

        if let Some(ref a) = value.get_anchor() {
            rope.push(model::model_alias(self, Cow::from(a.clone())));
            rope.push(Node::Space);
        };

        rope.push(Node::String(EncodedString::from(value.get_value().to_string().into_bytes())));

        Ok(Rope::from(rope))
    }

    fn decode(&self, _: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        match String::from_utf8(Vec::from(value)) {
            Ok(s) => Ok(TaggedValue::from(IncognitumValue::new(Cow::from(s)))),
            _ => Err(()),
        }
    }

    fn meta_init(
        &self,
        anchor: Option<String>,
        tag: Option<String>,
        value: &[u8],
    ) -> Result<TaggedValue, ()> {
        let string = match String::from_utf8(Vec::from(value)) {
            Ok(s) => s,
            _ => return Err(()),
        };

        let mut value = IncognitumValue::new(Cow::from(string));

        value = match tag {
            Some(tag) => value.set_tag(Cow::from(tag)),
            None => value,
        };

        value = match anchor {
            Some(anchor) => value.set_anchor(Cow::from(anchor)),
            None => value,
        };

        Ok(TaggedValue::from(value))
    }
}

#[derive(Debug)]
pub struct IncognitumValue {
    tag: Option<Cow<'static, str>>,
    anchor: Option<Cow<'static, str>>,
    value: Cow<'static, str>,
}

impl IncognitumValue {
    pub fn new(value: Cow<'static, str>) -> IncognitumValue {
        IncognitumValue {
            tag: None,
            anchor: None,
            value: value,
        }
    }

    pub fn set_tag(self, tag: Cow<'static, str>) -> IncognitumValue {
        IncognitumValue {
            tag: Some(tag),
            anchor: self.anchor,
            value: self.value,
        }
    }

    pub fn set_anchor(self, anchor: Cow<'static, str>) -> IncognitumValue {
        IncognitumValue {
            tag: self.tag,
            anchor: Some(anchor),
            value: self.value,
        }
    }

    pub fn get_tag(&self) -> &Option<Cow<'static, str>> {
        &self.tag
    }

    pub fn get_anchor(&self) -> &Option<Cow<'static, str>> {
        &self.anchor
    }

    pub fn get_value(&self) -> &Cow<'static, str> {
        &self.value
    }
}

impl Tagged for IncognitumValue {
    fn get_tag(&self) -> Cow<'static, str> {
        Cow::from(TAG)
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut Any {
        self
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
        // let incognitum = IncognitumFactory.build_model (&get_charset_utf8 ());
        let incognitum = Incognitum; // ::new (&get_charset_utf8 ());

        assert_eq!(incognitum.get_tag().as_ref(), TAG);
    }

    #[test]
    fn encode() {
        let renderer = Renderer; // ::new (&get_charset_utf8 ());
                                 // let incognitum = IncognitumFactory.build_model (&get_charset_utf8 ());
        let incognitum = Incognitum; // ::new (&get_charset_utf8 ());

        let ops: &[(
            Option<&'static str>,
            Option<&'static str>,
            &'static str,
            &'static str,
        )] = &[
            (
                None,
                None,
                r#""Hey, this is a string!""#,
                r#""Hey, this is a string!""#,
            ),
            (
                Some("tag:yamlette.org,1:test"),
                None,
                r"Another string in here",
                r"!<tag:yamlette.org,1:test> Another string in here",
            ),
            (
                None,
                Some("anchor1"),
                r"One more string value",
                r"&anchor1 One more string value",
            ),
            (
                Some("tag:yamlette.org,1:test"),
                Some("anchor2"),
                r"Even more strings in here",
                r"!<tag:yamlette.org,1:test> &anchor2 Even more strings in here",
            ),
        ];

        for i in 0..ops.len() {
            let mut ival = IncognitumValue::new(Cow::from(ops[i].2));

            ival = if let Some(tag) = ops[i].0 {
                ival.set_tag(Cow::from(tag))
            } else {
                ival
            };
            ival = if let Some(anc) = ops[i].1 {
                ival.set_anchor(Cow::from(anc))
            } else {
                ival
            };

            if let Ok(rope) =
                incognitum.encode(&renderer, TaggedValue::from(ival), &mut iter::empty())
            {
                let vec = rope.render(&renderer);
                assert_eq!(vec, ops[i].3.to_string().into_bytes().to_vec());
            } else {
                assert!(false)
            }
        }
    }

    #[test]
    fn decode() {
        let incognitum = Incognitum; // ::new (&get_charset_utf8 ());

        let ops = [
            ("Hey, this is a string!", "Hey, this is a string!"),
            (r"'Hey, that\'s the string!'", r"'Hey, that\'s the string!'"),
            (r#""Hey,\n\ that's\tthe\0string\\""#, r#""Hey,\n\ that's\tthe\0string\\""#),
            (r#""This\x0Ais\x09a\x2c\x20test""#, r#""This\x0Ais\x09a\x2c\x20test""#),
            (r#""\u0422\u0435\u0441\u0442\x0a""#, r#""\u0422\u0435\u0441\u0442\x0a""#),
            (r#""\u30c6\u30b9\u30c8\x0a""#, r#""\u30c6\u30b9\u30c8\x0a""#),
            (r#""\U00013000\U00013001\U00013002\U00013003\U00013004\U00013005\U00013006\U00013007""#, r#""\U00013000\U00013001\U00013002\U00013003\U00013004\U00013005\U00013006\U00013007""#)
        ];

        for i in 0..ops.len() {
            if let Ok(tagged) = incognitum.decode(false, ops[i].0.as_bytes()) {
                assert_eq!(tagged.get_tag(), Cow::from(TAG));

                let val = tagged
                    .as_any()
                    .downcast_ref::<IncognitumValue>()
                    .unwrap()
                    .get_value();

                assert_eq!(*val, Cow::from(ops[i].1));
            } else {
                assert!(false)
            }
        }
    }
}
