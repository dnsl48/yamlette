extern crate skimmer;

use self::skimmer::symbol::{ CopySymbol, Combo };


use txt::{ CharSet, Encoding, Twine };

use model::{ EncodedString, Model, Node, Rope, Renderer, Tagged, TaggedValue };

use std::any::Any;
use std::iter::Iterator;
use std::marker::PhantomData;



pub const TAG: &'static str = "tag:yaml.org,2002:yaml";
static TWINE_TAG: Twine = Twine::Static (TAG);




pub struct Yaml<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    marker_tag: Char,
    marker_alias: Char,
    marker_anchor: Char,

    s_quote: Char,
    d_quote: Char,

    encoding: Encoding,

    _dchr: PhantomData<DoubleChar>
}



impl<Char, DoubleChar> Yaml<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }


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
}



impl<Char, DoubleChar> Model for Yaml<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    type Char = Char;
    type DoubleChar = DoubleChar;

    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

    fn get_encoding (&self) -> Encoding { self.encoding }

    fn is_decodable (&self) -> bool { true }

    fn is_encodable (&self) -> bool { true }


    fn encode (&self, _renderer: &Renderer<Char, DoubleChar>, value: TaggedValue, _tags: &mut Iterator<Item=&(Twine, Twine)>) -> Result<Rope, TaggedValue> {
        match <TaggedValue as Into<Result<YamlValue, TaggedValue>>>::into (value) {
            Ok (yp) => Ok (Rope::from (Node::String (EncodedString::from (match yp {
                YamlValue::Alias => self.marker_alias.new_vec (),
                YamlValue::Anchor => self.marker_anchor.new_vec (),
                YamlValue::Tag => self.marker_tag.new_vec ()
            })))),
            Err (value) => Err (value)
        }
    }


    fn decode (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let vlen = value.len ();

        let mut ptr = 0;
        let mut quote_state = 0; // 1 - single, 2 - double


        if explicit {
            if self.s_quote.contained_at (value, 0) {
                ptr += self.s_quote.len ();
                quote_state = 1;
            } else if self.d_quote.contained_at (value, 0) {
                ptr += self.d_quote.len ();
                quote_state = 2;
            }
        }


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


        if quote_state > 0 {
            if quote_state == 1 && self.s_quote.contained_at (value, ptr) {
                ptr += self.s_quote.len ();
            } else if quote_state == 2 && self.d_quote.contained_at (value, ptr) {
                ptr += self.d_quote.len ();
            } else { return Err ( () ) }
        }

        if vlen > ptr { return Err ( () ) }

        Ok ( TaggedValue::from (val) )
    }
}



#[derive (Debug)]
pub enum YamlValue {
    Alias,
    Anchor,
    Tag
}



impl Tagged for YamlValue {
    fn get_tag (&self) -> &Twine { &TWINE_TAG }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



impl AsRef<str> for YamlValue {
    fn as_ref (&self) -> &'static str {
        match *self {
            YamlValue::Alias => "*",
            YamlValue::Anchor => "&",
            YamlValue::Tag => "!"
        }
    }
}



/*
pub struct YamlFactory;

impl Factory for YamlFactory {
    fn get_tag (&self) -> &Twine { &TWINE_TAG }

    fn build_model<Char: CopySymbol + 'static, DoubleChar: CopySymbol + Combo + 'static> (&self, cset: &CharSet<Char, DoubleChar>) -> Box<Model<Char=Char, DoubleChar=DoubleChar>> { Box::new (Yaml::new (cset)) }
}
*/



#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use model::{ Tagged, Renderer };
    use txt::get_charset_utf8;

    use std::iter;



    #[test]
    fn tag () {
        // let yaml = YamlFactory.build_model (&get_charset_utf8 ());
        let yaml = Yaml::new (&get_charset_utf8 ());

        assert_eq! (yaml.get_tag (), TAG);
    }



    #[test]
    fn encode () {
        let renderer = Renderer::new (&get_charset_utf8 ());
        let yaml = Yaml::new (&get_charset_utf8 ());

        assert_eq! (yaml.encode (&renderer, TaggedValue::from (YamlValue::Tag), &mut iter::empty ()).ok ().unwrap ().render (&renderer), vec! [b'!']);
        assert_eq! (yaml.encode (&renderer, TaggedValue::from (YamlValue::Alias), &mut iter::empty ()).ok ().unwrap ().render (&renderer), vec! [b'*']);
        assert_eq! (yaml.encode (&renderer, TaggedValue::from (YamlValue::Anchor), &mut iter::empty ()).ok ().unwrap ().render (&renderer), vec! [b'&']);
    }



    #[test]
    fn decode () {
        let yaml = Yaml::new (&get_charset_utf8 ());


        if let Ok (tagged) = yaml.decode (true, "!".as_bytes ()) {
            assert_eq! (tagged.get_tag (), &TWINE_TAG);
            if let Some (&YamlValue::Tag) = tagged.as_any ().downcast_ref::<YamlValue> () {} else { assert! (false) }
        } else { assert! (false) }


        if let Ok (tagged) = yaml.decode (true, "*".as_bytes ()) {
            assert_eq! (tagged.get_tag (), &TWINE_TAG);
            if let Some (&YamlValue::Alias) = tagged.as_any ().downcast_ref::<YamlValue> () {} else { assert! (false) }
        } else { assert! (false) }


        if let Ok (tagged) = yaml.decode (true, "&".as_bytes ()) {
            assert_eq! (tagged.get_tag (), &TWINE_TAG);
            if let Some (&YamlValue::Anchor) = tagged.as_any ().downcast_ref::<YamlValue> () {} else { assert! (false) }
        } else { assert! (false) }


        assert! (yaml.decode (true, "=".as_bytes ()).is_err ());
    }
}
