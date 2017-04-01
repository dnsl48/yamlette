extern crate skimmer;


use txt::Twine;

use model::{ model_issue_rope, EncodedString, Model, Node, Rope, Renderer, Tagged, TaggedValue };
use model::style::CommonStyles;

use std::any::Any;
use std::default::Default;
use std::iter::Iterator;



pub const TAG: &'static str = "tag:yaml.org,2002:null";
static TWINE_TAG: Twine = Twine::Static (TAG);



#[derive (Copy, Clone, Debug)]
pub struct Null; /*<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    s_quote: Char,
    d_quote: Char,

    tilde: Char,

    letter_n: Char,
    letter_u: Char,
    letter_l: Char,

    letter_t_n: Char,
    letter_t_u: Char,
    letter_t_l: Char,

    encoding: Encoding,

    _dchr: PhantomData<DoubleChar>
}
*/



impl Null {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }

/*
    pub fn new (cset: &CharSet<Char, DoubleChar>) -> Null<Char, DoubleChar> {
        Null {
            encoding: cset.encoding,

            tilde: cset.tilde,

            letter_n: cset.letter_n,
            letter_u: cset.letter_u,
            letter_l: cset.letter_l,

            letter_t_n: cset.letter_t_n,
            letter_t_u: cset.letter_t_u,
            letter_t_l: cset.letter_t_l,

            s_quote: cset.apostrophe,
            d_quote: cset.quotation,

            _dchr: PhantomData
        }
    }
*/

    fn read_null (&self, value: &[u8], ptr: usize) -> usize {
        match value.get (ptr).map (|b| *b) {
            Some (b'~') => 1,
            Some (b'n') => if value.starts_with ("null".as_bytes ()) { 4 } else { 0 },
            Some (b'N') => if value.starts_with ("Null".as_bytes ()) || value.starts_with ("NULL".as_bytes ()) { 4 } else { 0 },
            _ => 0
        }

        //      if self.tilde.contained_at (value, ptr) { self.tilde.len () }
        // else if self.letter_n.contained_at (value, ptr) &&
        //         self.letter_u.contained_at (value, ptr + self.letter_n.len ()) &&
        //         self.letter_l.contained_at (value, ptr + self.letter_n.len () + self.letter_u.len ()) &&
        //         self.letter_l.contained_at (value, ptr + self.letter_n.len () + self.letter_u.len () + self.letter_l.len ())
        //     {
        //         self.letter_n.len () + self.letter_u.len () + self.letter_l.len () + self.letter_l.len ()
        //     }
        // else if self.letter_t_n.contained_at (value, ptr) {
        //     if self.letter_u.contained_at (value, ptr + self.letter_t_n.len ()) &&
        //        self.letter_l.contained_at (value, ptr + self.letter_t_n.len () + self.letter_u.len ()) &&
        //        self.letter_l.contained_at (value, ptr + self.letter_t_n.len () + self.letter_u.len () + self.letter_l.len ())
        //     {
        //         self.letter_t_n.len () + self.letter_u.len () + self.letter_l.len () + self.letter_l.len ()
        //     } else
        //     if self.letter_t_u.contained_at (value, ptr + self.letter_t_n.len ()) &&
        //        self.letter_t_l.contained_at (value, ptr + self.letter_t_n.len () + self.letter_t_u.len ()) &&
        //        self.letter_t_l.contained_at (value, ptr + self.letter_t_n.len () + self.letter_t_u.len () + self.letter_t_l.len ())
        //     {
        //         self.letter_t_n.len () + self.letter_t_u.len () + self.letter_t_l.len () + self.letter_t_l.len ()
        //     } else { 0 }
        // } else { 0 }
    }
}



impl Model for Null {
    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }


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

        let node = Node::String (EncodedString::from ("~".as_bytes ()));

        Ok (model_issue_rope (self, node, issue_tag, alias, tags))
    }


    fn decode (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        if value.len () == 0 { return Ok ( TaggedValue::from (NullValue::default ()) ) }

        let mut ptr: usize = 0;
        let mut quote_state: u8 = 0;

        if explicit {
            match value.get (ptr).map (|b| *b) {
                Some (b'\'') => { ptr += 1; quote_state = 1; }
                Some (b'"')  => { ptr += 1; quote_state = 2; }
                _ => ()
            };

            /*
            if self.s_quote.contained_at_start (value) {
                ptr += self.s_quote.len ();
                quote_state = 1;
            } else if self.d_quote.contained_at_start (value) {
                ptr += self.d_quote.len ();
                quote_state = 2;
            }
            */
        }

        let maybe_null = self.read_null (value, ptr);
        if maybe_null > 0 {
            ptr += maybe_null;

            if quote_state > 0 {
                match value.get (ptr).map (|b| *b) {
                    Some (b'\'') if quote_state == 1 => (),
                    Some (b'"')  if quote_state == 2 => (),
                    _ => return Err ( () )
                };
                /*
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
                */
            }

            return Ok ( TaggedValue::from (NullValue::default ()) )
        }


        if quote_state > 0 {
            match value.get (ptr).map (|b| *b) {
                Some (b'\'') if quote_state == 1 => Ok ( TaggedValue::from (NullValue::default ()) ),
                Some (b'"')  if quote_state == 2 => Ok ( TaggedValue::from (NullValue::default ()) ),
                _ => Err ( () )
            }

            /*
            if quote_state == 1 && ptr == self.s_quote.len () {
                if self.s_quote.contained_at (value, ptr) {
                    return Ok ( TaggedValue::from (NullValue::default ()) )
                }
            } else if quote_state == 2 && ptr == self.d_quote.len () {
                if self.d_quote.contained_at (value, ptr) {
                    return Ok ( TaggedValue::from (NullValue::default ()) )
                }
            }
            */
        } else { Err ( () ) }
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
    fn get_tag (&self) -> &Twine { &TWINE_TAG }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



impl AsRef<str> for NullValue {
    fn as_ref (&self) -> &'static str { "~" }
}



#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use model::{ Tagged, Renderer };
    // use txt::get_charset_utf8;

    use std::iter;



    #[test]
    fn tag () {
        let null = Null; // ::new (&get_charset_utf8 ());

        assert_eq! (null.get_tag (), TAG);
    }



    #[test]
    fn encode () {
        let renderer = Renderer; // ::new (&get_charset_utf8 ());
        let null = Null; // ::new (&get_charset_utf8 ());

        if let Ok (rope) = null.encode (&renderer, TaggedValue::from (NullValue::default ()), &mut iter::empty ()) {
            let encode = rope.render (&renderer);
            assert_eq! (encode, "~".as_bytes ());
        } else { assert! (false) }
    }



    #[test]
    fn decode () {
        let null = Null; // ::new (&get_charset_utf8 ());


        let options = ["", "~", "null", "Null", "NULL"];


        for i in 0 .. options.len () {
            if let Ok (tagged) = null.decode (true, options[i].as_bytes ()) {
                assert_eq! (tagged.get_tag (), &TWINE_TAG);

                if let None = tagged.as_any ().downcast_ref::<NullValue> () { assert! (false) }
            } else { assert! (false) }
        }


        let decode = null.decode (true, "nil".as_bytes ());
        assert! (decode.is_err ());
    }
}
