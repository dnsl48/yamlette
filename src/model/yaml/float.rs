extern crate fraction;
extern crate num;
extern crate skimmer;


use self::fraction::{ Fraction, BigFraction, Sign };

use self::num::{ BigUint, ToPrimitive };
use self::num::traits::Signed;

use self::skimmer::symbol::{ Char, Word, Symbol };


use txt::{ CharSet, Encoding, Twine };

use model::{ model_issue_rope, EncodedString, Factory, Model, Node, Rope, Renderer, Tagged, TaggedValue };
use model::style::CommonStyles;


use std::any::Any;
use std::cmp::Ordering;
use std::fmt;
use std::mem;
use std::ops::{ AddAssign, DivAssign, MulAssign, Neg };
use std::iter::Iterator;




pub const TAG: &'static str = "tag:yaml.org,2002:float";
static TWINE_TAG: Twine = Twine::Static (TAG);




#[derive (Clone, Debug, Hash)]
enum Mint {
    I (u64),
    B (Option<BigUint>)
}



impl Mint {
    pub fn new () -> Mint { Mint::I (0) }

    fn is_i (&self) -> bool {
        match *self {
            Mint::I (_) => true,
            _ => false
        }
    }

    fn get_i (&self) -> u64 {
        match *self {
            Mint::I (v) => v,
            _ => unreachable! ()
        }
    }

    fn set_i (&mut self, val: u64) {
        match *self {
            Mint::I (ref mut v) => *v = val,
            _ => unreachable! ()
        }
    }

    fn take_b (&mut self) -> BigUint {
        match *self {
            Mint::B (ref mut v) => v.take ().unwrap (),
            _ => unreachable! ()
        }
    }
}


impl PartialOrd for Mint {
    fn partial_cmp (&self, other: &Mint) -> Option<Ordering> {
        match *self {
            Mint::I (s) => {
                match *other {
                    Mint::I (o) => s.partial_cmp (&o),
                    Mint::B (Some (ref o)) => BigUint::from (s).partial_cmp (o),
                    _ => unreachable! ()
                }
            }
            Mint::B (Some (ref s)) => {
                match *other {
                    Mint::I (o) => s.partial_cmp (&BigUint::from (o)),
                    Mint::B (Some (ref o)) => s.partial_cmp (o),
                    _ => unreachable! ()
                }
            }
            _ => unreachable! ()
        }
    }
}


impl PartialEq for Mint {
    fn eq (&self, other: &Mint) -> bool {
        match *self {
            Mint::I (s) => {
                match *other {
                    Mint::I (o) => s.eq (&o),
                    Mint::B (Some (ref o)) => BigUint::from (s).eq (o),
                    _ => unreachable! ()
                }
            }
            Mint::B (Some (ref s)) => {
                match *other {
                    Mint::I (o) => s.eq (&BigUint::from (o)),
                    Mint::B (Some (ref o)) => s.eq (o),
                    _ => unreachable! ()
                }
            }
            _ => unreachable! ()
        }
    }
}


impl DivAssign<u64> for Mint {
    fn div_assign (&mut self, val: u64) {
        if self.is_i () {
            if let Some (n) = self.get_i ().checked_div (val) {
                self.set_i (n);
            } else {
                let mut bi = BigUint::from (self.get_i ());
                bi = bi / BigUint::from (val);
                *self = Mint::B (Some (bi));
            }
        } else {
            let mut bi = self.take_b ();
            bi = bi / BigUint::from (val);
            *self = Mint::B (Some (bi));
        }
    }
}


impl MulAssign<u64> for Mint {
    fn mul_assign (&mut self, val: u64) {
        if self.is_i () {
            if let Some (n) = self.get_i ().checked_mul (val) {
                self.set_i (n);
            } else {
                let mut bi = BigUint::from (self.get_i ());
                bi = bi * BigUint::from (val);
                *self = Mint::B (Some (bi));
            }
        } else {
            let mut bi = self.take_b ();
            bi = bi * BigUint::from (val);
            *self = Mint::B (Some (bi));
        }
    }
}



impl AddAssign<u64> for Mint {
    fn add_assign (&mut self, val: u64) {
        if self.is_i () {
            if let Some (n) = self.get_i ().checked_add (val) {
                self.set_i (n);
            } else {
                let mut bi = BigUint::from (self.get_i ());
                bi = bi + BigUint::from (val);
                *self = Mint::B (Some (bi));
            }
        } else {
            let mut bi = self.take_b ();
            bi = bi + BigUint::from (val);
            *self = Mint::B (Some (bi));
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




pub struct Float {
    encoding: Encoding,

    nan: [Word; 3],
    inf: [Word; 3],

    _nan: Word,
    inf_p: Word,
    inf_n: Word,

    digit: [Char; 10],
    colon: Char,
    dot: Char,
    letter_e: Char,
    letter_t_e: Char,
    minus: Char,
    plus: Char,
    underscore: Char,
    line_feed: Char,
    carriage_return: Char,
    space: Char,
    tab_h: Char,

    s_quote: Char,
    d_quote: Char,

    chr_len: usize
}



impl Float {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }


    pub fn new (cset: &CharSet) -> Float {
        let chars = [&cset.colon, &cset.hyphen_minus, &cset.full_stop, &cset.plus, &cset.low_line, &cset.letter_e,
                     &cset.letter_t_e, &cset.digit_0, &cset.digit_1, &cset.digit_2, &cset.digit_3, &cset.digit_4,
                     &cset.digit_5, &cset.digit_6, &cset.digit_7, &cset.digit_8, &cset.digit_9, &cset.line_feed,
                     &cset.carriage_return, &cset.space, &cset.tab_h];

        let mut char_len = 1;

        for i in 0 .. chars.len () {
            if chars[i].len () > char_len { char_len = chars[i].len (); }
        }


        Float {
            encoding: cset.encoding,

            nan: [
                Word::combine (&[&cset.letter_n, &cset.letter_a, &cset.letter_n]),
                Word::combine (&[&cset.letter_t_n, &cset.letter_a, &cset.letter_t_n]),
                Word::combine (&[&cset.letter_t_n, &cset.letter_t_a, &cset.letter_t_n])
            ],

            inf: [
                Word::combine (&[&cset.letter_i, &cset.letter_n, &cset.letter_f]),
                Word::combine (&[&cset.letter_t_i, &cset.letter_n, &cset.letter_f]),
                Word::combine (&[&cset.letter_t_i, &cset.letter_t_n, &cset.letter_t_f])
            ],

            _nan: Word::combine (&[
                &cset.full_stop,
                &cset.letter_n,
                &cset.letter_a,
                &cset.letter_n
            ]),

            inf_p: Word::combine (&[
                &cset.full_stop,
                &cset.letter_i,
                &cset.letter_n,
                &cset.letter_f
            ]),

            inf_n: Word::combine (&[
                &cset.hyphen_minus,
                &cset.full_stop,
                &cset.letter_i,
                &cset.letter_n,
                &cset.letter_f
            ]),

            digit: [
                cset.digit_0.clone (),
                cset.digit_1.clone (),
                cset.digit_2.clone (),
                cset.digit_3.clone (),
                cset.digit_4.clone (),
                cset.digit_5.clone (),
                cset.digit_6.clone (),
                cset.digit_7.clone (),
                cset.digit_8.clone (),
                cset.digit_9.clone ()
            ],

            colon: cset.colon.clone (),
            dot: cset.full_stop.clone (),
            letter_e: cset.letter_e.clone (),
            letter_t_e: cset.letter_t_e.clone (),
            minus: cset.hyphen_minus.clone (),
            plus: cset.plus.clone (),
            underscore: cset.low_line.clone (),
            line_feed: cset.line_feed.clone (),
            carriage_return: cset.carriage_return.clone (),
            space: cset.space.clone (),
            tab_h: cset.tab_h.clone (),

            s_quote: cset.apostrophe.clone (),
            d_quote: cset.quotation.clone (),

            chr_len: char_len
        }
    }


    #[inline]
    fn base_decode (&self, explicit: bool, value: &[u8], base60: bool, optional_dot: bool) -> Result<MaybeBigFraction, ()> {
        let mut ptr: usize = 0;
        let mut state: u8 = 0;

        const STATE_SIGN: u8 = 1;
        const STATE_SIGN_N: u8 = 3;

        const STATE_NUM: u8 = 4;

        const STATE_DOT: u8 = 8;

        const STATE_E: u8 = 16;
        const STATE_E_SIGN: u8 = 32 + 16;
        const STATE_E_SIGN_N: u8 = 64 + 32 + 16;

        const STATE_END: u8 = 128;


        //    [-+]?([0-9][0-9_]*)?(:[0-5][0-9])*\.[0-9.]*([eE][-+][0-9]+)?
        //     |        |               |        |   |     |    |         |
        // STATE_SIGN   |               |  STATE_DOT |  STATE_E |    STATE_END
        //              |               |            |          |
        //          STATE_NUM   <stateless parse>    |    STATE_E_SIGN
        //                                           |
        //                                       STATE_NUM

        let mut inf: bool = false;
        let mut nan: bool = false;

        let mut exp = Mint::new ();
        let mut num = Mint::new ();
        let mut den = Mint::new ();
        den += 1u64;

        let mut quote_state = 0; // 1 - single, 2 - double
        let mut actual_num = false;

        let vlen = value.len ();


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


            if self.underscore.contained_at (value, ptr) {
                ptr += self.underscore.len ();
                continue;
            }


            if state & STATE_SIGN != STATE_SIGN {
                if self.minus.contained_at (value, ptr) {
                    state = state | STATE_SIGN_N;
                    ptr += self.minus.len ();
                    continue;
                } else if self.plus.contained_at (value, ptr) {
                    state = state | STATE_SIGN;
                    ptr += self.plus.len ();
                    continue;
                }

                state = state | STATE_SIGN;
            }

            if state & STATE_DOT != STATE_DOT {
                if self.dot.contained_at (value, ptr) {
                    state = state | STATE_DOT;
                    ptr += self.dot.len ();
                    continue;
                } else if base60 && self.colon.contained_at (value, ptr) {
                    ptr += self.colon.len ();

                    let mut digit: u32 = 0;

                    for i in 0 .. 11 {
                        if i == 10 {
                            state = state | STATE_END;
                            continue 'top;
                        }

                        if self.digit[i].contained_at (value, ptr) {
                            ptr += self.digit[i].len ();
                            digit = i as u32;
                            break;
                        }
                    }

                    let mut digit2: Option<u32> = None;

                    if digit < 6u32 {
                        for i in 0 .. 10 {
                            if self.digit[i].contained_at (value, ptr) {
                                ptr += self.digit[i].len ();
                                digit2 = Some (i as u32);
                                break;
                            }
                        }
                    };

                    let n: u32 = if digit2.is_some () {
                        digit * 10 + digit2.unwrap ()
                    } else { digit };

                    num *= 60u64;
                    num += n as u64;

                    if vlen == ptr {
                        break;
                    } else if self.dot.contained_at (value, ptr) {
                        continue;
                    } else if self.colon.contained_at (value, ptr) {
                        continue;
                    } else {
                        state = state | STATE_END;
                        continue 'top;
                    }
                }
            } else if state & STATE_NUM != STATE_NUM {
                for i in 0 .. self.nan.len () {
                    if self.nan[i].contained_at (value, ptr) {
                        ptr += self.nan[i].len ();
                        nan = true;
                        state = state | STATE_END;
                        continue 'top;
                    }
                }

                for i in 0 .. self.inf.len () {
                    if self.inf[i].contained_at (value, ptr) {
                        ptr += self.inf[i].len ();
                        inf = true;
                        state = state | STATE_END;
                        continue 'top;
                    }
                }
            }
            state = state | STATE_NUM;

            if (actual_num && (optional_dot || state & STATE_DOT == STATE_DOT)) && state & STATE_E != STATE_E {
                if self.letter_e.contained_at (value, ptr) {
                    state = state | STATE_E;
                    ptr += self.letter_e.len ();
                    exp = Mint::new ();
                    continue;
                }

                if self.letter_t_e.contained_at (value, ptr) {
                    state = state | STATE_E;
                    ptr += self.letter_t_e.len ();
                    exp = Mint::new ();
                    continue;
                }
            } else if state & STATE_E == STATE_E && state & STATE_E_SIGN != STATE_E_SIGN {
                if self.minus.contained_at (value, ptr) {
                    state = state | STATE_E_SIGN_N;
                    ptr += self.minus.len ();
                    continue;
                } else if self.plus.contained_at (value, ptr) {
                    state = state | STATE_E_SIGN;
                    ptr += self.plus.len ();
                    continue;
                }

                state = state | STATE_E_SIGN;
            }


            let mut digit: u32 = 0;

            for i in 0 .. 11 {
                if i == 10 {
                    state = state | STATE_END;
                    continue 'top;
                }

                if self.digit[i].contained_at (value, ptr) {
                    ptr += self.digit[i].len ();
                    digit = i as u32;
                    break;
                }
            }

            if state & STATE_E == STATE_E {
                exp *= 10u64;
                exp += digit as u64;
                continue;
            }


            if state & STATE_DOT == STATE_DOT {
                num *= 10u64;
                num += digit as u64;
                den *= 10u64;
                continue;
            }

            num *= 10u64;
            num += digit as u64;
            actual_num = true;
        }

        if nan { return Ok (MaybeBigFraction::from (Fraction::nan ())); }
        else if inf {
            return Ok (if state & STATE_SIGN_N == STATE_SIGN_N {
                MaybeBigFraction::from (Fraction::neg_infinity ())
            } else {
                MaybeBigFraction::from (Fraction::infinity ())
            });
        }

        if state & STATE_NUM != STATE_NUM { return Err ( () ) }

        if state & STATE_E == STATE_E && exp > Mint::new () {
            let mut eptr = Mint::new ();

            loop {
                if eptr >= exp { break; }

                if state & STATE_E_SIGN_N == STATE_E_SIGN_N {
                    den *= 10u64;
                } else {
                    if den > Mint::I (1u64) {
                        den /= 10;
                    } else {
                        num *= 10;
                    }
                }

                eptr += 1u64;
            }
        }

        let value = MaybeBigFraction::new (num, den);
        let value = if state & STATE_SIGN_N == STATE_SIGN_N { -value } else { value };

        Ok (value)
    }
}



impl Model for Float {
    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

    fn get_encoding (&self) -> Encoding { self.encoding }


    fn is_decodable (&self) -> bool { true }

    fn is_encodable (&self) -> bool { true }


    fn encode (&self, _renderer: &Renderer, value: TaggedValue, tags: &mut Iterator<Item=&(Twine, Twine)>) -> Result<Rope, TaggedValue> {
        let mut value: FloatValue = match <TaggedValue as Into<Result<FloatValue, TaggedValue>>>::into (value) {
            Ok (value) => value,
            Err (value) => return Err (value)
        };

        let issue_tag = value.issue_tag ();
        let alias = value.take_alias ();
        let value = value.value;

        if value.is_nan () {
            return Ok ( model_issue_rope (self, Node::String (EncodedString::from (self._nan.new_vec ())), issue_tag, alias, tags) )
        }

        if value.is_infinite () {
            if value.is_negative () { return Ok (model_issue_rope (self, Node::String (EncodedString::from (self.inf_n.new_vec ())), issue_tag, alias, tags)) }
            else { return Ok (model_issue_rope (self, Node::String (EncodedString::from (self.inf_p.new_vec ())), issue_tag, alias, tags)) }
        }

        let src = if let Some (value) = value.format_as_float () {
            value
        } else {
            let mut val = FloatValue::from (value);
            val.set_issue_tag (issue_tag);
            val.set_alias (alias);

            return Err (TaggedValue::from (val) )
        };

        let mut production: Vec<u8> = Vec::with_capacity (src.len () * self.chr_len);

        for bt in src.as_bytes () {
            let symbol = match *bt {
                b'0' => &self.digit[0],
                b'1' => &self.digit[1],
                b'2' => &self.digit[2],
                b'3' => &self.digit[3],
                b'4' => &self.digit[4],
                b'5' => &self.digit[5],
                b'6' => &self.digit[6],
                b'7' => &self.digit[7],
                b'8' => &self.digit[8],
                b'9' => &self.digit[9],

                b'.' => &self.dot,
                b'e' => &self.letter_e,
                b'-' => &self.minus,
                b'+' => &self.plus,

                b'E' => &self.letter_t_e,
                b'_' => &self.underscore,

                _ => unreachable! () // TODO: make sure it's true
            };

            production.extend (symbol.as_slice ());
        }

        let node = Node::String (EncodedString::from (production));

        Ok (model_issue_rope (self, node, issue_tag, alias, tags))
    }


    fn decode (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        if let Ok (frac) = self.base_decode (explicit, value, false, true) {
            Ok ( TaggedValue::from (FloatValue::new (frac)) )
        } else { Err ( () ) }
    }


    fn decode11 (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        if let Ok (frac) = self.base_decode (explicit, value, true, false) {
            Ok ( TaggedValue::from (FloatValue::new (frac)) )
        } else { Err ( () ) }
    }
}




#[derive (Clone, Debug, PartialOrd, PartialEq, Eq, Hash)]
enum MaybeBigFraction {
    Fra (Fraction),
    Big (BigFraction)
}



impl MaybeBigFraction {
    pub fn new (mut num: Mint, mut den: Mint) -> MaybeBigFraction {
        if num.is_i () && den.is_i () {
            MaybeBigFraction::Fra (Fraction::new (num.get_i (), den.get_i ()))
        } else {
            let n = if num.is_i () { BigUint::from (num.get_i ()) } else { num.take_b () };
            let d = if den.is_i () { BigUint::from (den.get_i ()) } else { den.take_b () };
            MaybeBigFraction::Big (BigFraction::new (n, d))
        }
    }


    pub fn is_nan (&self) -> bool {
        match *self {
            MaybeBigFraction::Fra (ref f) => f.is_nan (),
            MaybeBigFraction::Big (ref f) => f.is_nan ()
        }
    }


    pub fn is_infinite (&self) -> bool {
        match *self {
            MaybeBigFraction::Fra (ref f) => f.is_infinite (),
            MaybeBigFraction::Big (ref f) => f.is_infinite ()
        }
    }


    pub fn is_negative (&self) -> bool {
        match *self {
            MaybeBigFraction::Fra (ref f) => f.is_negative (),
            MaybeBigFraction::Big (ref f) => f.is_negative ()
        }
    }


    pub fn format_as_float (&self) -> Option<String> {
        match *self {
            MaybeBigFraction::Fra (ref f) => f.format_as_float (),
            MaybeBigFraction::Big (ref f) => f.format_as_float ()
        }
    }


    pub fn sign (&self) -> Option<&Sign> {
        match *self {
            MaybeBigFraction::Fra (ref f) => f.sign (),
            MaybeBigFraction::Big (ref f) => f.sign ()
        }
    }


    pub fn promote (&mut self) {
        let mbf = mem::replace (self, MaybeBigFraction::Fra (Fraction::new_nan ()));
        *self = match mbf {
            MaybeBigFraction::Fra (f) => MaybeBigFraction::Big (f.into_big ()),
            MaybeBigFraction::Big (f) => MaybeBigFraction::Big (f)
        };
    }
}



impl Neg for MaybeBigFraction {
    type Output = Self;

    fn neg (self) -> Self {
        match self {
            MaybeBigFraction::Fra (f) => MaybeBigFraction::Fra (-f),
            MaybeBigFraction::Big (f) => MaybeBigFraction::Big (-f)
        }
    }
}



impl Into<BigFraction> for MaybeBigFraction {
    fn into (self) -> BigFraction {
        match self {
            MaybeBigFraction::Fra (f) => f.into_big (),
            MaybeBigFraction::Big (f) => f
        }
    }
}



impl From<Fraction> for MaybeBigFraction {
    fn from (val: Fraction) -> Self { MaybeBigFraction::Fra (val) }
}




#[derive (Clone, Debug)]
pub struct FloatValue {
    style: u8,
    alias: Option<Twine>,
    value: MaybeBigFraction
}



impl FloatValue {
    fn new (value: MaybeBigFraction) -> FloatValue { FloatValue { style: 0, alias: None, value: value } }

    pub fn set_alias (&mut self, alias: Option<Twine>) { self.alias = alias; }

    pub fn take_alias (&mut self) -> Option<Twine> { self.alias.take () }

    pub fn init_common_styles (&mut self, common_styles: CommonStyles) {
        self.set_issue_tag (common_styles.issue_tag ());
    }

    pub fn issue_tag (&self) -> bool { self.style & 1 == 1 }

    pub fn set_issue_tag (&mut self, val: bool) { if val { self.style |= 1; } else { self.style &= !1; } }

    pub fn sign (&self) -> Option<&Sign> { self.value.sign () }

    pub fn promote (&mut self) -> &FloatValue {
        self.value.promote ();
        self
    }

    pub fn is_nan (&self) -> bool { self.value.is_nan () }

    pub fn is_negative (&self) -> bool { self.value.is_negative () }

    pub fn is_infinite (&self) -> bool { self.value.is_infinite () }

    pub fn format_as_float (&self) -> Option<String> {
        match self.value {
            MaybeBigFraction::Fra (ref f) => f.format_as_float (),
            MaybeBigFraction::Big (ref f) => f.format_as_float ()
        }
    }
}



macro_rules! from_float {
    ( $($t:ty),* ) => {
        $(
        impl From<$t> for FloatValue
            where
                Fraction: From<$t>,
                BigFraction: From<$t>
        {
            fn from (val: $t) -> FloatValue {
                let f = Fraction::from (val);
                let maybe = if f.is_nan () {
                    let bf = BigFraction::from (val);
                    if bf.is_nan () { MaybeBigFraction::Fra (f) }
                    else { MaybeBigFraction::Big (bf) }
                } else { MaybeBigFraction::Fra (f) };

                FloatValue::new (maybe)
            }
        }
        )*
    }
}

from_float! (f32, f64);


impl From<MaybeBigFraction> for FloatValue {
    fn from (f: MaybeBigFraction) -> FloatValue { FloatValue::new (f) }
}


impl From<BigFraction> for FloatValue {
    fn from (f: BigFraction) -> FloatValue { FloatValue::new (MaybeBigFraction::Big (f)) }
}



impl From<Fraction> for FloatValue {
    fn from (f: Fraction) -> FloatValue { FloatValue::new (MaybeBigFraction::Fra (f)) }
}



impl Into<BigFraction> for FloatValue {
    fn into (self) -> BigFraction { self.value.into () }
}



impl Into<Result<Fraction, FloatValue>> for FloatValue {
    fn into (self) -> Result<Fraction, FloatValue> {
        match self.value {
            MaybeBigFraction::Fra (f) => Ok (f),
            _ => Err (self)
        }
    }
}



impl<'a> Into<Result<&'a BigFraction, &'a FloatValue>> for &'a FloatValue {
    fn into (self) -> Result<&'a BigFraction, &'a FloatValue> {
        match self.value {
            MaybeBigFraction::Big (ref f) => Ok (f),
            _ => Err (self)
        }
    }
}



impl<'a> Into<Result<&'a Fraction, &'a FloatValue>> for &'a FloatValue {
    fn into (self) -> Result<&'a Fraction, &'a FloatValue> {
        match self.value {
            MaybeBigFraction::Fra (ref f) => Ok (f),
            _ => Err (self)
        }
    }
}



impl Tagged for FloatValue {
    fn get_tag (&self) -> &Twine { Float::get_tag () }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



impl ToPrimitive for FloatValue {
    fn to_i64(&self) -> Option<i64> {
        match self.value {
            MaybeBigFraction::Fra (ref f) => f.to_i64 (),
            MaybeBigFraction::Big (ref f) => f.to_i64 ()
        }
    }


    fn to_u64(&self) -> Option<u64> {
        match self.value {
            MaybeBigFraction::Fra (ref f) => f.to_u64 (),
            MaybeBigFraction::Big (ref f) => f.to_u64 ()
        }
    }
}




pub struct FloatFactory;



impl Factory for FloatFactory {
    fn get_tag (&self) -> &Twine { Float::get_tag () }

    fn build_model (&self, cset: &CharSet) -> Box<Model> { Box::new (Float::new (cset)) }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use super::fraction::BigFraction;
    use super::num::traits::Signed;
    use super::num::BigUint;

    use model::{ Factory, Tagged, Renderer };
    use txt::get_charset_utf8;

    use std::f64;
    use std::iter;



    macro_rules! encoded_fraction_is {
        ($coder:expr, $fraction:expr, $str:expr) => {{
            let renderer = Renderer::new (&get_charset_utf8 ());
            if let Ok (rope) = $coder.encode (&renderer, TaggedValue::from (FloatValue::from ($fraction)), &mut iter::empty ()) {
                let bytes = rope.render (&renderer);
                assert_eq! (bytes, $str.as_bytes ())
            } else { assert! (false) }
        }}
    }


    macro_rules! encoded_float_is {
        ($coder:expr, $float:expr, $str:expr) => {{
            let renderer = Renderer::new (&get_charset_utf8 ());
            if let Ok (rope) = $coder.encode (&renderer, TaggedValue::from (FloatValue::from ($float)), &mut iter::empty ()) {
                let bytes = rope.render (&renderer);
                assert_eq! (bytes, $str.as_bytes ())
            } else { assert! (false) }
        }}
    }


    macro_rules! decoded_is_f64 {
        ($coder:expr, $str:expr, $val:expr) => {{
            if let Ok (tagged) = $coder.decode (true, &$str.to_string ().into_bytes ()) {
                assert_eq! (tagged.get_tag (), Float::get_tag ());

                let val: BigFraction = tagged.as_any ().downcast_ref::<FloatValue> ().unwrap ().value.clone ().into ();
                assert_eq! (val, BigFraction::from ($val));
            } else { assert! (false) }
        }};

        (11, $coder:expr, $str:expr, $val:expr) => {{
            if let Ok (tagged) = $coder.decode11 (true, &$str.to_string ().into_bytes ()) {
                assert_eq! (tagged.get_tag (), Float::get_tag ());

                let val: BigFraction = tagged.as_any ().downcast_ref::<FloatValue> ().unwrap ().value.clone ().into ();
                assert_eq! (val, BigFraction::from ($val));
            } else { assert! (false) }
        }}
    }


    macro_rules! decoded_is_frac {
        ($coder:expr, $str:expr, ($num:expr, $den:expr)) => {{
            if let Ok (tagged) = $coder.decode (true, &$str.to_string ().into_bytes ()) {
                assert_eq! (tagged.get_tag (), Float::get_tag ());

                let val: BigFraction = tagged.as_any ().downcast_ref::<FloatValue> ().unwrap ().value.clone ().into ();
                assert_eq! (val, BigFraction::new (BigUint::from ($num as u64), BigUint::from ($den as u64)));
            } else { assert! (false) }
        }};

        (11, $coder:expr, $str:expr, ($num:expr, $den:expr)) => {{
            if let Ok (tagged) = $coder.decode11 (true, &$str.to_string ().into_bytes ()) {
                assert_eq! (tagged.get_tag (), Float::get_tag ());

                let val: BigFraction = tagged.as_any ().downcast_ref::<FloatValue> ().unwrap ().value.clone ().into ();
                assert_eq! (val, BigFraction::new (BigUint::from ($num as u64), BigUint::from ($den as u64)));
            } else { assert! (false) }
        }}
    }



    #[test]
    fn tag () {
        let float = FloatFactory.build_model (&get_charset_utf8 ());

        assert_eq! (float.get_tag (), TAG);
    }



    #[test]
    fn encode () {
        let float = FloatFactory.build_model (&get_charset_utf8 ());

        encoded_fraction_is! (float, BigFraction::nan (), ".nan");
        encoded_fraction_is! (float, BigFraction::infinity (), ".inf");
        encoded_fraction_is! (float, BigFraction::neg_infinity (), "-.inf");

        encoded_float_is! (float, 0f64, "0.0");
        encoded_float_is! (float, 1f64, "1.0");
        encoded_float_is! (float, 1.1f64, "1.1");
        encoded_float_is! (float, 1234_5678.111_222e-4, "1234.5678111222");
    }



    #[test]
    fn decode_inf () {
        let float = FloatFactory.build_model (&get_charset_utf8 ());

        if let Ok (tagged) = float.decode (true, &".inf".to_string ().into_bytes ()) {
            assert_eq! (tagged.get_tag (), float.get_tag ());

            let val: BigFraction = tagged.as_any ().downcast_ref::<FloatValue> ().unwrap ().value.clone ().into ();
            assert! (val.is_infinite ());
            assert! (val > BigFraction::from (0));
        } else { assert! (false) }


        if let Ok (tagged) = float.decode (true, &".Inf".to_string ().into_bytes ()) {
            assert_eq! (tagged.get_tag (), float.get_tag ());

            let val: BigFraction = tagged.as_any ().downcast_ref::<FloatValue> ().unwrap ().value.clone ().into ();
            assert! (val.is_infinite ());
            assert! (val > BigFraction::from (0));
        } else { assert! (false) }


        if let Ok (tagged) = float.decode (true, &".INF".to_string ().into_bytes ()) {
            assert_eq! (tagged.get_tag (), float.get_tag ());

            let val: BigFraction = tagged.as_any ().downcast_ref::<FloatValue> ().unwrap ().value.clone ().into ();
            assert! (val.is_infinite ());
            assert! (val > BigFraction::from (0));
        } else { assert! (false) }


        let decoded = float.decode (true, &".INf".to_string ().into_bytes ());
        assert! (decoded.is_err ());


        if let Ok (tagged) = float.decode (true, &"-.inf".to_string ().into_bytes ()) {
            assert_eq! (tagged.get_tag (), float.get_tag ());

            let val: BigFraction = tagged.as_any ().downcast_ref::<FloatValue> ().unwrap ().value.clone ().into ();
            assert! (val.is_infinite ());
            assert! (val.is_negative ());
            assert! (val < BigFraction::from (0));
        } else { assert! (false) }
    }



    #[test]
    fn decode_nan () {
        let float = FloatFactory.build_model (&get_charset_utf8 ());


        if let Ok (tagged) = float.decode (true, &".nan".to_string ().into_bytes ()) {
            assert_eq! (tagged.get_tag (), float.get_tag ());

            let val: BigFraction = tagged.as_any ().downcast_ref::<FloatValue> ().unwrap ().value.clone ().into ();
            assert! (val.is_nan ());
        } else { assert! (false) }



        if let Ok (tagged) = float.decode (true, &".NaN".to_string ().into_bytes ()) {
            assert_eq! (tagged.get_tag (), float.get_tag ());

            let val: BigFraction = tagged.as_any ().downcast_ref::<FloatValue> ().unwrap ().value.clone ().into ();
            assert! (val.is_nan ());
        } else { assert! (false) }



        if let Ok (tagged) = float.decode (true, &".NAN".to_string ().into_bytes ()) {
            assert_eq! (tagged.get_tag (), float.get_tag ());

            let val: BigFraction = tagged.as_any ().downcast_ref::<FloatValue> ().unwrap ().value.clone ().into ();
            assert! (val.is_nan ());
        } else { assert! (false) }



        let decoded = float.decode (true, &".NAn".to_string ().into_bytes ());
        assert! (decoded.is_err ());
    }



    #[test]
    fn decode () {
        let float = FloatFactory.build_model (&get_charset_utf8 ());


        decoded_is_f64! (float, "-.inf", f64::NEG_INFINITY);
        decoded_is_f64! (float, "128", 128_f64);
        decoded_is_frac! (float, "128.4", (1284, 10));
        decoded_is_frac! (float, "128.44", (12844, 100));
        decoded_is_frac! (float, "128.48604620", (1284860462, 10000000));

        decoded_is_f64! (float, "128.48604620e10", 128.48604620e10);
        decoded_is_f64! (float, "128.48604620e+12", 128.48604620e12);

        decoded_is_frac! (float, "128.48604620e-2", (1284860462, 1000000000));

        decoded_is_f64! (11, float, "01:30", 90);

        decoded_is_frac! (float, "6.8523015e+5", (68523015, 100));
        decoded_is_frac! (float, "685.230_15e+03", (68523015, 100));
        decoded_is_frac! (float, "685_230.15", (68523015, 100));

        decoded_is_frac! (11, float, "190:20:30.15", (68523015, 100));

        decoded_is_f64! (float, "12e03", 12e03);


        let decoded = float.decode (true, &"190:20:30.15".to_string ().into_bytes ());
        assert! (decoded.is_err ());

        let decoded = float.decode (true, &"01:30".to_string ().into_bytes ());
        assert! (decoded.is_err ());

        let decoded = float.decode11 (true, &"12e03".to_string ().into_bytes ());
        assert! (decoded.is_err ());

        let decoded = float.decode (true, &"e".to_string ().into_bytes ());
        assert! (decoded.is_err ());

        let decoded = float.decode11 (true, &"e".to_string ().into_bytes ());
        assert! (decoded.is_err ());
    }



    #[test]
    fn decode_nl () {
        let float = FloatFactory.build_model (&get_charset_utf8 ());

        if let Ok (_) = float.decode (true, &"\n".to_string ().into_bytes ()) {
            assert! (false);
        } else {}

        if let Ok (_) = float.decode (true, &r#""\n""#.to_string ().into_bytes ()) {
            assert! (false);
        } else {}

        if let Ok (_) = float.decode (true, &r#""""#.to_string ().into_bytes ()) {
            assert! (false);
        } else {}

        if let Ok (_) = float.decode (true, &r#"'\n'"#.to_string ().into_bytes ()) {
            assert! (false);
        } else {}

        if let Ok (_) = float.decode (true, &r#"''"#.to_string ().into_bytes ()) {
            assert! (false);
        } else {}
    }
}
