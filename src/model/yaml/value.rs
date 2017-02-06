extern crate skimmer;

use self::skimmer::symbol::{ Char, Symbol };


use txt::{ CharSet, Encoding, Twine };

use model::{ EncodedString, Factory, Model, Node, Rope, Renderer, Tagged, TaggedValue };

use std::any::Any;
use std::iter::Iterator;




pub const TAG: &'static str = "tag:yaml.org,2002:value";
static TWINE_TAG: Twine = Twine::Static (TAG);




pub struct Value {
    encoding: Encoding,

    marker: Char,

    s_quote: Char,
    d_quote: Char
}



impl Value {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }


    pub fn new (cset: &CharSet) -> Value {
        Value {
            encoding: cset.encoding,

            marker: cset.equal.clone (),

            s_quote: cset.apostrophe.clone (),
            d_quote: cset.quotation.clone ()
        }
    }
}



impl Model for Value {
    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

    fn get_encoding (&self) -> Encoding { self.encoding }

    fn is_decodable (&self) -> bool { true }

    fn is_encodable (&self) -> bool { true }


    fn encode (&self, _renderer: &Renderer, value: TaggedValue, _tags: &mut Iterator<Item=&(Twine, Twine)>) -> Result<Rope, TaggedValue> {
        match <TaggedValue as Into<Result<ValueValue, TaggedValue>>>::into (value) {
            Ok (_) => Ok ( Rope::from (Node::String (EncodedString::from (self.marker.new_vec ()))) ),
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
    }
}




#[derive (Debug)]
pub struct ValueValue;



impl Tagged for ValueValue {
    fn get_tag (&self) -> &Twine { Value::get_tag () }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



impl AsRef<str> for ValueValue {
    fn as_ref (&self) -> &'static str { "=" }
}




pub struct ValueFactory;



impl Factory for ValueFactory {
    fn get_tag (&self) -> &Twine { Value::get_tag () }

    fn build_model (&self, cset: &CharSet) -> Box<Model> { Box::new (Value::new (cset)) }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use model::{ Factory, Tagged, Renderer };
    use txt::get_charset_utf8;

    use std::iter;



    #[test]
    fn tag () {
        let value = ValueFactory.build_model (&get_charset_utf8 ());

        assert_eq! (value.get_tag (), TAG);
    }



    #[test]
    fn encode () {
        let renderer = Renderer::new (&get_charset_utf8 ());
        let value = Value::new (&get_charset_utf8 ());

        if let Ok (rope) = value.encode (&renderer, TaggedValue::from (ValueValue), &mut iter::empty ()) {
            let vec = rope.render (&renderer);
            assert_eq! (vec, vec! [b'=']);
        } else { assert! (false) }
    }



    #[test]
    fn decode () {
        let value = Value::new (&get_charset_utf8 ());

        if let Ok (tagged) = value.decode (true, "=".as_bytes ()) {
            assert_eq! (tagged.get_tag (), Value::get_tag ());

            if let None = tagged.as_any ().downcast_ref::<ValueValue> () { assert! (false) }
        } else { assert! (false) }

        assert! (value.decode (true, "=:".as_bytes ()).is_err ());
    }
}
