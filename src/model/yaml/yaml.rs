extern crate skimmer;

use model::{EncodedString, Model, Node, Renderer, Rope, Tagged, TaggedValue};

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

    /*
    pub fn new (cset: &CharSet<Char, DoubleChar>) -> Yaml<Char, DoubleChar> {
        Yaml {
            encoding: cset.encoding,

            marker_tag: cset.exclamation,
            marker_alias: cset.asterisk,
            marker_anchor: cset.ampersand,

            s_quote: cset.apostrophe,
            d_quote: cset.quotation,

            _dchr: PhantomData
        }
    }
    */
}

impl Model for Yaml {
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

        /*
        let val = if self.marker_tag.contained_at (value, ptr) {
            ptr += self.marker_tag.len ();
            YamlValue::Tag
        } else if self.marker_alias.contained_at (value, ptr) {
            ptr += self.marker_alias.len ();
            YamlValue::Alias
        } else if self.marker_anchor.contained_at (value, ptr) {
            ptr += self.marker_anchor.len ();
            YamlValue::Anchor
        } else { return Err ( () ) };
        */

        if quote_state > 0 {
            match value.get(ptr).map(|b| *b) {
                Some(b'\'') | Some(b'"') => {
                    ptr += 1;
                }
                _ => return Err(()),
            }
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

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_mut_any(&mut self) -> &mut Any {
        self as &mut Any
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

    use model::{Renderer, Tagged};
    // use txt::get_charset_utf8;

    use std::iter;

    #[test]
    fn tag() {
        // let yaml = YamlFactory.build_model (&get_charset_utf8 ());
        let yaml = Yaml; // ::new (&get_charset_utf8 ());

        assert_eq!(yaml.get_tag(), TAG);
    }

    #[test]
    fn encode() {
        let renderer = Renderer; // ::new (&get_charset_utf8 ());
        let yaml = Yaml; // ::new (&get_charset_utf8 ());

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
        let yaml = Yaml; // ::new (&get_charset_utf8 ());

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
