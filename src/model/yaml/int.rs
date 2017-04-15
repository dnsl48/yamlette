extern crate num;
extern crate skimmer;

use self::num::{ BigInt, BigUint, ToPrimitive };

use model::{ model_issue_rope, EncodedString, Model, Node, Rope, Renderer, Tagged, TaggedValue };
use model::style::CommonStyles;

use std::any::Any;
use std::borrow::Cow;
use std::fmt;
use std::ops::{ AddAssign, MulAssign, Neg };
use std::iter::Iterator;



pub static TAG: &'static str = "tag:yaml.org,2002:int";




#[derive (Clone, Debug, Hash)]
enum Mint {
    I (i64),
    B (Option<BigInt>)
}



impl Mint {
    pub fn new () -> Mint { Mint::I (0) }

    fn is_i (&self) -> bool {
        match *self {
            Mint::I (_) => true,
            _ => false
        }
    }

    fn get_i (&self) -> i64 {
        match *self {
            Mint::I (v) => v,
            _ => unreachable! ()
        }
    }

    fn set_i (&mut self, val: i64) {
        match *self {
            Mint::I (ref mut v) => *v = val,
            _ => unreachable! ()
        }
    }

    fn take_b (&mut self) -> BigInt {
        match *self {
            Mint::B (ref mut v) => v.take ().unwrap (),
            _ => unreachable! ()
        }
    }
}


impl MulAssign<i64> for Mint {
    fn mul_assign (&mut self, val: i64) {
        if self.is_i () {
            if let Some (n) = self.get_i ().checked_mul (val) {
                self.set_i (n);
            } else {
                let mut bi = BigInt::from (self.get_i ());
                bi = bi * BigInt::from (val);
                *self = Mint::B (Some (bi));
            }
        } else {
            let mut bi = self.take_b ();
            bi = bi * BigInt::from (val);
            *self = Mint::B (Some (bi));
        }
    }
}



impl AddAssign<i64> for Mint {
    fn add_assign (&mut self, val: i64) {
        if self.is_i () {
            if let Some (n) = self.get_i ().checked_add (val) {
                self.set_i (n);
            } else {
                let mut bi = BigInt::from (self.get_i ());
                bi = bi + BigInt::from (val);
                *self = Mint::B (Some (bi));
            }
        } else {
            let mut bi = self.take_b ();
            bi = bi + BigInt::from (val);
            *self = Mint::B (Some (bi));
        }
    }
}


impl Neg for Mint {
    type Output = Self;

    fn neg (mut self) -> Self {
        if self.is_i () {
            if let Some (n) = self.get_i ().checked_neg () {
                Mint::I (n)
            } else {
                let bi = BigInt::from (self.get_i ());
                Mint::B (Some (-bi))
            }
        } else {
            let bi = self.take_b ();
            Mint::B (Some (-bi))
        }
    }
}



impl fmt::Display for Mint {
    fn fmt (&self, ftr: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Mint::I (ref v) => write! (ftr, "{}", v),
            Mint::B (ref v) => write! (ftr, "{}", v.as_ref ().unwrap ())
        }
    }
}




#[derive (Clone, Copy)]
pub struct Int; /*<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    digit_0: Char,

    colon: Char,
    minus: Char,
    plus: Char,
    underscore: Char,
    line_feed: Char,
    carriage_return: Char,
    space: Char,
    tab_h: Char,
    letter_b: Char,
    letter_o: Char,
    letter_x: Char,

    s_quote: Char,
    d_quote: Char,

    encoding: Encoding,

    _dchr: PhantomData<DoubleChar>
}
*/



impl Int {
    pub fn get_tag () -> Cow<'static, str> { Cow::from (TAG) }


    fn base_decode (&self, explicit: bool, value: &[u8], sexagesimals: bool, shortocts: bool) -> Result<Mint, ()> {
        // println! ("yoba, {}, explicit: {:?}", unsafe { String::from_utf8_unchecked (Vec::from (value)) }, explicit);
        if !explicit && ! match value.get (0).map (|b| *b) {
            Some (b'+') |
            Some (b'-') |
            Some (b'0' ... b'9') => true,
            _ => false
        } { return Err ( () ) }
        // if !explicit && ! if let Some (b'0' ... b'9') = value.get (0).map (|b| *b) { true } else { false } { return Err ( () ) }
        // if !explicit && !self.encoding.check_is_dec_num (value) { return Err ( () ) }

        const STATE_SIGN: u8 = 1;
        const STATE_SIGN_N: u8 = 3;

        const STATE_NUM: u8 = 4;
        const STATE_BIN: u8 = 4 + 8;
        const STATE_HEX: u8 = 4 + 16;
        const STATE_OCT: u8 = 4 + 32;
        const STATE_DEC: u8 = 4 + 64;
        const STATE_END: u8 = 4 + 128;

        let mut quote_state: u8 = 0; // 1 - single, 2 - double
        let mut state: u8 = 0;

        let mut ptr: usize = 0;
        let mut val = Mint::new ();

        loop {
            match value.get (ptr).map (|b| *b) {
                None => break,

                Some (b'\'') => {
                    ptr += 1;
                    if explicit && quote_state == 0 && state & STATE_END == 0 {
                        quote_state = 1;
                    } else if quote_state == 1 {
                        if ptr == 2 { return Err ( () ) }
                        quote_state = 0;
                        state = state | STATE_END;
                        break
                    } else {
                        state = state | STATE_END;
                        break
                    }
                }

                Some (b'"') => {
                    ptr += 1;
                    if explicit && quote_state == 0 && state & STATE_END == 0 {
                        quote_state = 2;
                    } else if quote_state == 2 {
                        if ptr == 2 { return Err ( () ) }
                        quote_state = 0;
                        state = state | STATE_END;
                        break
                    } else {
                        state = state | STATE_END;
                        break
                    }
                }

                Some (b'-') => {
                    if state & STATE_SIGN == 0 {
                        ptr += 1;
                        state = state | STATE_SIGN_N;
                    } else { break }
                }

                Some (b'+') => {
                    if state & STATE_SIGN == 0 {
                        ptr += 1;
                        state = state | STATE_SIGN;
                    } else { break }
                }

                _ if state & STATE_SIGN == 0 => { state = state | STATE_SIGN; }

                Some (b'0') if state & STATE_NUM == 0 => {
                    ptr += 1;
                    state = state | STATE_NUM;

                    match value.get (ptr).map (|b| *b) {
                        Some (b'b') => { ptr += 1; state = state | STATE_BIN; }
                        Some (b'o') => { ptr += 1; state = state | STATE_OCT; }
                        Some (b'x') => { ptr += 1; state = state | STATE_HEX; }
                        Some (v @ b'0' ... b'9') => {
                            state = state | if (v - b'0') < 8 && shortocts { STATE_OCT } else { STATE_DEC };
                        }
                        _ => { state = state | STATE_END; }
                    };

                    break
                }

                _ if state & STATE_NUM == 0 => { state = state | STATE_DEC; break }

                _ => break
            }
        }

        if state & STATE_BIN == STATE_BIN {
            loop {
                match value.get (ptr).map (|b| *b) {
                    None => break,

                    Some (b'0') => {
                        ptr += 1;
                        val *= 2;
                    }

                    Some (b'1') => {
                        ptr += 1;
                        val *= 2;
                        val += 1;
                    }

                    Some (b'_') => { ptr += 1; }

                    _ => { state = state | STATE_END; break }
                }
            }
        } else if state & STATE_OCT == STATE_OCT {
            loop {
                match value.get (ptr).map (|b| *b) {
                    None => break,

                    Some (v @ b'0' ... b'7') => {
                        ptr += 1;
                        val *= 8;
                        val += (v - b'0') as i64;
                    }

                    Some (b'_') => { ptr += 1; }

                    _ => { state = state | STATE_END; break }
                }
            }
        } else if state & STATE_HEX == STATE_HEX {
            loop {
                match value.get (ptr).map (|b| *b) {
                    None => break,

                    Some (v @ b'0' ... b'9') => {
                        ptr += 1;
                        val *= 16;
                        val += (v - b'0') as i64;
                    }

                    Some (v @ b'a' ... b'f') => {
                        ptr += 1;
                        val *= 16;
                        val += (10 + (v - b'a')) as i64;
                    }

                    Some (v @ b'A' ... b'F') => {
                        ptr += 1;
                        val *= 16;
                        val += (10 + (v - b'A')) as i64;
                    }

                    Some (b'_') => { ptr += 1; }

                    _ => { state = state | STATE_END; break }
                }
            }
        } else if state & STATE_DEC == STATE_DEC {
            loop {
                match value.get (ptr).map (|b| *b) {
                    None => break,

                    Some (v @ b'0' ... b'9') => {
                        ptr += 1;
                        val *= 10;
                        val += (v - b'0') as i64;
                    }

                    Some (b'_') => { ptr += 1; }

                    Some (b':') if sexagesimals => {
                        ptr += 1;

                        let digit: i64 = match value.get (ptr).map (|b| *b) {
                            Some (val @ b'0' ... b'9') => {
                                ptr += 1;
                                (val - b'0') as i64
                            }
                            _ => return Err ( () )
                        };

                        let mut digit2: Option<i64> = None;

                        if digit < 6 {
                            match value.get (ptr).map (|b| *b) {
                                Some (val @ b'0' ... b'9') => {
                                    ptr += 1;
                                    digit2 = Some ((val - b'0') as i64);
                                }
                                _ => ()
                            }
                        }

                        let num: i64 = if digit2.is_some () {
                            digit * 10 + digit2.unwrap ()
                        } else { digit };

                        val *= 60;
                        val += num;

                        if value.len () == ptr {
                            break;
                        } else if let Some (b':') = value.get (ptr).map (|b| *b) {
                            continue;
                        } else {
                            state = state | STATE_END;
                            continue;
                        }
                    }

                    _ => { state = state | STATE_END; break }
                }
            }
        }

        if state & STATE_END == STATE_END {
            if ptr == 0 { return Err ( () ) }
            if quote_state > 0 {
                match value.get (ptr).map (|b| *b) {
                    Some (b'"') if quote_state == 2 => { ptr += 1; }
                    Some (b'\'') if quote_state == 1 => { ptr += 1; }
                    _ => return Err ( () )
                }
            }

            loop {
                match value.get (ptr).map (|b| *b) {
                    None => break,

                    Some (b' ') |
                    Some (b'\n') |
                    Some (b'\t') |
                    Some (b'\r') => { ptr += 1; }

                    _ => return Err ( () )
                };
            }
        }

        if state & STATE_NUM != STATE_NUM { return Err ( () ) }
        if state & STATE_SIGN_N == STATE_SIGN_N { val = -val; };

        Ok (val)
    }
}



impl Model for Int {
    fn get_tag (&self) -> Cow<'static, str> { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }


    fn is_decodable (&self) -> bool { true }

    fn is_encodable (&self) -> bool { true }


    fn encode (&self, _renderer: &Renderer, value: TaggedValue, tags: &mut Iterator<Item=&(Cow<'static, str>, Cow<'static, str>)>) -> Result<Rope, TaggedValue> {
        let mut value = match <TaggedValue as Into<Result<IntValue, TaggedValue>>>::into (value) {
            Ok (value) => value,
            Err (value) => return Err (value)
        };

        let issue_tag = value.issue_tag ();
        let alias = value.take_alias ();
        let value = value.value;

        let value = format! ("{}", value);
        let node = Node::String (EncodedString::from (value.into_bytes ()));
        Ok (model_issue_rope (self, node, issue_tag, alias, tags))
    }


    fn decode (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        Ok ( TaggedValue::from (IntValue::new (self.base_decode (explicit, value, false, false) ?)) )
    }


    fn decode11 (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        Ok ( TaggedValue::from (IntValue::new (self.base_decode (explicit, value, true, true) ?)) )
    }
}




#[derive (Clone, Debug)]
pub struct IntValue {
    style: u8,
    alias: Option<Cow<'static, str>>,
    value: Mint
}



impl IntValue {
    fn new (value: Mint) -> IntValue { IntValue { style: 0, alias: None, value: value } }

    pub fn set_alias (&mut self, alias: Option<Cow<'static, str>>) { self.alias = alias; }

    pub fn take_alias (&mut self) -> Option<Cow<'static, str>> { self.alias.take () }

    pub fn init_common_styles (&mut self, common_styles: CommonStyles) {
        self.set_issue_tag (common_styles.issue_tag ());
    }

    pub fn issue_tag (&self) -> bool { self.style & 1 == 1 }

    pub fn set_issue_tag (&mut self, val: bool) { if val { self.style |= 1; } else { self.style &= !1; } }
}



impl ToPrimitive for IntValue {
    fn to_i64(&self) -> Option<i64> {
        match self.value {
            Mint::I (v) => Some (v),
            _ => None  // there is no point in even trying
        }
    }


    fn to_u64(&self) -> Option<u64> {
        match self.value {
            Mint::I (v) => v.to_u64 (),
            Mint::B (Some (ref v)) => v.to_u64 (),
            _ => unreachable! ()
        }
    }
}



impl Tagged for IntValue {
    fn get_tag (&self) -> Cow<'static, str> { Cow::from (TAG) }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



impl AsRef<Mint> for IntValue {
    fn as_ref (&self) -> &Mint { &self.value }
}



impl Into<BigInt> for IntValue {
    fn into (self) -> BigInt {
        match self.value {
            Mint::I (v) => BigInt::from (v),
            Mint::B (v) => v.unwrap ()
        }
    }
}



impl<'a> Into<Option<&'a BigInt>> for &'a IntValue {
    fn into (self) -> Option<&'a BigInt> {
        match self.value {
            Mint::B (Some (ref v)) => Some (v),
            _ => None
        }
    }
}



impl Into<Result<i64, IntValue>> for IntValue {
    fn into (self) -> Result<i64, IntValue> {
        match self.value {
            Mint::I (v) => Ok (v),
            _ => Err (self)
        }
    }
}



impl Into<Result<i64, BigInt>> for IntValue {
    fn into (self) -> Result<i64, BigInt> {
        match self.value {
            Mint::I (v) => Ok (v),
            Mint::B (v) => Err (v.unwrap ())
        }
    }
}


macro_rules! from_int {
    ( $($t:ty),* ) => {
        $(
        impl From<$t> for IntValue where $t: ToPrimitive + Into<BigInt> {
            fn from (val: $t) -> IntValue {
                match val.to_i64 () {
                    Some (v) => IntValue::new (Mint::I (v)),
                    None => IntValue::new (Mint::B (Some (val.into ())))
                }
            }
        }
        )*
    }
}

from_int! (i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, BigInt, BigUint);



#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;
    use super::num::BigInt;

    use model::{ Tagged, Renderer };
    // use txt::get_charset_utf8;

    use std::iter;
    use std::str::FromStr;



    #[test]
    fn tag () {
        let int = Int; // ::new (&get_charset_utf8 ());

        assert_eq! (int.get_tag (), TAG);
    }



    #[test]
     fn encode () {
        let renderer = Renderer; // ::new (&get_charset_utf8 ());
        let int = Int; // ::new (&get_charset_utf8 ());


        let options = [0b0000_1111, 0o12, 0xA0, 581, -8888];
        let results = ["15", "10", "160", "581", "-8888"];


        for i in 0 .. options.len () {
            if let Ok (rope) = int.encode (&renderer, TaggedValue::from (IntValue::from (options[i])), &mut iter::empty ()) {
                let encoded = rope.render (&renderer);
                assert_eq! (encoded, results[i].to_string ().into_bytes ());
            } else { assert! (false) }
        }
    }



    #[test]
    fn decode () {
        let int = Int; // ::new (&get_charset_utf8 ());


        let options = ["0b0000_1111", "0b1010_0111_0100_1010_1110", "02472256",
                       "0o2472256", "0x_0A_74_AE", "0x_0a_74_ae", "685230",
                       "+685_230", "-685_230", "0b0000_1111  	"];
        let results = [15, 685230, 2472256, 685230, 685230, 685230, 685230, 685230, -685230, 15];


        for i in 0 .. options.len () {
            if let Ok (tagged) = int.decode (false, &options[i].to_string ().into_bytes ()) {
                assert_eq! (tagged.get_tag (), Cow::from (TAG));

                let intval: Result<IntValue, _> = tagged.into ();
                let intval = intval.unwrap ();

                let i64val: Result<i64, BigInt> = intval.into ();
                assert! (i64val.is_ok ());

                let val = i64val.unwrap ();

                assert_eq! (val, results[i] as i64);
            } else { assert! (false, format! ("Could not decode {}", &options[i])) }
        }


        let decoded = int.decode (true, &"190:20:30".to_string ().into_bytes ());
        assert! (decoded.is_err ());
    }


    #[test]
    fn decode11 () {
        let int = Int; // ::new (&get_charset_utf8 ());


        let options = ["0b0000_1111", "0b1010_0111_0100_1010_1110", "02472256", "0o2472256", "0x_0A_74_AE", "0x_0a_74_ae", "685230", "+685_230", "190:20:30"];
        let results = [15, 685230, 685230, 685230, 685230, 685230, 685230, 685230, 685230];


        for i in 0 .. options.len () {
            if let Ok (tagged) = int.decode11 (true, &options[i].to_string ().into_bytes ()) {
                assert_eq! (tagged.get_tag (), Cow::from (TAG));

                let intval: Result<IntValue, _> = tagged.into ();
                let intval = intval.unwrap ();

                let i64val: Result<i64, BigInt> = intval.into ();
                assert! (i64val.is_ok ());

                let val = i64val.unwrap ();

                assert_eq! (val, results[i]);
            } else { assert! (false) }
        }
    }


    #[test]
    fn decode_bigint () {
        let int = Int; // ::new (&get_charset_utf8 ());

        let option = "1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890";
        let result = BigInt::from_str (option).unwrap ();

        if let Ok (tagged) = int.decode (true, &option.to_string ().into_bytes ()) {
            assert_eq! (tagged.get_tag (), Cow::from (TAG));

            let intval: Result<IntValue, _> = tagged.into ();
            let intval = intval.unwrap ();

            let theval: Result<i64, BigInt> = intval.into ();
            assert! (theval.is_err ());

            let val = theval.unwrap_err ();

            assert_eq! (val, result);
        } else { assert! (false) }
    }


    #[test]
    fn decode_nl () {
        let int = Int; // ::new (&get_charset_utf8 ());

        if let Ok (_) = int.decode (true, &"\n".to_string ().into_bytes ()) {
            assert! (false);
        } else {}

        if let Ok (_) = int.decode (true, &r#""\n""#.to_string ().into_bytes ()) {
            assert! (false);
        } else {}

        if let Ok (_) = int.decode (true, &r#""""#.to_string ().into_bytes ()) {
            assert! (false);
        } else {}

        if let Ok (_) = int.decode (true, &r#"'\n'"#.to_string ().into_bytes ()) {
            assert! (false);
        } else {}

        if let Ok (_) = int.decode (true, &r#"''"#.to_string ().into_bytes ()) {
            assert! (false);
        } else {}
    }
}
