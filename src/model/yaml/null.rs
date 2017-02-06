extern crate skimmer;

use self::skimmer::symbol::{ Char, Word, Rune, Symbol };


use txt::{ CharSet, Encoding, Twine };

use model::{ model_issue_rope, EncodedString, Factory, Model, Node, Rope, Renderer, Tagged, TaggedValue };
use model::style::CommonStyles;

use std::any::Any;
use std::default::Default;
use std::iter::Iterator;




pub const TAG: &'static str = "tag:yaml.org,2002:null";
static TWINE_TAG: Twine = Twine::Static (TAG);




pub struct Null {
    encoding: Encoding,

    tbl: [Rune; 4],

    s_quote: Char,
    d_quote: Char
}



impl Null {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }

    pub fn new (cset: &CharSet) -> Null {
        Null {
            encoding: cset.encoding,

            tbl: [
                Rune::from (cset.tilde.clone ()),
                Rune::from (Word::combine (&[&cset.letter_n, &cset.letter_u, &cset.letter_l, &cset.letter_l])),
                Rune::from (Word::combine (&[&cset.letter_t_n, &cset.letter_u, &cset.letter_l, &cset.letter_l])),
                Rune::from (Word::combine (&[&cset.letter_t_n, &cset.letter_t_u, &cset.letter_t_l, &cset.letter_t_l]))
            ],

            s_quote: cset.apostrophe.clone (),
            d_quote: cset.quotation.clone ()
        }
    }
}



impl Model for Null {
    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

    fn get_encoding (&self) -> Encoding { self.encoding }


    fn is_decodable (&self) -> bool { true }

    fn is_encodable (&self) -> bool { true }


    fn has_default (&self) -> bool { true }

    fn get_default (&self) -> TaggedValue { TaggedValue::from (NullValue::default ()) }


    fn encode (&self, _renderer: &Renderer, value: TaggedValue, tags: &mut Iterator<Item=&(Twine, Twine)>) -> Result<Rope, TaggedValue> {
        let mut val: NullValue = match <TaggedValue as Into<Result<NullValue, TaggedValue>>>::into (value) {
            Ok (value) => value,
            Err (value) => return Err (value)
        };

        let issue_tag = val.issue_tag ();
        let alias = val.take_alias ();

        let node = Node::String (EncodedString::from (self.tbl[0].new_vec ()));

        Ok (model_issue_rope (self, node, issue_tag, alias, tags))
    }


    fn decode (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        if value.len () == 0 { return Ok ( TaggedValue::from (NullValue::default ()) ) }

        let mut ptr = 0;
        let mut quote_state = 0;

        if explicit {
            if self.s_quote.contained_at (value, 0) {
                ptr += self.s_quote.len ();
                quote_state = 1;
            } else if self.d_quote.contained_at (value, 0) {
                ptr += self.d_quote.len ();
                quote_state = 2;
            }
        }


        for i in 0 .. 4 {
            if self.tbl[i].contained_at (value, ptr) {
                ptr += self.tbl[i].len ();

                if quote_state > 0 {
                    if quote_state == 1 {
                        if self.s_quote.contained_at (value, ptr) {
                            // ptr += self.s_quote.len (); ??
                        } else {
                            return Err ( () )
                        }
                    } else if quote_state == 2 {
                        if self.d_quote.contained_at (value, ptr) {
                            // ptr += self.d_quote.len (); ??
                        } else {
                            return Err ( () )
                        }
                    }
                }

                return Ok ( TaggedValue::from (NullValue::default ()) )
            }
        }


        if quote_state > 0 {
            if quote_state == 1 && ptr == self.s_quote.len () {
                if self.s_quote.contained_at (value, ptr) {
                    return Ok ( TaggedValue::from (NullValue::default ()) )
                }
            } else if quote_state == 2 && ptr == self.d_quote.len () {
                if self.d_quote.contained_at (value, ptr) {
                    return Ok ( TaggedValue::from (NullValue::default ()) )
                }
            }
        }


        Err ( () )
    }
}




#[derive (Clone, Debug)]
pub struct NullValue {
    style: u8,
    alias: Option<Twine>
}



impl NullValue {
    pub fn new (styles: CommonStyles, alias: Option<Twine>) -> NullValue { NullValue {
        style: if styles.issue_tag () { 1 } else { 0 },
        alias: alias
    } }

    pub fn take_alias (&mut self) -> Option<Twine> { self.alias.take () }

    pub fn issue_tag (&self) -> bool { self.style & 1 == 1 }

    pub fn set_issue_tag (&mut self, val: bool) { if val { self.style |= 1; } else { self.style &= !1; } }
}


impl Default for NullValue {
    fn default () -> NullValue { NullValue { style: 0, alias: None } }
}


impl Tagged for NullValue {
    fn get_tag (&self) -> &Twine { Null::get_tag () }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



impl AsRef<str> for NullValue {
    fn as_ref (&self) -> &'static str { "~" }
}




pub struct NullFactory;



impl Factory for NullFactory {
    fn get_tag (&self) -> &Twine { Null::get_tag () }

    fn build_model (&self, cset: &CharSet) -> Box<Model> { Box::new (Null::new (cset)) }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use model::{ Factory, Tagged, Renderer };
    use txt::get_charset_utf8;

    use std::iter;



    #[test]
    fn tag () {
        let null = NullFactory.build_model (&get_charset_utf8 ());

        assert_eq! (null.get_tag (), TAG);
    }



    #[test]
    fn encode () {
        let renderer = Renderer::new (&get_charset_utf8 ());
        let null = NullFactory.build_model (&get_charset_utf8 ());

        if let Ok (rope) = null.encode (&renderer, TaggedValue::from (NullValue::default ()), &mut iter::empty ()) {
            let encode = rope.render (&renderer);
            assert_eq! (encode, "~".as_bytes ());
        } else { assert! (false) }
    }



    #[test]
    fn decode () {
        let null = NullFactory.build_model (&get_charset_utf8 ());


        let options = ["", "~", "null", "Null", "NULL"];


        for i in 0 .. options.len () {
            if let Ok (tagged) = null.decode (true, options[i].as_bytes ()) {
                assert_eq! (tagged.get_tag (), Null::get_tag ());

                if let None = tagged.as_any ().downcast_ref::<NullValue> () { assert! (false) }
            } else { assert! (false) }
        }


        let decode = null.decode (true, "nil".as_bytes ());
        assert! (decode.is_err ());
    }
}
