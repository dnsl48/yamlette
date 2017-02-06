use txt::{ CharSet, Twine };
use txt::encoding::{ Encoding, Unicode };

use model::{ EncodedString, Factory, Model, Node, Rope, Renderer, Tagged, TaggedValue };

use std::any::Any;
use std::iter::Iterator;




pub const TAG: &'static str = "tag:yamlette.org,1:literal";
static TWINE_TAG: Twine = Twine::Static (TAG);



pub struct Literal {
    encoding: Encoding
}



impl Literal {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }

    pub fn new (cset: &CharSet) -> Literal { Literal { encoding: cset.encoding } }

    pub fn bytes_to_string (&self, bytes: &[u8]) -> Result<String, ()> { self.encoding.bytes_to_string (bytes) }

    pub fn string_to_bytes (&self, string: String) -> Vec<u8> { self.encoding.string_to_bytes (string) }
}



impl Model for Literal {
    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

    fn get_encoding (&self) -> Encoding { self.encoding }

    fn is_decodable (&self) -> bool { true }

    fn is_encodable (&self) -> bool { true }

    fn has_default (&self) -> bool { true }

    fn get_default (&self) -> TaggedValue { TaggedValue::from (LiteralValue { value: Twine::from ("") }) }


    fn encode (&self, _renderer: &Renderer, value: TaggedValue, _tags: &mut Iterator<Item=&(Twine, Twine)>) -> Result<Rope, TaggedValue> {
        match <TaggedValue as Into<Result<LiteralValue, TaggedValue>>>::into (value) {
            Ok (value) => match value.value {
                Twine::String (s) => Ok (Rope::from (Node::String (EncodedString::from (self.encoding.string_to_bytes (s))))),
                Twine::Static (s) => Ok (Rope::from (Node::String (match self.encoding.str_to_bytes (s) {
                    Ok (s) => EncodedString::from (s),
                    Err (s) => EncodedString::from (s)
                })))
            },
            Err (value) => Err (value)
        }
    }


    fn decode (&self, _: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let string = try! (self.bytes_to_string (value));
        Ok ( TaggedValue::from (LiteralValue::from (string)) )
    }
}




#[derive (Debug)]
pub struct LiteralValue { value: Twine }



impl Tagged for LiteralValue {
    fn get_tag (&self) -> &Twine { Literal::get_tag () }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



impl From<String> for LiteralValue {
    fn from (value: String) -> LiteralValue { LiteralValue { value: Twine::from (value) } }
}



impl From<&'static str> for LiteralValue {
    fn from (value: &'static str) -> LiteralValue { LiteralValue { value: Twine::from (value) } }
}



impl AsRef<str> for LiteralValue {
    fn as_ref (&self) -> &str { self.value.as_ref () }
}




pub struct LiteralFactory;



impl Factory for LiteralFactory {
    fn get_tag (&self) -> &Twine { Literal::get_tag () }

    fn build_model (&self, cset: &CharSet) -> Box<Model> { Box::new (Literal::new (cset)) }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use model::{ Factory, Tagged, Renderer };
    use txt::get_charset_utf8;

    use std::iter;



    #[test]
    fn tag () {
        let literal = LiteralFactory.build_model (&get_charset_utf8 ());

        assert_eq! (literal.get_tag ().as_ref (), TAG);
    }



    #[test]
    fn encode () {
        let renderer = Renderer::new (&get_charset_utf8 ());
        let literal = Literal::new (&get_charset_utf8 ());

        let ops = [
            (r#""Hey, this is a string!""#, r#""Hey, this is a string!""#),
            (r#""Hey,\nthis is\tanother\" one""#, r#""Hey,\nthis is\tanother\" one""#)
        ];


        for i in 0 .. ops.len () {
            if let Ok (rope) = literal.encode (&renderer, TaggedValue::from (LiteralValue::from (ops[i].0.to_string ())), &mut iter::empty ()) {
                let vec = rope.render (&renderer);
                assert_eq! (vec, ops[i].1.to_string ().into_bytes ().to_vec ());
            } else { assert! (false) }

            if let Ok (rope) = literal.encode (&renderer, TaggedValue::from (LiteralValue::from (ops[i].0)), &mut iter::empty ()) {
                let vec = rope.render (&renderer);
                assert_eq! (vec, ops[i].1.to_string ().into_bytes ().to_vec ());
            } else { assert! (false) }
        }
    }



    #[test]
    fn decode () {
        let literal = Literal::new (&get_charset_utf8 ());

        let ops = [
            ("Hey, this is a string!", "Hey, this is a string!"),
            (r"'Hey, that\'s the string!'", r"'Hey, that\'s the string!'"),
            (r#""Hey,\n\ that's\tthe\0string\\""#, r#""Hey,\n\ that's\tthe\0string\\""#),
            (r#""This\x0Ais\x09a\x2c\x20test""#, r#""This\x0Ais\x09a\x2c\x20test""#),
            (r#""\u0422\u0435\u0441\u0442\x0a""#, r#""\u0422\u0435\u0441\u0442\x0a""#),
            (r#""\u30c6\u30b9\u30c8\x0a""#, r#""\u30c6\u30b9\u30c8\x0a""#),
            (r#""\U00013000\U00013001\U00013002\U00013003\U00013004\U00013005\U00013006\U00013007""#, r#""\U00013000\U00013001\U00013002\U00013003\U00013004\U00013005\U00013006\U00013007""#)
        ];


        for i in 0 .. ops.len () {
            if let Ok (value) = literal.decode (false, ops[i].0.as_bytes ()) {
                assert_eq! (value.get_tag (), Literal::get_tag ());

                let val: &str = value.as_any ().downcast_ref::<LiteralValue> ().unwrap ().as_ref ();

                assert_eq! (val, ops[i].1);
            } else { assert! (false) }
        }
    }
}
