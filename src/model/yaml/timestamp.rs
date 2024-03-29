extern crate fraction;
extern crate num;
extern crate skimmer;

use self::fraction::{BigFraction, Fraction};
use self::num::BigUint;

use crate::model::{EncodedString, Model, Node, Renderer, Rope, Tagged, TaggedValue};

use crate::model::yaml::float::FloatValue;

use std::any::Any;
use std::borrow::Cow;
use std::i32;
use std::iter::Iterator;

pub static TAG: &'static str = "tag:yaml.org,2002:timestamp";

#[derive(Clone, Copy)]
pub struct Timestamp;

impl Timestamp {
    pub fn get_tag() -> Cow<'static, str> {
        Cow::from(TAG)
    }

    fn parse_figure(&self, ptr: &mut usize, value: &[u8]) -> Option<i64> {
        let mut figure: Option<i64> = None;

        loop {
            match value.get(*ptr).map(|b| *b) {
                Some(val @ b'0'..=b'9') => {
                    let val = val - b'0';

                    figure = if let Some(nval) =
                        (if figure.is_some() { figure.unwrap() } else { 0 }).checked_mul(10)
                    {
                        if let Some(nval) = nval.checked_add(val as i64) {
                            *ptr += 1;
                            Some(nval)
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    };
                }
                _ => break,
            }
        }

        figure
    }

    fn parse_fraction(&self, ptr: &mut usize, value: &[u8]) -> Option<FloatValue> {
        let mut fraction: Option<(Result<u64, BigUint>, Result<u64, BigUint>)> = None;

        loop {
            match value.get(*ptr).map(|b| *b) {
                Some(val @ b'0'..=b'9') => {
                    *ptr += 1;
                    let val = val - b'0';

                    let mut f = if fraction.is_some() {
                        fraction.unwrap()
                    } else {
                        (Ok(0), Ok(1))
                    };

                    f.0 = match f.0 {
                        Ok(u) => {
                            if let Some(nval) = u.checked_mul(10) {
                                if let Some(nval) = nval.checked_add(val as u64) {
                                    Ok(nval)
                                } else {
                                    Err(BigUint::from(nval) + BigUint::from(val as u64))
                                }
                            } else {
                                Err(BigUint::from(u) * BigUint::from(10u8)
                                    + BigUint::from(val as u64))
                            }
                        }
                        Err(b) => Err(b * BigUint::from(10u8) + BigUint::from(val as u64)),
                    };

                    f.1 = match f.1 {
                        Ok(u) => {
                            if let Some(nval) = u.checked_mul(10) {
                                Ok(nval)
                            } else {
                                Err(BigUint::from(u) * BigUint::from(10u8))
                            }
                        }
                        Err(b) => Err(b * BigUint::from(10u8)),
                    };

                    fraction = Some((f.0, f.1));
                }
                _ => break,
            }
        }

        if let Some((num, den)) = fraction {
            if num.is_ok() && den.is_ok() {
                Some(FloatValue::from(Fraction::new(
                    num.ok().unwrap(),
                    den.ok().unwrap(),
                )))
            } else if num.is_ok() {
                Some(FloatValue::from(BigFraction::new(
                    BigUint::from(num.ok().unwrap()),
                    den.err().unwrap(),
                )))
            } else if den.is_ok() {
                Some(FloatValue::from(BigFraction::new(
                    num.err().unwrap(),
                    BigUint::from(den.ok().unwrap()),
                )))
            } else {
                Some(FloatValue::from(BigFraction::new(
                    num.err().unwrap(),
                    den.err().unwrap(),
                )))
            }
        } else {
            None
        }
    }
}

impl Model for Timestamp {
    fn get_tag(&self) -> Cow<'static, str> {
        Self::get_tag()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
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
        _tags: &mut dyn Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
    ) -> Result<Rope, TaggedValue> {
        let value: TimestampValue =
            match <TaggedValue as Into<Result<TimestampValue, TaggedValue>>>::into(value) {
                Ok(value) => value,
                Err(value) => return Err(value),
            };

        let mut src = String::with_capacity(32);

        if value.year.is_some() && value.month.is_some() && value.day.is_some() {
            src.push_str(&format!(
                "{:04}-{:02}-{:02}",
                value.year.as_ref().unwrap(),
                value.month.as_ref().unwrap(),
                value.day.as_ref().unwrap()
            ));
        }

        if value.hour.is_some() || value.minute.is_some() || value.second.is_some() {
            if src.len() > 0 {
                src.push_str("T")
            };

            src.push_str(&format!(
                "{:02}:{:02}:{:02}",
                if let Some(h) = value.hour.as_ref() {
                    *h
                } else {
                    0
                },
                if let Some(m) = value.minute.as_ref() {
                    *m
                } else {
                    0
                },
                if let Some(s) = value.second.as_ref() {
                    *s
                } else {
                    0
                }
            ));

            let fi = if let Some(f) = value.fraction.as_ref() {
                let f = f.clone().format_as_float();
                if let Some(f) = f {
                    src.push_str(&f[1..]);
                    true
                } else {
                    false
                }
            } else {
                true
            };

            if !fi {
                return Err(TaggedValue::from(value));
            }

            if let Some(h) = value.tz_hour.as_ref() {
                if *h > 0 {
                    src.push_str(&format!("{:+02}", h));
                } else {
                    src.push_str(&format!("{:+03}", h));
                }

                if let Some(ref m) = value.tz_minute.as_ref() {
                    src.push_str(&format!(":{:02}", m));
                }
            }
        }

        Ok(Rope::from(Node::String(EncodedString::from(
            src.into_bytes(),
        ))))
    }

    fn decode(&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let mut ptr: usize = 0;

        let mut state: u8 = 0;

        const STATE_YEAR: u8 = 1;
        const STATE_MONTH: u8 = 2;
        const STATE_DAY: u8 = 4;
        const STATE_HOUR: u8 = 8;
        const STATE_MINUTE: u8 = 16;
        const STATE_SECOND: u8 = 32;
        const STATE_TZ_HOUR: u8 = 64;
        const STATE_TZ_MINUTE: u8 = 128;

        let mut dt = TimestampValue::new();

        let mut quote_state = 0; // 1 - single, 2 - double

        'top: loop {
            if ptr >= value.len() {
                break;
            }

            if explicit && ptr == 0 && quote_state == 0 {
                match value.get(ptr).map(|b| *b) {
                    Some(b'\'') => {
                        ptr += 1;
                        quote_state = 1;
                        continue 'top;
                    }
                    Some(b'"') => {
                        ptr += 1;
                        quote_state = 2;
                        continue 'top;
                    }
                    _ => (),
                }
            }

            if state == 0 {
                let ltz = if let Some(b'-') = value.get(ptr).map(|b| *b) {
                    ptr += 1;
                    true
                } else {
                    false
                };

                let figure = self.parse_figure(&mut ptr, value);

                if figure.is_none() {
                    return Err(());
                }

                if !ltz && figure.unwrap() >= 0 && figure.unwrap() < 25 {
                    if let Some(b':') = value.get(ptr).map(|b| *b) {
                        state = STATE_HOUR;
                        dt = dt.hour(figure.unwrap() as u8);
                    }
                }

                if state == 0
                    && figure.unwrap() >= (i32::MIN as i64)
                    && figure.unwrap() <= (i32::MAX as i64)
                {
                    state = STATE_YEAR;
                    dt = dt.year((figure.unwrap() * if ltz { -1 } else { 1 }) as i32);
                }

                continue;
            } else if state == STATE_YEAR {
                if let Some(b'-') = value.get(ptr).map(|b| *b) {
                    ptr += 1;

                    let figure = self.parse_figure(&mut ptr, value);

                    if figure.is_none() {
                        return Err(());
                    }

                    if figure.unwrap() > 0 && figure.unwrap() < 13 {
                        state = state | STATE_MONTH;
                        dt = dt.month(figure.unwrap() as u8);
                    } else {
                        return Err(());
                    }
                } else {
                    return Err(());
                }

                continue;
            } else if state == STATE_YEAR | STATE_MONTH {
                if let Some(b'-') = value.get(ptr).map(|b| *b) {
                    ptr += 1;

                    let figure = self.parse_figure(&mut ptr, value);

                    if figure.is_none() {
                        return Err(());
                    }

                    if figure.unwrap() > 0 && figure.unwrap() < 32 {
                        state = state | STATE_DAY;
                        dt = dt.day(figure.unwrap() as u8);
                    } else {
                        return Err(());
                    }
                } else {
                    return Err(());
                }

                continue;
            } else if state == STATE_YEAR | STATE_MONTH | STATE_DAY {
                match value.get(ptr).map(|b| *b) {
                    Some(b'T') | Some(b' ') | Some(b't') | Some(b'\t') => {
                        ptr += 1;
                    }
                    _ => return Err(()),
                };

                let figure = self.parse_figure(&mut ptr, value);

                if figure.is_none() {
                    return Err(());
                }

                if figure.unwrap() >= 0 && figure.unwrap() < 25 {
                    state = state | STATE_HOUR;
                    dt = dt.hour(figure.unwrap() as u8);
                } else {
                    return Err(());
                }

                continue;
            } else if state & (STATE_HOUR | STATE_MINUTE | STATE_SECOND) == STATE_HOUR {
                if let Some(b':') = value.get(ptr).map(|b| *b) {
                    ptr += 1;

                    let figure = self.parse_figure(&mut ptr, value);

                    if figure.is_none() {
                        return Err(());
                    }

                    if figure.unwrap() >= 0 && figure.unwrap() < 61 {
                        state = state | STATE_MINUTE;
                        dt = dt.minute(figure.unwrap() as u8);
                    } else {
                        return Err(());
                    }
                } else {
                    return Err(());
                }

                continue;
            } else if state & (STATE_HOUR | STATE_MINUTE | STATE_SECOND)
                == (STATE_HOUR | STATE_MINUTE)
            {
                if let Some(b':') = value.get(ptr).map(|b| *b) {
                    ptr += 1;

                    let figure = self.parse_figure(&mut ptr, value);

                    if figure.is_none() {
                        return Err(());
                    }

                    if figure.unwrap() >= 0 && figure.unwrap() < 61 {
                        state = state | STATE_SECOND;
                        dt = dt.second(figure.unwrap() as u8);
                    } else {
                        return Err(());
                    }
                } else {
                    return Err(());
                }

                continue;
            } else if state & (STATE_HOUR | STATE_MINUTE | STATE_SECOND)
                == (STATE_HOUR | STATE_MINUTE | STATE_SECOND)
            {
                if let Some(b'.') = value.get(ptr).map(|b| *b) {
                    ptr += 1;

                    let fraction = self.parse_fraction(&mut ptr, value);

                    if fraction.is_none() {
                        return Err(());
                    }

                    dt = dt.fraction(fraction.unwrap());
                }

                if ptr >= value.len() {
                    break 'top;
                }

                loop {
                    match value.get(ptr).map(|b| *b) {
                        Some(b' ') | Some(b'\t') => {
                            ptr += 1;
                        }
                        _ => break,
                    };
                }

                match value.get(ptr).map(|b| *b) {
                    Some(b'z') | Some(b'Z') => {
                        ptr += 1;
                        state = state | STATE_TZ_HOUR | STATE_TZ_MINUTE;
                        dt = dt.tz_hour(0).tz_minute(0);
                    }
                    Some(b'-') => {
                        ptr += 1;
                        let figure = self.parse_figure(&mut ptr, value);
                        if figure.is_none() {
                            return Err(());
                        }
                        if figure.unwrap() >= 0 && figure.unwrap() < 25 {
                            state = state | STATE_TZ_HOUR;
                            dt = dt.tz_hour((figure.unwrap() as i8) * -1);
                        } else {
                            return Err(());
                        }
                    }
                    Some(b'+') => {
                        ptr += 1;
                        let figure = self.parse_figure(&mut ptr, value);
                        if figure.is_none() {
                            return Err(());
                        }
                        if figure.unwrap() >= 0 && figure.unwrap() < 25 {
                            state = state | STATE_TZ_HOUR;
                            dt = dt.tz_hour(figure.unwrap() as i8);
                        } else {
                            return Err(());
                        }
                    }
                    _ => (),
                };

                if state & STATE_TZ_HOUR == 0 && value.len() > ptr {
                    let figure = self.parse_figure(&mut ptr, value);

                    if figure.is_none() {
                        return Err(());
                    }

                    if figure.unwrap() >= 0 && figure.unwrap() < 25 {
                        state = state | STATE_TZ_HOUR;

                        dt = dt.tz_hour(figure.unwrap() as i8);
                    } else {
                        return Err(());
                    }

                    continue;
                }

                if state & STATE_TZ_MINUTE == 0 && value.len() > ptr {
                    match value.get(ptr).map(|b| *b) {
                        Some(b':') => {
                            ptr += 1;
                        }
                        _ => return Err(()),
                    };

                    let figure = self.parse_figure(&mut ptr, value);

                    if figure.is_none() {
                        return Err(());
                    }

                    if figure.unwrap() >= 0 && figure.unwrap() < 61 {
                        state = state | STATE_TZ_MINUTE;

                        dt = dt.tz_minute(figure.unwrap() as u8);
                    } else {
                        return Err(());
                    }

                    continue;
                }
            } else {
                return Err(());
            }

            break;
        }

        if state > 0 {
            if quote_state > 0 {
                match value.get(ptr).map(|b| *b) {
                    Some(b'\'') if quote_state == 1 => (),
                    Some(b'"') if quote_state == 2 => (),
                    _ => return Err(()),
                };
            }

            Ok(TaggedValue::from(dt))
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct TimestampValue {
    pub year: Option<i32>,
    pub month: Option<u8>,
    pub day: Option<u8>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
    pub second: Option<u8>,
    pub fraction: Option<FloatValue>,
    pub tz_hour: Option<i8>,
    pub tz_minute: Option<u8>,
}

impl TimestampValue {
    pub fn new() -> TimestampValue {
        TimestampValue {
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
            second: None,
            fraction: None,
            tz_hour: None,
            tz_minute: None,
        }
    }

    pub fn year(mut self, val: i32) -> TimestampValue {
        self.year = Some(val);
        self
    }

    pub fn month(mut self, val: u8) -> TimestampValue {
        self.month = Some(val);
        self
    }

    pub fn day(mut self, val: u8) -> TimestampValue {
        self.day = Some(val);
        self
    }

    pub fn hour(mut self, val: u8) -> TimestampValue {
        self.hour = Some(val);
        self
    }

    pub fn minute(mut self, val: u8) -> TimestampValue {
        self.minute = Some(val);
        self
    }

    pub fn second(mut self, val: u8) -> TimestampValue {
        self.second = Some(val);
        self
    }

    pub fn fraction(mut self, val: FloatValue) -> TimestampValue {
        self.fraction = Some(val);
        self
    }

    pub fn tz_hour(mut self, val: i8) -> TimestampValue {
        self.tz_hour = Some(val);
        self
    }

    pub fn tz_minute(mut self, val: u8) -> TimestampValue {
        self.tz_minute = Some(val);
        self
    }
}

impl Tagged for TimestampValue {
    fn get_tag(&self) -> Cow<'static, str> {
        Cow::from(TAG)
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}

#[cfg(all(test, not(feature = "dev")))]
mod tests {
    // TODO: tests on parsing failures

    use super::*;
    extern crate num;

    use super::fraction::Fraction;

    use crate::model::yaml::float::FloatValue;
    use crate::model::{Renderer, Tagged};

    use std::iter;

    #[test]
    fn tag() {
        let ts_coder = Timestamp; // ::new (&get_charset_utf8 ());

        assert_eq!(ts_coder.get_tag(), TAG);
    }

    macro_rules! encoded_dt_is {
        ($coder:expr, $dt:expr, $str:expr) => {{
            let renderer = Renderer; // ::new (&get_charset_utf8 ());
            if let Ok(rope) = $coder.encode(&renderer, TaggedValue::from($dt), &mut iter::empty()) {
                let encoded = rope.render(&renderer);
                assert_eq!($str.to_string().into_bytes(), encoded);
            } else {
                assert!(false)
            }
        }};
    }

    #[test]
    fn encode() {
        let ts_coder = Timestamp; // ::new (&get_charset_utf8 ());

        encoded_dt_is!(
            ts_coder,
            TimestampValue::new().year(2016).month(1).day(16),
            "2016-01-16"
        );
        encoded_dt_is!(
            ts_coder,
            TimestampValue::new().hour(18).minute(58).second(3),
            "18:58:03"
        );

        let dt = TimestampValue::new()
            .year(2016)
            .month(1)
            .day(16)
            .hour(18)
            .minute(58)
            .second(3);
        encoded_dt_is!(ts_coder, dt.clone(), "2016-01-16T18:58:03");

        let dt = dt
            .fraction(FloatValue::from(Fraction::new(25u8, 100u8)))
            .tz_hour(12);
        encoded_dt_is!(ts_coder, dt.clone(), "2016-01-16T18:58:03.25+12");

        let dt = dt.tz_hour(-2);
        encoded_dt_is!(ts_coder, dt.clone(), "2016-01-16T18:58:03.25-02");

        let dt = dt.tz_minute(25);
        encoded_dt_is!(ts_coder, dt, "2016-01-16T18:58:03.25-02:25");
    }

    #[test]
    fn decode() {
        let ts_coder = Timestamp; // ::new (&get_charset_utf8 ());

        if let Ok(tagged) = ts_coder.decode(true, "2016-01-16".as_bytes()) {
            assert_eq!(tagged.get_tag(), Cow::from(TAG));

            if let Some(decoded) = tagged.as_any().downcast_ref::<TimestampValue>() {
                assert!(decoded.year.is_some());
                assert_eq!(decoded.year.unwrap(), 2016);

                assert!(decoded.month.is_some());
                assert_eq!(decoded.month.unwrap(), 1);

                assert!(decoded.day.is_some());
                assert_eq!(decoded.day.unwrap(), 16);

                assert!(decoded.hour.is_none());
                assert!(decoded.minute.is_none());
                assert!(decoded.second.is_none());
                assert!(decoded.fraction.is_none());
                assert!(decoded.tz_hour.is_none());
                assert!(decoded.tz_minute.is_none());
            } else {
                assert!(false)
            }
        } else {
            assert!(false)
        }

        if let Ok(tagged) = ts_coder.decode(true, "23:59:11".as_bytes()) {
            assert_eq!(tagged.get_tag(), Cow::from(TAG));

            if let Some(decoded) = tagged.as_any().downcast_ref::<TimestampValue>() {
                assert!(decoded.year.is_none());
                assert!(decoded.month.is_none());
                assert!(decoded.day.is_none());

                assert!(decoded.hour.is_some());
                assert_eq!(decoded.hour.unwrap(), 23);

                assert!(decoded.minute.is_some());
                assert_eq!(decoded.minute.unwrap(), 59);

                assert!(decoded.second.is_some());
                assert_eq!(decoded.second.unwrap(), 11);

                assert!(decoded.fraction.is_none());
                assert!(decoded.tz_hour.is_none());
                assert!(decoded.tz_minute.is_none());
            } else {
                assert!(false)
            }
        } else {
            assert!(false)
        }

        if let Ok(tagged) = ts_coder.decode(true, "2016-06-03T23:59:11.0045-12:25".as_bytes()) {
            assert_eq!(tagged.get_tag(), Cow::from(TAG));

            if let Some(decoded) = tagged.as_any().downcast_ref::<TimestampValue>() {
                assert!(decoded.year.is_some());
                assert_eq!(decoded.year.unwrap(), 2016);

                assert!(decoded.month.is_some());
                assert_eq!(decoded.month.unwrap(), 6);

                assert!(decoded.day.is_some());
                assert_eq!(decoded.day.unwrap(), 3);

                assert!(decoded.hour.is_some());
                assert_eq!(decoded.hour.unwrap(), 23);

                assert!(decoded.minute.is_some());
                assert_eq!(decoded.minute.unwrap(), 59);

                assert!(decoded.second.is_some());
                assert_eq!(decoded.second.unwrap(), 11);

                assert!(decoded.fraction.is_some());
                let frac = decoded.fraction.as_ref().unwrap(); // 0.0045 == 9/2000
                assert_eq!(
                    format!("{:.4}", Fraction::new(9u8, 2000u16)),
                    frac.format_as_float().unwrap()
                );

                assert!(decoded.tz_hour.is_some());
                assert_eq!(decoded.tz_hour.unwrap(), -12);

                assert!(decoded.tz_minute.is_some());
                assert_eq!(decoded.tz_minute.unwrap(), 25);
            } else {
                assert!(false)
            }
        } else {
            assert!(false)
        }
    }
}
