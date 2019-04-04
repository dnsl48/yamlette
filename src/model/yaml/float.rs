extern crate fraction;
extern crate num;
extern crate skimmer;

use self::fraction::convert::TryToConvertFrom;
use self::fraction::{BigFraction, BigUint, DynaInt, One, Sign, Zero};

use self::num::traits::Signed;
use self::num::ToPrimitive;

use model::style::CommonStyles;
use model::Fraction;
use model::{model_issue_rope, EncodedString, Model, Node, Renderer, Rope, Tagged, TaggedValue};

use std::any::Any;
use std::borrow::Cow;
// use std::cmp::Ordering;
// use std::fmt;
use std::iter::Iterator;
// use std::mem;
// use std::ops::{AddAssign, DivAssign, MulAssign, Neg};

pub static TAG: &'static str = "tag:yaml.org,2002:float";

#[derive(Clone, Copy)]
pub struct Float;

impl Float {
    pub fn get_tag() -> Cow<'static, str> {
        Cow::from(TAG)
    }

    fn base_decode(
        &self,
        explicit: bool,
        value: &[u8],
        base60: bool,
        optional_dot: bool,
    ) -> Result<Fraction, ()> {
        if !explicit
            && !match value.get(0).map(|b| *b) {
                Some(b'+') | Some(b'-') | Some(b'.') | Some(b'0'...b'9') => true,
                _ => false,
            }
        {
            return Err(());
        }

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

        let mut exp: DynaInt<u64, BigUint> = DynaInt::zero();
        let mut num: DynaInt<u64, BigUint> = DynaInt::zero();
        let mut den: DynaInt<u64, BigUint> = DynaInt::zero();
        den += DynaInt::S(1u64);

        let mut quote_state = 0; // 1 - single, 2 - double
        let mut actual_num = false;

        let mut ptr: usize = 0;
        let mut state: u8 = 0;

        loop {
            match value.get(ptr).map(|b| *b) {
                None => break,

                Some(b'_') => {
                    ptr += 1;
                }

                Some(b'\'') => {
                    ptr += 1;
                    if explicit && quote_state == 0 && state & STATE_END == 0 {
                        quote_state = 1;
                    } else if quote_state == 1 {
                        if ptr == 2 {
                            return Err(());
                        }
                        quote_state = 0;
                        state = state | STATE_END;
                        break;
                    } else {
                        state = state | STATE_END;
                        break;
                    }
                }

                Some(b'"') => {
                    ptr += 1;
                    if explicit && quote_state == 0 && state & STATE_END == 0 {
                        quote_state = 2;
                    } else if quote_state == 2 {
                        if ptr == 2 {
                            return Err(());
                        }
                        quote_state = 0;
                        state = state | STATE_END;
                        break;
                    } else {
                        state = state | STATE_END;
                        break;
                    }
                }

                Some(b'-') if state & STATE_SIGN == 0 => {
                    ptr += 1;
                    state = state | STATE_SIGN_N;
                }

                Some(b'+') if state & STATE_SIGN == 0 => {
                    ptr += 1;
                    state = state | STATE_SIGN;
                }

                _ if state & STATE_SIGN == 0 => {
                    state = state | STATE_SIGN;
                }

                Some(b'.') if state & STATE_DOT != STATE_DOT => {
                    ptr += 1;
                    state = state | STATE_DOT;
                }

                Some(b':') if base60 && state & STATE_DOT != STATE_DOT => {
                    ptr += 1;

                    let digit: u32;

                    match value.get(ptr).map(|b| *b) {
                        Some(val @ b'0'...b'9') => {
                            ptr += 1;
                            digit = (val - b'0') as u32;
                        }
                        _ => {
                            state = state | STATE_END;
                            break;
                        }
                    }

                    let mut digit2: Option<u32> = None;

                    if digit < 6u32 {
                        match value.get(ptr).map(|b| *b) {
                            Some(val @ b'0'...b'9') => {
                                ptr += 1;
                                digit2 = Some((val - b'0') as u32);
                            }
                            _ => (),
                        };
                    };

                    let n: u32 = if digit2.is_some() {
                        digit * 10 + digit2.unwrap()
                    } else {
                        digit
                    };

                    num *= DynaInt::S(60u64);
                    num += DynaInt::S(n as u64);

                    match value.get(ptr).map(|b| *b) {
                        None => break,

                        Some(b'.') | Some(b':') => (),

                        _ => {
                            state = state | STATE_END;
                            break;
                        }
                    }
                }

                _ if state & STATE_NUM != STATE_NUM => {
                    let value = &value[ptr..];
                    if value.starts_with("nan".as_bytes())
                        || value.starts_with("NaN".as_bytes())
                        || value.starts_with("NAN".as_bytes())
                    {
                        ptr += 3;
                        nan = true;
                        state = state | STATE_END;
                        break;
                    } else if value.starts_with("inf".as_bytes())
                        || value.starts_with("Inf".as_bytes())
                        || value.starts_with("INF".as_bytes())
                    {
                        ptr += 3;
                        inf = true;
                        state = state | STATE_END;
                        break;
                    }

                    state = state | STATE_NUM;
                }

                Some(b'e') | Some(b'E')
                    if (actual_num && (optional_dot || state & STATE_DOT == STATE_DOT))
                        && state & STATE_E != STATE_E =>
                {
                    ptr += 1;
                    state = state | STATE_E;
                    exp = DynaInt::zero();
                }

                Some(b'-')
                    if state & STATE_E == STATE_E && state & STATE_E_SIGN != STATE_E_SIGN =>
                {
                    ptr += 1;
                    state = state | STATE_E_SIGN_N;
                }

                Some(b'+')
                    if state & STATE_E == STATE_E && state & STATE_E_SIGN != STATE_E_SIGN =>
                {
                    ptr += 1;
                    state = state | STATE_E_SIGN;
                }

                _ if state & STATE_E == STATE_E && STATE_E_SIGN != STATE_E_SIGN => {
                    state = state | STATE_E_SIGN;
                }

                Some(b) => {
                    let digit: u32;

                    if b >= b'0' && b <= b'9' {
                        ptr += 1;
                        digit = (b - b'0') as u32;
                    } else {
                        state = state | STATE_END;
                        break;
                    }

                    if state & STATE_E == STATE_E {
                        exp *= DynaInt::S(10u64);
                        exp += DynaInt::S(digit as u64);
                        continue;
                    }

                    if state & STATE_DOT == STATE_DOT {
                        num *= DynaInt::S(10u64);
                        num += DynaInt::S(digit as u64);
                        den *= DynaInt::S(10u64);
                        continue;
                    }

                    num *= DynaInt::S(10u64);
                    num += DynaInt::S(digit as u64);
                    actual_num = true;
                }
            }
        }

        if state & STATE_END == STATE_END {
            if ptr == 0 {
                return Err(());
            }
            if quote_state > 0 {
                match value.get(ptr).map(|b| *b) {
                    Some(b'"') if quote_state == 2 => {
                        ptr += 1;
                    }
                    Some(b'\'') if quote_state == 1 => {
                        ptr += 1;
                    }
                    _ => return Err(()),
                };
            }

            loop {
                match value.get(ptr).map(|b| *b) {
                    None => break,

                    Some(b' ') | Some(b'\n') | Some(b'\t') | Some(b'\r') => {
                        ptr += 1;
                    }

                    _ => return Err(()),
                };
            }
        }

        if nan {
            return Ok(Fraction::nan());
        } else if inf {
            return Ok(if state & STATE_SIGN_N == STATE_SIGN_N {
                Fraction::neg_infinity()
            } else {
                Fraction::infinity()
            });
        }

        if state & STATE_NUM != STATE_NUM {
            return Err(());
        }

        if state & STATE_E == STATE_E && exp > DynaInt::zero() {
            let mut eptr = DynaInt::zero();

            loop {
                if eptr >= exp {
                    break;
                }

                if state & STATE_E_SIGN_N == STATE_E_SIGN_N {
                    den *= DynaInt::S(10u64);
                } else {
                    if den > DynaInt::one() {
                        den /= DynaInt::S(10u64);
                    } else {
                        num *= DynaInt::S(10u64);
                    }
                }

                eptr += DynaInt::one();
            }
        }

        let value = Fraction::new(num, den);
        let value = if state & STATE_SIGN_N == STATE_SIGN_N {
            -value
        } else {
            value
        };

        Ok(value)
    }
}

impl Model for Float {
    fn get_tag(&self) -> Cow<'static, str> {
        Self::get_tag()
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut Any {
        self
    }

    fn is_decodable(&self) -> bool {
        true
    }

    fn is_encodable(&self) -> bool {
        true
    }

    fn encode(
        &self,
        _renderer: &Renderer,
        value: TaggedValue,
        tags: &mut Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
    ) -> Result<Rope, TaggedValue> {
        let mut value: FloatValue =
            match <TaggedValue as Into<Result<FloatValue, TaggedValue>>>::into(value) {
                Ok(value) => value,
                Err(value) => return Err(value),
            };

        let issue_tag = value.issue_tag();
        let alias = value.take_alias();
        let value = value.value;

        if value.is_nan() {
            let node = Node::String(EncodedString::from(".nan".as_bytes()));
            return Ok(model_issue_rope(self, node, issue_tag, alias, tags));
        }

        if value.is_infinite() {
            let value = if value.is_negative() { "-.inf" } else { ".inf" };

            let node = Node::String(EncodedString::from(value.as_bytes()));
            return Ok(model_issue_rope(self, node, issue_tag, alias, tags));
        }

        let value = format!("{:.64}", value);

        let node = Node::String(EncodedString::from(value.into_bytes()));
        Ok(model_issue_rope(self, node, issue_tag, alias, tags))
    }

    fn decode(&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        if let Ok(frac) = self.base_decode(explicit, value, false, true) {
            Ok(TaggedValue::from(FloatValue::new(frac)))
        } else {
            Err(())
        }
    }

    fn decode11(&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        if let Ok(frac) = self.base_decode(explicit, value, true, false) {
            Ok(TaggedValue::from(FloatValue::new(frac)))
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct FloatValue {
    style: u8,
    alias: Option<Cow<'static, str>>,
    value: Fraction,
}

impl FloatValue {
    fn new(value: Fraction) -> FloatValue {
        FloatValue {
            style: 0,
            alias: None,
            value: value,
        }
    }

    pub fn set_alias(&mut self, alias: Option<Cow<'static, str>>) {
        self.alias = alias;
    }

    pub fn take_alias(&mut self) -> Option<Cow<'static, str>> {
        self.alias.take()
    }

    pub fn init_common_styles(&mut self, common_styles: CommonStyles) {
        self.set_issue_tag(common_styles.issue_tag());
    }

    pub fn issue_tag(&self) -> bool {
        self.style & 1 == 1
    }

    pub fn set_issue_tag(&mut self, val: bool) {
        if val {
            self.style |= 1;
        } else {
            self.style &= !1;
        }
    }

    pub fn sign(&self) -> Option<Sign> {
        self.value.sign()
    }

    pub fn is_nan(&self) -> bool {
        self.value.is_nan()
    }

    pub fn is_negative(&self) -> bool {
        self.value.is_negative()
    }

    pub fn is_infinite(&self) -> bool {
        self.value.is_infinite()
    }

    pub fn format_as_float(&self) -> Option<String> {
        Some(format!("{:.64}", self.value))
    }
}

macro_rules! from_float {
    ( $($t:ty),* ) => {
        $(
        impl From<$t> for FloatValue
            where
                Fraction: From<$t>
        {
            fn from (val: $t) -> FloatValue {
                FloatValue::new (Fraction::from (val))
            }
        }
        )*
    }
}

from_float!(f32, f64);

impl From<Fraction> for FloatValue {
    fn from(f: Fraction) -> FloatValue {
        FloatValue::new(f)
    }
}

impl From<BigFraction> for FloatValue {
    fn from(f: BigFraction) -> FloatValue {
        FloatValue::new(Fraction::from_fraction(f))
    }
}

impl From<fraction::Fraction> for FloatValue {
    fn from(f: fraction::Fraction) -> FloatValue {
        FloatValue::new(Fraction::from_fraction(f))
    }
}

impl Into<BigFraction> for FloatValue {
    fn into(self) -> BigFraction {
        self.value.into_fraction()
    }
}

impl Into<Fraction> for FloatValue {
    fn into(self) -> Fraction {
        self.value
    }
}

impl Into<Result<fraction::Fraction, FloatValue>> for FloatValue {
    fn into(self) -> Result<fraction::Fraction, FloatValue> {
        match fraction::Fraction::try_to_convert_from(self.value.clone()) {
            Some(fraction) => Ok(fraction),
            None => Err(self),
        }
    }
}

impl<'a> Into<&'a Fraction> for &'a FloatValue {
    fn into(self) -> &'a Fraction {
        &self.value
    }
}

impl Tagged for FloatValue {
    fn get_tag(&self) -> Cow<'static, str> {
        Cow::from(TAG)
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_mut_any(&mut self) -> &mut Any {
        self as &mut Any
    }
}

impl ToPrimitive for FloatValue {
    fn to_i64(&self) -> Option<i64> {
        self.value.to_i64()
    }

    fn to_u64(&self) -> Option<u64> {
        self.value.to_u64()
    }
}

#[cfg(all(test, not(feature = "dev")))]
mod tests {
    use super::*;

    use super::num::traits::Signed;
    use model::{Fraction, Renderer, Tagged};
    // use txt::get_charset_utf8;

    use std::f64;
    use std::iter;

    macro_rules! encoded_fraction_is {
        ($coder:expr, $fraction:expr, $str:expr) => {{
            let renderer = Renderer; // ::new (&get_charset_utf8 ());
            if let Ok(rope) = $coder.encode(
                &renderer,
                TaggedValue::from(FloatValue::from($fraction)),
                &mut iter::empty(),
            ) {
                let bytes = rope.render(&renderer);
                assert_eq!(bytes, $str.as_bytes())
            } else {
                assert!(false)
            }
        }};
    }

    macro_rules! encoded_float_is {
        ($coder:expr, $float:expr, $str:expr) => {{
            let renderer = Renderer; // ::new (&get_charset_utf8 ());
            if let Ok(rope) = $coder.encode(
                &renderer,
                TaggedValue::from(FloatValue::from($float)),
                &mut iter::empty(),
            ) {
                let bytes = rope.render(&renderer);
                assert_eq!(bytes, $str.as_bytes())
            } else {
                assert!(false)
            }
        }};
    }

    macro_rules! decoded_is_f64 {
        ($coder:expr, $str:expr, $val:expr) => {{
            if let Ok(tagged) = $coder.decode(true, &$str.to_string().into_bytes()) {
                assert_eq!(tagged.get_tag(), Cow::from(TAG));

                let val: Fraction = tagged
                    .as_any()
                    .downcast_ref::<FloatValue>()
                    .unwrap()
                    .value
                    .clone();
                assert_eq!(val, Fraction::from($val));
            } else {
                assert!(false)
            }
        }};

        (11, $coder:expr, $str:expr, $val:expr) => {{
            if let Ok(tagged) = $coder.decode11(true, &$str.to_string().into_bytes()) {
                assert_eq!(tagged.get_tag(), Cow::from(TAG));

                let val: Fraction = tagged
                    .as_any()
                    .downcast_ref::<FloatValue>()
                    .unwrap()
                    .value
                    .clone();
                assert_eq!(val, Fraction::from($val));
            } else {
                assert!(false)
            }
        }};
    }

    macro_rules! decoded_is_frac {
        ($coder:expr, $str:expr, ($num:expr, $den:expr)) => {{
            if let Ok(tagged) = $coder.decode(true, &$str.to_string().into_bytes()) {
                assert_eq!(tagged.get_tag(), Cow::from(TAG));

                let val: Fraction = tagged
                    .as_any()
                    .downcast_ref::<FloatValue>()
                    .unwrap()
                    .value
                    .clone();
                assert_eq!(val, Fraction::new_generic(Sign::Plus, $num, $den).unwrap());
            } else {
                assert!(false)
            }
        }};

        (11, $coder:expr, $str:expr, ($num:expr, $den:expr)) => {{
            if let Ok(tagged) = $coder.decode11(true, &$str.to_string().into_bytes()) {
                assert_eq!(tagged.get_tag(), Cow::from(TAG));

                let val: Fraction = tagged
                    .as_any()
                    .downcast_ref::<FloatValue>()
                    .unwrap()
                    .value
                    .clone();
                assert_eq!(val, Fraction::new_generic(Sign::Plus, $num, $den).unwrap());
            } else {
                assert!(false)
            }
        }};
    }

    #[test]
    fn tag() {
        let float = Float; // ::new (&get_charset_utf8 ());

        assert_eq!(float.get_tag(), TAG);
    }

    #[test]
    fn encode() {
        let float = Float; // ::new (&get_charset_utf8 ());

        encoded_fraction_is!(float, Fraction::nan(), ".nan");
        encoded_fraction_is!(float, Fraction::infinity(), ".inf");
        encoded_fraction_is!(float, Fraction::neg_infinity(), "-.inf");

        encoded_float_is!(float, 0f64, "0");
        encoded_float_is!(float, 1f64, "1");
        encoded_float_is!(float, 1.1f64, "1.1");
        encoded_float_is!(float, 1234_5678.111_222e-4, "1234.5678111222");
    }

    #[test]
    fn decode_inf() {
        let float = Float; // ::new (&get_charset_utf8 ());

        if let Ok(tagged) = float.decode(true, &".inf".to_string().into_bytes()) {
            assert_eq!(tagged.get_tag(), float.get_tag());

            let val: Fraction = tagged
                .as_any()
                .downcast_ref::<FloatValue>()
                .unwrap()
                .value
                .clone();
            assert!(val.is_infinite());
            assert!(val > Fraction::from(0));
        } else {
            assert!(false)
        }

        if let Ok(tagged) = float.decode(true, &".Inf".to_string().into_bytes()) {
            assert_eq!(tagged.get_tag(), float.get_tag());

            let val: Fraction = tagged
                .as_any()
                .downcast_ref::<FloatValue>()
                .unwrap()
                .value
                .clone();
            assert!(val.is_infinite());
            assert!(val > Fraction::from(0));
        } else {
            assert!(false)
        }

        if let Ok(tagged) = float.decode(true, &".INF".to_string().into_bytes()) {
            assert_eq!(tagged.get_tag(), float.get_tag());

            let val: Fraction = tagged
                .as_any()
                .downcast_ref::<FloatValue>()
                .unwrap()
                .value
                .clone();
            assert!(val.is_infinite());
            assert!(val > Fraction::from(0));
        } else {
            assert!(false)
        }

        let decoded = float.decode(true, &".INf".to_string().into_bytes());
        assert!(decoded.is_err());

        if let Ok(tagged) = float.decode(true, &"-.inf".to_string().into_bytes()) {
            assert_eq!(tagged.get_tag(), float.get_tag());

            let val: Fraction = tagged
                .as_any()
                .downcast_ref::<FloatValue>()
                .unwrap()
                .value
                .clone();
            assert!(val.is_infinite());
            assert!(val.is_negative());
            assert!(val < Fraction::from(0));
        } else {
            assert!(false)
        }
    }

    #[test]
    fn decode_nan() {
        let float = Float; // ::new (&get_charset_utf8 ());

        if let Ok(tagged) = float.decode(true, &".nan".to_string().into_bytes()) {
            assert_eq!(tagged.get_tag(), float.get_tag());

            let val: Fraction = tagged
                .as_any()
                .downcast_ref::<FloatValue>()
                .unwrap()
                .value
                .clone();
            assert!(val.is_nan());
        } else {
            assert!(false)
        }

        if let Ok(tagged) = float.decode(true, &".NaN".to_string().into_bytes()) {
            assert_eq!(tagged.get_tag(), float.get_tag());

            let val: Fraction = tagged
                .as_any()
                .downcast_ref::<FloatValue>()
                .unwrap()
                .value
                .clone();
            assert!(val.is_nan());
        } else {
            assert!(false)
        }

        if let Ok(tagged) = float.decode(true, &".NAN".to_string().into_bytes()) {
            assert_eq!(tagged.get_tag(), float.get_tag());

            let val: Fraction = tagged
                .as_any()
                .downcast_ref::<FloatValue>()
                .unwrap()
                .value
                .clone();
            assert!(val.is_nan());
        } else {
            assert!(false)
        }

        let decoded = float.decode(true, &".NAn".to_string().into_bytes());
        assert!(decoded.is_err());
    }

    #[test]
    fn decode() {
        let float = Float; // ::new (&get_charset_utf8 ());

        decoded_is_f64!(float, "-.inf", f64::NEG_INFINITY);
        decoded_is_f64!(float, "128", 128_f64);
        decoded_is_frac!(float, "128.4", (1284, 10));
        decoded_is_frac!(float, "128.44", (12844, 100));
        decoded_is_frac!(float, "128.48604620", (1284860462, 10000000));

        decoded_is_f64!(float, "128.48604620e10", 128.48604620e10);
        decoded_is_f64!(float, "128.48604620e+12", 128.48604620e12);

        decoded_is_frac!(float, "128.48604620e-2", (1284860462, 1000000000));

        decoded_is_f64!(11, float, "01:30", 90);

        decoded_is_frac!(float, "6.8523015e+5", (68523015, 100));
        decoded_is_frac!(float, "685.230_15e+03", (68523015, 100));
        decoded_is_frac!(float, "685_230.15", (68523015, 100));

        decoded_is_frac!(11, float, "190:20:30.15", (68523015, 100));

        decoded_is_f64!(float, "12e03", 12e03);

        let decoded = float.decode(true, &"190:20:30.15".to_string().into_bytes());
        assert!(decoded.is_err());

        let decoded = float.decode(true, &"01:30".to_string().into_bytes());
        assert!(decoded.is_err());

        let decoded = float.decode11(true, &"12e03".to_string().into_bytes());
        assert!(decoded.is_err());

        let decoded = float.decode(true, &"e".to_string().into_bytes());
        assert!(decoded.is_err());

        let decoded = float.decode11(true, &"e".to_string().into_bytes());
        assert!(decoded.is_err());
    }

    #[test]
    fn decode_nl() {
        let float = Float; // ::new (&get_charset_utf8 ());

        if let Ok(_) = float.decode(true, &"\n".to_string().into_bytes()) {
            assert!(false);
        } else {
        }

        if let Ok(_) = float.decode(true, &r#""\n""#.to_string().into_bytes()) {
            assert!(false);
        } else {
        }

        if let Ok(_) = float.decode(true, &r#""""#.to_string().into_bytes()) {
            assert!(false);
        } else {
        }

        if let Ok(_) = float.decode(true, &r#"'\n'"#.to_string().into_bytes()) {
            assert!(false);
        } else {
        }

        if let Ok(_) = float.decode(true, &r#"''"#.to_string().into_bytes()) {
            assert!(false);
        } else {
        }
    }
}
