extern crate num;
extern crate skimmer;

use self::num::{ BigInt, BigUint, ToPrimitive };
use self::skimmer::symbol::{ Char, Symbol };

use txt::{ CharSet, Encoding, Twine };

use model::{ model_issue_rope, EncodedString, Factory, Model, Node, Rope, Renderer, Tagged, TaggedValue };
use model::style::CommonStyles;

use std::any::Any;
use std::fmt;
use std::ops::{ AddAssign, MulAssign, Neg };
use std::iter::Iterator;




pub const TAG: &'static str = "tag:yaml.org,2002:int";
static TWINE_TAG: Twine = Twine::Static (TAG);




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




pub struct Int {
    encoding: Encoding,
    tbl: [Char; 22],
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

    char_len: usize
}



impl Int {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }


    pub fn new (cset: &CharSet) -> Int {
        let chars = [&cset.colon, &cset.hyphen_minus, &cset.plus, &cset.low_line, &cset.letter_b, &cset.letter_o, &cset.letter_x,
                     &cset.digit_0, &cset.digit_1, &cset.digit_2, &cset.digit_3, &cset.digit_4, &cset.digit_5,
                     &cset.digit_6, &cset.digit_7, &cset.digit_8, &cset.digit_9, &cset.letter_a, &cset.letter_b,
                     &cset.letter_c, &cset.letter_d, &cset.letter_e, &cset.letter_f, &cset.letter_t_a, &cset.letter_t_b,
                     &cset.letter_t_c, &cset.letter_t_d, &cset.letter_t_e, &cset.letter_t_f, &cset.line_feed,
                     &cset.carriage_return, &cset.space, &cset.tab_h];

        let mut char_len = 1;

        for i in 0 .. chars.len () {
            if chars[i].len () > char_len { char_len = chars[i].len (); }
        }

        Int {
            encoding: cset.encoding,

            char_len: char_len,

            colon: cset.colon.clone (),
            minus: cset.hyphen_minus.clone (),
            plus: cset.plus.clone (),
            underscore: cset.low_line.clone (),
            line_feed: cset.line_feed.clone (),
            carriage_return: cset.carriage_return.clone (),
            space: cset.space.clone (),
            tab_h: cset.tab_h.clone (),
            letter_b: cset.letter_b.clone (),
            letter_o: cset.letter_o.clone (),
            letter_x: cset.letter_x.clone (),

            s_quote: cset.apostrophe.clone (),
            d_quote: cset.quotation.clone (),

            tbl: [
                cset.digit_0.clone (),
                cset.digit_1.clone (),
                cset.digit_2.clone (),
                cset.digit_3.clone (),
                cset.digit_4.clone (),
                cset.digit_5.clone (),
                cset.digit_6.clone (),
                cset.digit_7.clone (),
                cset.digit_8.clone (),
                cset.digit_9.clone (),
                cset.letter_a.clone (),
                cset.letter_b.clone (),
                cset.letter_c.clone (),
                cset.letter_d.clone (),
                cset.letter_e.clone (),
                cset.letter_f.clone (),
                cset.letter_t_a.clone (),
                cset.letter_t_b.clone (),
                cset.letter_t_c.clone (),
                cset.letter_t_d.clone (),
                cset.letter_t_e.clone (),
                cset.letter_t_f.clone ()
            ]
        }
    }


    fn base_decode (&self, explicit: bool, value: &[u8], sexagesimals: bool, shortocts: bool) -> Result<Mint, ()> {
        let mut val = Mint::new ();
        let mut state: u8 = 0;

        const STATE_SIGN: u8 = 1;
        const STATE_SIGN_N: u8 = 3;

        const STATE_NUM: u8 = 4;
        const STATE_BIN: u8 = 4 + 8;
        const STATE_HEX: u8 = 4 + 16;
        const STATE_OCT: u8 = 4 + 32;
        const STATE_DEC: u8 = 4 + 64;
        const STATE_END: u8 = 4 + 128;


        let vlen: usize = value.len ();
        let mut ptr: usize = 0;

        let mut quote_state = 0; // 1 - single, 2 - double


        'top: loop {
            if ptr >= vlen { break; }


            if quote_state == 1 {
                if self.s_quote.contained_at (value, ptr) {
                    if ptr == self.s_quote.len () { return Err ( () ) }
                    ptr += self.s_quote.len ();
                    quote_state = 0;
                    state = state | STATE_END;
                    continue;
                }
            }


            if quote_state == 2 {
                if self.d_quote.contained_at (value, ptr) {
                    if ptr == self.d_quote.len () { return Err ( () ) }
                    ptr += self.d_quote.len ();
                    quote_state = 0;
                    state = state | STATE_END;
                    continue;
                }
            }


            if explicit && quote_state == 0 && state & STATE_END == 0 {
                if self.s_quote.contained_at (value, ptr) {
                    ptr += self.s_quote.len ();
                    quote_state = 1;
                }
            }


            if explicit && quote_state == 0 && state & STATE_END == 0 {
                if self.d_quote.contained_at (value, ptr) {
                    ptr += self.d_quote.len ();
                    quote_state = 2;
                }
            }


            if state & STATE_END == STATE_END {
                if ptr == 0 { return Err ( () ) }
                if quote_state > 0 { return Err ( () ) }

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


            if state & STATE_SIGN == 0 {
                if self.minus.contained_at (value, ptr) {
                    state = state | STATE_SIGN_N;
                    ptr += self.minus.len ();
                    continue;
                }

                if self.plus.contained_at (value, ptr) {
                    state = state | STATE_SIGN;
                    ptr += self.plus.len ();
                    continue;
                }

                state = state | STATE_SIGN;
            }


            if state & STATE_NUM == 0 {
                if self.tbl[0].contained_at (value, ptr) {
                    state = state | STATE_NUM;
                    ptr += self.tbl[0].len ();

                    if self.letter_b.contained_at (value, ptr) {
                        state = state | STATE_BIN;
                        ptr += self.letter_b.len ();
                        continue;
                    }

                    if self.letter_o.contained_at (value, ptr) {
                        state = state | STATE_OCT;
                        ptr += self.letter_o.len ();
                        continue;
                    }

                    if self.letter_x.contained_at (value, ptr) {
                        state = state | STATE_HEX;
                        ptr += self.letter_x.len ();
                        continue;
                    }

                    if self.tbl[0].contained_at (value, ptr) {
                        state = state | if shortocts { STATE_OCT } else { STATE_DEC };
                        continue;
                    }

                    if self.tbl[1].contained_at (value, ptr) {
                        state = state | if shortocts { STATE_OCT } else { STATE_DEC };
                        continue;
                    }

                    if self.tbl[2].contained_at (value, ptr) {
                        state = state | if shortocts { STATE_OCT } else { STATE_DEC };
                        continue;
                    }

                    if self.tbl[3].contained_at (value, ptr) {
                        state = state | if shortocts { STATE_OCT } else { STATE_DEC };
                        continue;
                    }

                    if self.tbl[4].contained_at (value, ptr) {
                        state = state | if shortocts { STATE_OCT } else { STATE_DEC };
                        continue;
                    }

                    if self.tbl[5].contained_at (value, ptr) {
                        state = state | if shortocts { STATE_OCT } else { STATE_DEC };
                        continue;
                    }

                    if self.tbl[6].contained_at (value, ptr) {
                        state = state | if shortocts { STATE_OCT } else { STATE_DEC };
                        continue;
                    }

                    if self.tbl[7].contained_at (value, ptr) {
                        state = state | if shortocts { STATE_OCT } else { STATE_DEC };
                        continue;
                    }

                    if self.tbl[8].contained_at (value, ptr) {
                        state = state | STATE_DEC;
                        continue;
                    }

                    if self.tbl[9].contained_at (value, ptr) {
                        state = state | STATE_DEC;
                        continue;
                    }

                    state = state | STATE_END;
                    continue;
                }

                state = state | STATE_DEC;
                continue;
            }

            if state & STATE_BIN == STATE_BIN {
                'bin: loop {
                    if ptr >= vlen { break 'top; }

                    if self.tbl[0].contained_at (value, ptr) {
                        val *= 2;
                        ptr += self.tbl[0].len ();
                        continue;
                    }

                    if self.tbl[1].contained_at (value, ptr) {
                        val *= 2;
                        val += 1;
                        ptr += self.tbl[1].len ();
                        continue;
                    }

                    if self.underscore.contained_at (value, ptr) {
                        ptr += self.underscore.len ();
                        continue;
                    }

                    state = state | STATE_END;
                    continue 'top;
                }
            }


            if state & STATE_OCT == STATE_OCT {
                'oct: loop {
                    if ptr >= vlen { break 'top; }

                    for idx in 0 .. 8 {
                        if self.tbl[idx].contained_at (value, ptr) {
                            val *= 8;
                            if idx > 0 {
                                val += idx as i64;
                            }
                            ptr += self.tbl[idx].len ();
                            continue 'oct;
                        }
                    }

                    if self.underscore.contained_at (value, ptr) {
                        ptr += self.underscore.len ();
                        continue;
                    }

                    state = state | STATE_END;
                    continue 'top;
                }
            }


            if state & STATE_HEX == STATE_HEX {
                'hex: loop {
                    if ptr >= vlen { break 'top; }

                    for idx in 0 .. 22 {
                        if self.tbl[idx].contained_at (value, ptr) {
                            val *= 16;
                            if idx > 0 {
                                val += if idx > 15 { idx - 6 } else { idx } as i64;
                            }
                            ptr += self.tbl[idx].len ();
                            continue 'hex;
                        }
                    }

                    if self.underscore.contained_at (value, ptr) {
                        ptr += self.underscore.len ();
                        continue;
                    }

                    state = state | STATE_END;
                    continue 'top;
                }
            }

            if state & STATE_DEC == STATE_DEC {
                'dec: loop {
                    if ptr >= vlen { break 'top; }

                    for idx in 0 .. 10 {
                        if self.tbl[idx].contained_at (value, ptr) {
                            val *= 10;
                            if idx > 0 {
                                val += idx as i64;
                            }
                            ptr += self.tbl[idx].len ();
                            continue 'dec;
                        }
                    }

                    if self.underscore.contained_at (value, ptr) {
                        ptr += self.underscore.len ();
                        continue;
                    }

                    if sexagesimals && self.colon.contained_at (value, ptr) {
                        ptr += self.colon.len ();

                        let digit: i64;
                        'dig1: loop {
                            for idx in 0 .. 10 {
                                if self.tbl[idx].contained_at (value, ptr) {
                                    ptr += self.tbl[idx].len ();
                                    digit = idx as i64;
                                    break 'dig1;
                                }
                            }

                            return Err ( () )
                        }

                        let mut digit2: Option<i64> = None;

                        if digit < 6 {
                            'dig2: loop {
                                for idx in 0 .. 10 {
                                    if self.tbl[idx].contained_at (value, ptr) {
                                        ptr += self.tbl[idx].len ();
                                        digit2 = Some (idx as i64);
                                        break 'dig2;
                                    }
                                }
                                break;
                            }
                        }

                        let num: i64 = if digit2.is_some () {
                            digit * 10 + digit2.unwrap ()
                        } else { digit };

                        val *= 60;
                        val += num;

                        if vlen == ptr {
                            break 'top;
                        } else if self.colon.contained_at (value, ptr) {
                            continue;
                        } else {
                            state = state | STATE_END;
                            continue 'top;
                        }
                    }

                    state = state | STATE_END;
                    continue 'top;
                }
            }

            unreachable! ();
        }

        if state & STATE_NUM != STATE_NUM { return Err ( () ) }
        if state & STATE_SIGN_N == STATE_SIGN_N { val = -val; };

        Ok (val)
    }
}



impl Model for Int {
    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

    fn get_encoding (&self) -> Encoding { self.encoding }

    fn is_decodable (&self) -> bool { true }

    fn is_encodable (&self) -> bool { true }


    fn encode (&self, _renderer: &Renderer, value: TaggedValue, tags: &mut Iterator<Item=&(Twine, Twine)>) -> Result<Rope, TaggedValue> {
        let mut value = match <TaggedValue as Into<Result<IntValue, TaggedValue>>>::into (value) {
            Ok (value) => value,
            Err (value) => return Err (value)
        };

        let issue_tag = value.issue_tag ();
        let alias = value.take_alias ();
        let value = value.value;

        let src = format! ("{}", value);

        let mut production: Vec<u8> = Vec::with_capacity (src.len () * self.char_len);

        for b in src.as_bytes () {
            let w = match *b {
                b'0' => &self.tbl[0],
                b'1' => &self.tbl[1],
                b'2' => &self.tbl[2],
                b'3' => &self.tbl[3],
                b'4' => &self.tbl[4],
                b'5' => &self.tbl[5],
                b'6' => &self.tbl[6],
                b'7' => &self.tbl[7],
                b'8' => &self.tbl[8],
                b'9' => &self.tbl[9],

                b'-' => &self.minus,

                _ => unreachable! () // TODO: make sure that's true
            };

            production.extend (w.as_slice ())
        }

        let node = Node::String (EncodedString::from (production));

        Ok (model_issue_rope (self, node, issue_tag, alias, tags))
    }


    fn decode (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let result = try! (self.base_decode (explicit, value, false, false));
        Ok ( TaggedValue::from (IntValue::new (result)) )
    }


    fn decode11 (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let result = try! (self.base_decode (explicit, value, true, true));
        Ok ( TaggedValue::from (IntValue::new (result)) )
    }
}




#[derive (Clone, Debug)]
pub struct IntValue {
    style: u8,
    alias: Option<Twine>,
    value: Mint
}



impl IntValue {
    fn new (value: Mint) -> IntValue { IntValue { style: 0, alias: None, value: value } }

    pub fn set_alias (&mut self, alias: Option<Twine>) { self.alias = alias; }

    pub fn take_alias (&mut self) -> Option<Twine> { self.alias.take () }

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
    fn get_tag (&self) -> &Twine { Int::get_tag () }

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



pub struct IntFactory;



impl Factory for IntFactory {
    fn get_tag (&self) -> &Twine { Int::get_tag () }

    fn build_model (&self, cset: &CharSet) -> Box<Model> { Box::new (Int::new (cset)) }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;
    use super::num::BigInt;

    use model::{ Factory, Tagged, Renderer };
    use txt::get_charset_utf8;

    use std::iter;
    use std::str::FromStr;



    #[test]
    fn tag () {
        let int = IntFactory.build_model (&get_charset_utf8 ());

        assert_eq! (int.get_tag (), TAG);
    }



    #[test]
     fn encode () {
        let renderer = Renderer::new (&get_charset_utf8 ());
        let int = IntFactory.build_model (&get_charset_utf8 ());


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
        let int = IntFactory.build_model (&get_charset_utf8 ());


        let options = ["0b0000_1111", "0b1010_0111_0100_1010_1110", "02472256",
                       "0o2472256", "0x_0A_74_AE", "0x_0a_74_ae", "685230",
                       "+685_230", "0b0000_1111  	"];
        let results = [15, 685230, 2472256, 685230, 685230, 685230, 685230, 685230, 15];


        for i in 0 .. options.len () {
            if let Ok (tagged) = int.decode (true, &options[i].to_string ().into_bytes ()) {
                assert_eq! (tagged.get_tag (), Int::get_tag ());

                let intval: Result<IntValue, _> = tagged.into ();
                let intval = intval.unwrap ();

                let i64val: Result<i64, BigInt> = intval.into ();
                assert! (i64val.is_ok ());

                let val = i64val.unwrap ();

                assert_eq! (val, results[i] as i64);
            } else { assert! (false) }
        }


        let decoded = int.decode (true, &"190:20:30".to_string ().into_bytes ());
        assert! (decoded.is_err ());
    }


    #[test]
    fn decode11 () {
        let int = IntFactory.build_model (&get_charset_utf8 ());


        let options = ["0b0000_1111", "0b1010_0111_0100_1010_1110", "02472256", "0o2472256", "0x_0A_74_AE", "0x_0a_74_ae", "685230", "+685_230", "190:20:30"];
        let results = [15, 685230, 685230, 685230, 685230, 685230, 685230, 685230, 685230];


        for i in 0 .. options.len () {
            if let Ok (tagged) = int.decode11 (true, &options[i].to_string ().into_bytes ()) {
                assert_eq! (tagged.get_tag (), Int::get_tag ());

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
        let int = IntFactory.build_model (&get_charset_utf8 ());

        let option = "1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890";
        let result = BigInt::from_str (option).unwrap ();

        if let Ok (tagged) = int.decode (true, &option.to_string ().into_bytes ()) {
            assert_eq! (tagged.get_tag (), Int::get_tag ());

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
        let int = IntFactory.build_model (&get_charset_utf8 ());

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
