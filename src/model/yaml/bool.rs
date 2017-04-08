extern crate skimmer;

use txt::Twine;

use model::{ model_issue_rope, EncodedString, Model, Node, Rope, Renderer, Tagged, TaggedValue };
use model::style::CommonStyles;

use std::any::Any;
use std::iter::Iterator;



pub const TAG: &'static str = "tag:yaml.org,2002:bool";
static TWINE_TAG: Twine = Twine::Static (TAG);



#[derive (Clone, Copy)]
pub struct Bool; /*<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    line_feed: Char,
    carriage_return: Char,
    space: Char,
    tab_h: Char,

    letter_t: Char,
    letter_r: Char,
    letter_u: Char,
    letter_e: Char,

    letter_t_t: Char,
    letter_t_r: Char,
    letter_t_u: Char,
    letter_t_e: Char,

    letter_f: Char,
    letter_a: Char,
    letter_l: Char,
    letter_s: Char,

    letter_t_f: Char,
    letter_t_a: Char,
    letter_t_l: Char,
    letter_t_s: Char,

    letter_n: Char,
    letter_t_n: Char,

    letter_o: Char,
    letter_t_o: Char,

    letter_y: Char,
    letter_t_y: Char,

    s_quote: Char,
    d_quote: Char,

    encoding: Encoding,

    _dchr: PhantomData<DoubleChar>
}
*/



impl Bool {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }

/*
    pub fn new (cset: &CharSet<Char, DoubleChar>) -> Bool<Char, DoubleChar> {
        Bool {
            encoding: cset.encoding,

            line_feed: cset.line_feed,
            carriage_return: cset.carriage_return,
            space: cset.space,
            tab_h: cset.tab_h,

            letter_t: cset.letter_t,
            letter_r: cset.letter_r,
            letter_u: cset.letter_u,
            letter_e: cset.letter_e,

            letter_t_t: cset.letter_t_t,
            letter_t_r: cset.letter_t_r,
            letter_t_u: cset.letter_t_u,
            letter_t_e: cset.letter_t_e,

            letter_f: cset.letter_f,
            letter_a: cset.letter_a,
            letter_l: cset.letter_l,
            letter_s: cset.letter_s,

            letter_t_f: cset.letter_t_f,
            letter_t_a: cset.letter_t_a,
            letter_t_l: cset.letter_t_l,
            letter_t_s: cset.letter_t_s,

            letter_n: cset.letter_n,
            letter_t_n: cset.letter_t_n,

            letter_o: cset.letter_o,
            letter_t_o: cset.letter_t_o,

            letter_y: cset.letter_y,
            letter_t_y: cset.letter_t_y,

            s_quote: cset.apostrophe,
            d_quote: cset.quotation,

            _dchr: PhantomData
        }
    }
*/


    fn base_decode (&self, explicit: bool, value: &[u8], yaml_11: bool) -> Result<bool, ()> {
        let mut found_val: u8 = 0;
        let mut quote_state: u8 = 0; // 1 - single, 2 - double

        let mut ptr: usize = 0;

        if explicit {
            match value.get (ptr).map (|b| *b) {
                Some (b'\'') => { ptr += 1; quote_state = 1; }
                Some (b'"')  => { ptr += 1; quote_state = 2; }
                _ => ()
            };

            /*
            if self.s_quote.contained_at_start (value) {
                quote_state = 1;
                ptr += self.s_quote.len ();
            } else if self.d_quote.contained_at_start (value) {
                quote_state = 2;
                ptr += self.d_quote.len ();
            }
            */
        }

        let subslice = &value[ptr ..];

        if
            subslice.starts_with ("true".as_bytes ()) ||
            subslice.starts_with ("True".as_bytes ()) ||
            subslice.starts_with ("TRUE".as_bytes ()) { ptr += 4; found_val = 3; }
        else if
            subslice.starts_with ("false".as_bytes ()) ||
            subslice.starts_with ("False".as_bytes ()) ||
            subslice.starts_with ("FALSE".as_bytes ()) { ptr += 5; found_val = 1; }
        else if yaml_11 {
            if
                subslice.starts_with ("on".as_bytes ()) ||
                subslice.starts_with ("On".as_bytes ()) ||
                subslice.starts_with ("ON".as_bytes ()) { ptr += 2; found_val = 3; }
            else if
                subslice.starts_with ("off".as_bytes ()) ||
                subslice.starts_with ("Off".as_bytes ()) ||
                subslice.starts_with ("OFF".as_bytes ()) { ptr += 3; found_val = 1; }
            else if
                subslice.starts_with ("yes".as_bytes ()) ||
                subslice.starts_with ("Yes".as_bytes ()) ||
                subslice.starts_with ("YES".as_bytes ()) { ptr += 3; found_val = 3; }
            else if
                subslice.starts_with ("no".as_bytes ()) ||
                subslice.starts_with ("No".as_bytes ()) ||
                subslice.starts_with ("NO".as_bytes ()) { ptr += 2; found_val = 1; }
            else { match subslice.get (0).map (|b| *b) {
                Some (b'Y') |
                Some (b'y') => { ptr += 1; found_val = 3; }
                Some (b'N') |
                Some (b'n') => { ptr += 1; found_val = 1; }
                _ => ()
            } }
        }

        if found_val == 0 { return Err ( () ) }

        if quote_state > 0 {
            match value.get (ptr).map (|b| *b) {
                Some (b'\'') if quote_state == 1 => { ptr += 1; }
                Some (b'"')  if quote_state == 2 => { ptr += 1; }
                _ => return Err ( () )
            }

            /*
            if quote_state == 1 && self.s_quote.contained_at (value, ptr) {
                ptr += self.s_quote.len ();
            } else if quote_state == 2 && self.d_quote.contained_at (value, ptr) {
                ptr += self.d_quote.len ();
            } else {
                return Err ( () );
            }
            */
        }

        loop {
            match value.get (ptr).map (|b| *b) {
                None => break,

                Some (b' ') |
                Some (b'\n') |
                Some (b'\t') |
                Some (b'\r') => { ptr += 1; }

                _ => return Err ( () )
            }
        }

        Ok (found_val & 2 == 2)
    }
}



impl Model for Bool {
    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

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

        let node = Node::String (EncodedString::from (value.as_bytes ()));

        Ok (model_issue_rope (self, node, issue_tag, alias, tags))
    }


    fn decode (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        Ok ( TaggedValue::from (BoolValue::from (self.base_decode (explicit, value, false) ?)) )
    }


    fn decode11 (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        Ok ( TaggedValue::from (BoolValue::from (self.base_decode (explicit, value, true) ?)) )
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
    fn get_tag (&self) -> &Twine { &TWINE_TAG }

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



#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use model::{ Tagged, TaggedValue, Renderer };
    // use txt::get_charset_utf8;

    use std::iter;



    #[test]
    fn tag () {
        let bool = Bool; // ::new (&get_charset_utf8 ());

        assert_eq! (bool.get_tag (), TAG);
    }



    #[test]
    fn decode11 () {
        let bool = Bool; // ::new (&get_charset_utf8 ());

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

            assert_eq! (prod.get_tag (), &TWINE_TAG);

            let val: &bool = prod.as_any ().downcast_ref::<BoolValue> ().unwrap ().as_ref ();

            assert_eq! (*val, results[i]);
        }


        let decode = bool.decode11 (true, "folso".as_bytes ());
        assert! (decode.is_err ());
    }



    #[test]
    fn decode () {
        let bool = Bool; // ::new (&get_charset_utf8 ());

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

            assert_eq! (prod.get_tag (), &TWINE_TAG);

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
        let renderer = Renderer; // ::new (&get_charset_utf8 ());
        let bool = Bool; // ::new (&get_charset_utf8 ());


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
