extern crate skimmer;

use self::skimmer::symbol::{ Char, Word, Rune, Symbol };

use txt::{ CharSet, Encoding, Unicode, Twine };

use model::{ model_issue_rope, EncodedString, Factory, Model, Node, Rope, Renderer, Tagged, TaggedValue };
use model::style::CommonStyles;

use std::any::Any;
use std::iter::Iterator;



pub const TAG: &'static str = "tag:yaml.org,2002:bool";
static TWINE_TAG: Twine = Twine::Static (TAG);




pub struct Bool {
    encoding: Encoding,

    true_words: [Rune; 11],
    false_words: [Rune; 11],

    line_feed: Char,
    carriage_return: Char,
    space: Char,
    tab_h: Char,

    s_quote: Char,
    d_quote: Char
}



impl Bool {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }


    pub fn new (cset: &CharSet) -> Bool {
        Bool {
            encoding: cset.encoding,

            true_words: [
                Rune::from (Word::combine (&[&cset.letter_t, &cset.letter_r, &cset.letter_u, &cset.letter_e])),         // true
                Rune::from (Word::combine (&[&cset.letter_t_t, &cset.letter_r, &cset.letter_u, &cset.letter_e])),       // True
                Rune::from (Word::combine (&[&cset.letter_t_t, &cset.letter_t_r, &cset.letter_t_u, &cset.letter_t_e])), // TRUE

                Rune::from (Word::combine (&[&cset.letter_o, &cset.letter_n])),     // on
                Rune::from (Word::combine (&[&cset.letter_t_o, &cset.letter_n])),   // On
                Rune::from (Word::combine (&[&cset.letter_t_o, &cset.letter_t_n])), // ON

                Rune::from (Word::combine (&[&cset.letter_y, &cset.letter_e, &cset.letter_s])),       // yes
                Rune::from (Word::combine (&[&cset.letter_t_y, &cset.letter_e, &cset.letter_s])),     // Yes
                Rune::from (Word::combine (&[&cset.letter_t_y, &cset.letter_t_e, &cset.letter_t_s])), // YES

                Rune::from (cset.letter_y.clone ()),   // y
                Rune::from (cset.letter_t_y.clone ())  // Y
            ],

            false_words: [
                Rune::from (Word::combine (&[&cset.letter_f, &cset.letter_a, &cset.letter_l, &cset.letter_s, &cset.letter_e])),           // false
                Rune::from (Word::combine (&[&cset.letter_t_f, &cset.letter_a, &cset.letter_l, &cset.letter_s, &cset.letter_e])),         // False
                Rune::from (Word::combine (&[&cset.letter_t_f, &cset.letter_t_a, &cset.letter_t_l, &cset.letter_t_s, &cset.letter_t_e])), // FALSE

                Rune::from (Word::combine (&[&cset.letter_o, &cset.letter_f, &cset.letter_f])),       // off
                Rune::from (Word::combine (&[&cset.letter_t_o, &cset.letter_f, &cset.letter_f])),     // Off
                Rune::from (Word::combine (&[&cset.letter_t_o, &cset.letter_t_f, &cset.letter_t_f])), // OFF

                Rune::from (Word::combine (&[&cset.letter_n, &cset.letter_o])),     // no
                Rune::from (Word::combine (&[&cset.letter_t_n, &cset.letter_o])),   // No
                Rune::from (Word::combine (&[&cset.letter_t_n, &cset.letter_t_o])), // NO

                Rune::from (cset.letter_n.clone ()),   // n
                Rune::from (cset.letter_t_n.clone ())  // N
            ],

            line_feed: cset.line_feed.clone (),
            carriage_return: cset.carriage_return.clone (),
            space: cset.space.clone (),
            tab_h: cset.tab_h.clone (),

            s_quote: cset.apostrophe.clone (),
            d_quote: cset.quotation.clone ()
        }
    }


    fn base_decode (&self, explicit: bool, value: &[u8], upto: usize) -> Result<bool, ()> {
        let mut found: bool = false;
        let mut val: bool = false;
        let mut ptr: usize = 0;
        let vlen: usize = value.len ();

        let mut quote_state = 0; // 1 - single, 2 - double

        if explicit {
            if self.s_quote.contained_at (value, 0) {
                quote_state = 1;
                ptr += self.s_quote.len ();
            } else if self.d_quote.contained_at (value, 0) {
                quote_state = 2;
                ptr += self.d_quote.len ();
            }
        }

        for i in 0 .. upto {
            if self.true_words[i].contained_at (value, ptr) {
                found = true;
                val = true;
                ptr += self.true_words[i].len ();
                break;
            } else if self.false_words[i].contained_at (value, ptr) {
                found = true;
                val = false;
                ptr += self.false_words[i].len ();
                break;
            }
        }

        if !found { return Err ( () ) }

        if quote_state > 0 {
            if quote_state == 1 && self.s_quote.contained_at (value, ptr) {
                ptr += self.s_quote.len ();
            } else if quote_state == 2 && self.d_quote.contained_at (value, ptr) {
                ptr += self.d_quote.len ();
            } else {
                return Err ( () );
            }
        }

        loop {
            if ptr >= vlen { break; }

            if self.space.contained_at (value, ptr) {
                ptr += self.space.len ();
                continue;
            }

            if self.tab_h.contained_at (value, ptr) {
                ptr += self.tab_h.len ();
                continue;
            }

            if self.line_feed.contained_at (value, ptr) {
                ptr += self.line_feed.len ();
                continue;
            }

            if self.carriage_return.contained_at (value, ptr) {
                ptr += self.carriage_return.len ();
                continue;
            }

            return Err ( () )
        }

        Ok (val)
    }
}



impl Model for Bool {
    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

    fn get_encoding (&self) -> Encoding { self.encoding }

    fn is_decodable (&self) -> bool { true }

    fn is_encodable (&self) -> bool { true }


    fn encode (&self, _renderer: &Renderer, value: TaggedValue, tags: &mut Iterator<Item=&(Twine, Twine)>) -> Result<Rope, TaggedValue> {
        let mut value = match <TaggedValue as Into<Result<BoolValue, TaggedValue>>>::into (value) {
            Ok (value) => value,
            Err (value) => return Err (value)
        };

        let issue_tag = value.issue_tag ();
        let alias = value.take_alias ();
        let value = value.to_bool ();

        let value = if value { "true" } else { "false" };

        let node = Node::String (match self.get_encoding ().str_to_bytes (value) {
            Ok (s) => EncodedString::from (s),
            Err (s) => EncodedString::from (s)
        });

        Ok (model_issue_rope (self, node, issue_tag, alias, tags))
    }


    fn decode (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let result = try! (self.base_decode (explicit, value, 3));
        Ok ( TaggedValue::from (BoolValue::from (result)) )
    }


    fn decode11 (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let result = try! (self.base_decode (explicit, value, 11));
        Ok ( TaggedValue::from (BoolValue::from (result)) )
    }
}




#[derive (Debug)]
pub struct BoolValue {
    style: u8,
    value: bool,
    alias: Option<Twine>
}



impl BoolValue {
    pub fn new (value: bool, styles: CommonStyles, alias: Option<Twine>) -> BoolValue { BoolValue {
        style: if styles.issue_tag () { 1 } else { 0 },
        value: value,
        alias: alias
    } }

    pub fn to_bool (self) -> bool { self.value }

    pub fn take_alias (&mut self) -> Option<Twine> { self.alias.take () }

    pub fn issue_tag (&self) -> bool { self.style & 1 == 1 }

    pub fn set_issue_tag (&mut self, val: bool) { if val { self.style |= 1; } else { self.style &= !1; } }
}



impl Tagged for BoolValue {
    fn get_tag (&self) -> &Twine { Bool::get_tag () }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



impl From<bool> for BoolValue {
    fn from (val: bool) -> BoolValue { BoolValue::new (val, CommonStyles::default (), None) }
}



impl AsRef<bool> for BoolValue {
    fn as_ref (&self) -> &bool { &self.value }
}



impl AsMut<bool> for BoolValue {
    fn as_mut (&mut self) -> &mut bool { &mut self.value }
}




pub struct BoolFactory;


impl Factory for BoolFactory {
    fn get_tag (&self) -> &Twine { Bool::get_tag () }

    fn build_model (&self, cset: &CharSet) -> Box<Model> { Box::new (Bool::new (cset)) }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use model::{ Tagged, TaggedValue, Factory, Renderer };
    use txt::get_charset_utf8;

    use std::iter;



    #[test]
    fn tag () {
        let bool = BoolFactory.build_model (&get_charset_utf8 ());

        assert_eq! (bool.get_tag (), TAG);
    }



    #[test]
    fn decode11 () {
        let bool = BoolFactory.build_model (&get_charset_utf8 ());

        let options = [
            "y", "Y", "yes", "Yes", "YES",
            "n", "N", "no", "No", "NO",
            "true", "True", "TRUE",
            "false", "False", "FALSE",
            "on", "On", "ON",
            "off", "Off", "OFF"
        ];

        let results = [
            true, true, true, true, true,
            false, false, false, false, false,
            true, true, true,
            false, false, false,
            true, true, true,
            false, false, false
        ];


        for i in 0 .. options.len () {
            let tagged = bool.decode11 (true, options[i].as_bytes ());
            assert! (tagged.is_ok (), format! ("Cannot decode '{}'", options[i]));

            let prod: TaggedValue = tagged.unwrap ();

            assert_eq! (prod.get_tag (), Bool::get_tag ());

            let val: &bool = prod.as_any ().downcast_ref::<BoolValue> ().unwrap ().as_ref ();

            assert_eq! (*val, results[i]);
        }


        let decode = bool.decode11 (true, "folso".as_bytes ());
        assert! (decode.is_err ());
    }



    #[test]
    fn decode () {
        let bool = BoolFactory.build_model (&get_charset_utf8 ());

        let options = [
            "true", "True", "TRUE",
            "false", "False", "FALSE"
        ];

        let results = [
            true, true, true,
            false, false, false,
        ];


        for i in 0 .. options.len () {
            let tagged = bool.decode (true, options[i].as_bytes ());
            assert! (tagged.is_ok ());

            let prod: TaggedValue = tagged.unwrap ();

            assert_eq! (prod.get_tag (), Bool::get_tag ());

            let val: &bool = prod.as_any ().downcast_ref::<BoolValue> ().unwrap ().as_ref ();

            assert_eq! (*val, results[i]);
        }


        let decode = bool.decode (true, "Yes".as_bytes ());
        assert! (decode.is_err ());

        let decode = bool.decode (true, "No".as_bytes ());
        assert! (decode.is_err ());
    }



    #[test]
    fn encode () {
        let renderer = Renderer::new (&get_charset_utf8 ());
        let bool = BoolFactory.build_model (&get_charset_utf8 ());


        if let Ok (rope) = bool.encode (&renderer, TaggedValue::from (BoolValue::from (true)), &mut iter::empty ()) {
            let encode = rope.render (&renderer);
            assert_eq! (encode, "true".as_bytes ());
        } else { assert! (false, "Unexpected result"); }


        if let Ok (rope) = bool.encode (&renderer, TaggedValue::from (BoolValue::from (false)), &mut iter::empty ()) {
            let encode = rope.render (&renderer);
            assert_eq! (encode, "false".as_bytes ());
        } else { assert! (false, "Unexpected result"); }
    }
}
