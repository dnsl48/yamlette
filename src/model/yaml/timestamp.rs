extern crate fraction;
extern crate skimmer;
extern crate num;


use self::fraction::{ Fraction, BigFraction };
use self::skimmer::symbol::{ Char, Symbol };
use self::num::BigUint;


use txt::{ CharSet, Encoding, Twine };

use model::{ EncodedString, Factory, Model, Node, Rope, Renderer, Tagged, TaggedValue };

use model::yaml::float::FloatValue;

use std::any::Any;
use std::i32;
use std::iter::Iterator;




pub const TAG: &'static str = "tag:yaml.org,2002:timestamp";
static TWINE_TAG: Twine = Twine::Static (TAG);




pub struct Timestamp {
    encoding: Encoding,

    dgt: [Char; 10],
    colon: Char,
    minus: Char,
    dot: Char,
    letter_t: Char,
    letter_t_t: Char,
    letter_z: Char,
    letter_t_z: Char,
    plus: Char,
    space: Char,
    tab: Char,

    s_quote: Char,
    d_quote: Char,

    chr_len: usize
}



impl Timestamp {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }


    pub fn new (cset: &CharSet) -> Timestamp {
        let chars = [&cset.colon, &cset.hyphen_minus, &cset.full_stop, &cset.letter_t, &cset.letter_t_t, &cset.letter_z,
                     &cset.letter_t_z, &cset.plus, &cset.space, &cset.tab_h, &cset.digit_0, &cset.digit_1,
                     &cset.digit_2, &cset.digit_3, &cset.digit_4, &cset.digit_5, &cset.digit_6, &cset.digit_7,
                     &cset.digit_8, &cset.digit_9];

        let mut char_len = 1;

        for i in 0 .. chars.len () {
            if chars[i].len () > char_len { char_len = chars[i].len (); }
        }

        Timestamp {
            encoding: cset.encoding,

            dgt: [
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
            minus: cset.hyphen_minus.clone (),
            dot: cset.full_stop.clone (),
            letter_t: cset.letter_t.clone (),
            letter_t_t: cset.letter_t_t.clone (),
            letter_z: cset.letter_z.clone (),
            letter_t_z: cset.letter_t_z.clone (),
            plus: cset.plus.clone (),
            space: cset.space.clone (),
            tab: cset.tab_h.clone (),

            s_quote: cset.apostrophe.clone (),
            d_quote: cset.quotation.clone (),

            chr_len: char_len
        }
    }


    fn parse_figure (&self, ptr: &mut usize, value: &[u8]) -> Option<i64> {
        let mut figure: Option<i64> = None;

        'figure: loop {
            for i in 0 .. 10 {
                if self.dgt[i].contained_at (value, *ptr) {
                    figure = if let Some (nval) = (if figure.is_some () { figure.unwrap () } else { 0 }).checked_mul (10) {
                        if let Some (nval) = nval.checked_add (i as i64) {
                            *ptr += self.dgt[i].len ();
                            Some (nval)
                        } else { return None }
                    } else { return None };
                    continue 'figure;
                }
            }
            break;
        }

        figure
    }


    fn parse_fraction (&self, ptr: &mut usize, value: &[u8]) -> Option<FloatValue> {
        let mut fraction: Option<(Result<u64, BigUint>, Result<u64, BigUint>)> = None;

        'fraction: loop {
            for i in 0 .. 10 {
                if self.dgt[i].contained_at (value, *ptr) {
                    *ptr += self.dgt[i].len ();

                    let mut f = if fraction.is_some () { fraction.unwrap () } else { (Ok (0), Ok (1)) };

                    f.0 = match f.0 {
                        Ok (u) => {
                            if let Some (nval) = u.checked_mul (10) {
                                if let Some (nval) = nval.checked_add (i as u64) {
                                    Ok (nval)
                                } else {
                                    Err (BigUint::from (nval) + BigUint::from (i as u64))
                                }
                            } else {
                                Err (BigUint::from (u) * BigUint::from (10u8) + BigUint::from (i as u64))
                            }
                        }
                        Err (b) => { Err (b * BigUint::from (10u8) + BigUint::from (i as u64)) }
                    };

                    f.1 = match f.1 {
                        Ok (u) => if let Some (nval) = u.checked_mul (10) {
                            Ok (nval)
                        } else {
                            Err (BigUint::from (u) * BigUint::from (10u8))
                        },
                        Err (b) => { Err (b * BigUint::from (10u8)) }
                    };

                    fraction = Some ((f.0, f.1));

                    continue 'fraction;
                }
            }
            break;
        }

        if let Some ( (num, den) ) = fraction {
            if num.is_ok () && den.is_ok () {
                Some (FloatValue::from (Fraction::new (num.ok ().unwrap (), den.ok ().unwrap ())))
            } else if num.is_ok () {
                Some (FloatValue::from (BigFraction::new (BigUint::from (num.ok ().unwrap ()), den.err ().unwrap ())))
            } else if den.is_ok () {
                Some (FloatValue::from (BigFraction::new (num.err ().unwrap (), BigUint::from (den.ok ().unwrap ()))))
            } else {
                Some (FloatValue::from (BigFraction::new (num.err ().unwrap (), den.err ().unwrap ())))
            }
        } else { None }
    }
}



impl Model for Timestamp {
    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

    fn get_encoding (&self) -> Encoding { self.encoding }

    fn is_decodable (&self) -> bool { true }

    fn is_encodable (&self) -> bool { true }


    fn encode (&self, _renderer: &Renderer, value: TaggedValue, _tags: &mut Iterator<Item=&(Twine, Twine)>) -> Result<Rope, TaggedValue> {
        let value: TimestampValue = match <TaggedValue as Into<Result<TimestampValue, TaggedValue>>>::into (value) {
            Ok (value) => value,
            Err (value) => return Err (value)
        };

        let mut src = String::with_capacity (32);

        if value.year.is_some () && value.month.is_some () && value.day.is_some () {
            src.push_str (&format! ("{:04}-{:02}-{:02}", value.year.as_ref ().unwrap (), value.month.as_ref ().unwrap (), value.day.as_ref ().unwrap ()));
        }

        if value.hour.is_some () || value.minute.is_some () || value.second.is_some () {
            if src.len () > 0 { src.push_str ("T") };

            src.push_str (&format! (
                "{:02}:{:02}:{:02}",
                if let Some (h) = value.hour.as_ref () { *h } else { 0 },
                if let Some (m) = value.minute.as_ref () { *m } else { 0 },
                if let Some (s) = value.second.as_ref () { *s } else { 0 }
            ));

            let fi = if let Some (f) = value.fraction.as_ref () {
                let f = f.clone ().format_as_float ();
                if let Some (f) = f {
                    src.push_str (&f[1 ..]);
                    true
                } else { false }
            } else { true };

            if !fi { return Err ( TaggedValue::from (value) ) }

            if let Some (h) = value.tz_hour.as_ref () {
                if *h > 0 {
                    src.push_str (&format! ("{:+02}", h));
                } else {
                    src.push_str (&format! ("{:+03}", h));
                }

                if let Some (ref m) = value.tz_minute.as_ref () {
                    src.push_str (&format! (":{:02}", m));
                }
            }
        }

        let mut production: Vec<u8> = Vec::with_capacity (src.len () * self.chr_len);

        for chr in src.as_bytes () {
            let symbol = match *chr {
                b'0' => &self.dgt[0],
                b'1' => &self.dgt[1],
                b'2' => &self.dgt[2],
                b'3' => &self.dgt[3],
                b'4' => &self.dgt[4],
                b'5' => &self.dgt[5],
                b'6' => &self.dgt[6],
                b'7' => &self.dgt[7],
                b'8' => &self.dgt[8],
                b'9' => &self.dgt[9],

                b'-' => &self.minus,
                b':' => &self.colon,
                b'T' => &self.letter_t_t,
                b'.' => &self.dot,
                b'+' => &self.plus,

                _ => unreachable! ()
            };

            production.extend (symbol.as_slice ());
        }

        Ok (Rope::from (Node::String (EncodedString::from (production))))
    }


    fn decode (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let mut ptr: usize = 0;
        let vlen: usize = value.len ();

        let mut state: u8 = 0;

        const STATE_YEAR: u8 = 1;
        const STATE_MONTH: u8 = 2;
        const STATE_DAY: u8 = 4;
        const STATE_HOUR: u8 = 8;
        const STATE_MINUTE: u8 = 16;
        const STATE_SECOND: u8 = 32;
        const STATE_TZ_HOUR: u8 = 64;
        const STATE_TZ_MINUTE: u8 = 128;


        let mut dt = TimestampValue::new ();

        let mut quote_state = 0; // 1 - single, 2 - double


        'top: loop {
            if ptr >= vlen { break; }


            if explicit && ptr == 0 && quote_state == 0 {
                if self.s_quote.contained_at (value, ptr) {
                    ptr += self.s_quote.len ();
                    quote_state = 1;
                    continue;
                }
            }


            if explicit && ptr == 0 && quote_state == 0 {
                if self.d_quote.contained_at (value, ptr) {
                    ptr += self.d_quote.len ();
                    quote_state = 2;
                    continue;
                }
            }


            if state == 0 {
                let ltz = if self.minus.contained_at (value, ptr) { true } else { false };

                let figure = self.parse_figure (&mut ptr, value);

                if figure.is_none () { return Err ( () ) }

                if !ltz && figure.unwrap () >= 0 && figure.unwrap () < 25 {
                    if self.colon.contained_at (value, ptr) {
                        state = STATE_HOUR;
                        dt = dt.hour (figure.unwrap () as u8);
                    }
                }

                if state == 0 && figure.unwrap () >= (i32::MIN as i64) && figure.unwrap () <= (i32::MAX as i64) {
                    state = STATE_YEAR;
                    dt = dt.year ((figure.unwrap () * if ltz { -1 } else { 1 }) as i32);
                }

                continue;

            } else if state == STATE_YEAR {
                if self.minus.contained_at (value, ptr) {
                    ptr += self.minus.len ();

                    let figure = self.parse_figure (&mut ptr, value);

                    if figure.is_none () { return Err ( () ) }

                    if figure.unwrap () > 0 && figure.unwrap () < 13 {
                        state = state | STATE_MONTH;
                        dt = dt.month (figure.unwrap () as u8);
                    } else { return Err ( () ) }
                } else { return Err ( () ) }

                continue;

            } else if state == STATE_YEAR | STATE_MONTH {
                if self.minus.contained_at (value, ptr) {
                    ptr += self.minus.len ();

                    let figure = self.parse_figure (&mut ptr, value);

                    if figure.is_none () { return Err ( () ) }

                    if figure.unwrap () > 0 && figure.unwrap () < 32 {
                        state = state | STATE_DAY;
                        dt = dt.day (figure.unwrap () as u8);
                    } else { return Err ( () ) }
                } else { return Err ( () ) }

                continue;

            } else if state == STATE_YEAR | STATE_MONTH | STATE_DAY {
                if self.letter_t_t.contained_at (value, ptr) { ptr += self.letter_t_t.len (); }
                else if self.space.contained_at (value, ptr) { ptr += self.space.len (); }
                else if self.letter_t.contained_at (value, ptr) { ptr += self.letter_t.len (); }
                else if self.tab.contained_at (value, ptr) { ptr += self.tab.len (); }
                else { return Err ( () ) };


                let figure = self.parse_figure (&mut ptr, value);

                if figure.is_none () { return Err ( () ) }

                if figure.unwrap () >= 0 && figure.unwrap () < 25 {
                    state = state | STATE_HOUR;
                    dt = dt.hour (figure.unwrap () as u8);
                } else { return Err ( () ) }

                continue;

            } else if state & (STATE_HOUR | STATE_MINUTE | STATE_SECOND) == STATE_HOUR {
                if self.colon.contained_at (value, ptr) {
                    ptr += self.colon.len ();

                    let figure = self.parse_figure (&mut ptr, value);

                    if figure.is_none () { return Err ( () ) }

                    if figure.unwrap () >=0 && figure.unwrap () < 61 {
                        state = state | STATE_MINUTE;
                        dt = dt.minute (figure.unwrap () as u8);
                    } else { return Err ( () ) }
                } else { return Err ( () ) }

                continue;

            } else if state & (STATE_HOUR | STATE_MINUTE | STATE_SECOND) == (STATE_HOUR | STATE_MINUTE) {
                if self.colon.contained_at (value, ptr) {
                    ptr += self.colon.len ();

                    let figure = self.parse_figure (&mut ptr, value);

                    if figure.is_none () { return Err ( () ) }

                    if figure.unwrap () >=0 && figure.unwrap () < 61 {
                        state = state | STATE_SECOND;
                        dt = dt.second (figure.unwrap () as u8);
                    } else { return Err ( () ) }
                } else { return Err ( () ) }

                continue;

            } else if state & (STATE_HOUR | STATE_MINUTE | STATE_SECOND) == (STATE_HOUR | STATE_MINUTE | STATE_SECOND) {
                if self.dot.contained_at (value, ptr) {
                    ptr += self.dot.len ();

                    let fraction = self.parse_fraction (&mut ptr, value);

                    if fraction.is_none () { return Err ( () ) }

                    dt = dt.fraction (fraction.unwrap ());
                }

                if ptr >= vlen { break 'top; }

                loop {
                    if self.space.contained_at (value, ptr) {
                        ptr += self.space.len ();
                    } else if self.tab.contained_at (value, ptr) {
                        ptr += self.tab.len ();
                    } else { break; }
                }

                if self.letter_t_z.contained_at (value, ptr) {
                    ptr += self.letter_t_z.len ();

                    state = state | STATE_TZ_HOUR | STATE_TZ_MINUTE;

                    dt = dt.tz_hour (0).tz_minute (0);
                } else if self.letter_z.contained_at (value, ptr) {
                    ptr += self.letter_z.len ();

                    state = state | STATE_TZ_HOUR | STATE_TZ_MINUTE;

                    dt = dt.tz_hour (0).tz_minute (0);
                } else if self.minus.contained_at (value, ptr) {
                    ptr += self.minus.len ();

                    let figure = self.parse_figure (&mut ptr, value);

                    if figure.is_none () { return Err ( () ) }

                    if figure.unwrap () >= 0 && figure.unwrap () < 25 {
                        state = state | STATE_TZ_HOUR;

                        dt = dt.tz_hour ((figure.unwrap () as i8) * -1);
                    } else { return Err ( () ) }

                } else if self.plus.contained_at (value, ptr) {
                    ptr += self.plus.len ();

                    let figure = self.parse_figure (&mut ptr, value);

                    if figure.is_none () { return Err ( () ) }

                    if figure.unwrap () >= 0 && figure.unwrap () < 25 {
                        state = state | STATE_TZ_HOUR;

                        dt = dt.tz_hour (figure.unwrap () as i8);
                    } else { return Err ( () ) }

                }

                if state & STATE_TZ_HOUR == 0 && vlen > ptr {
                    let figure = self.parse_figure (&mut ptr, value);

                    if figure.is_none () { return Err ( () ) }

                    if figure.unwrap () >= 0 && figure.unwrap () < 25 {
                        state = state | STATE_TZ_HOUR;

                        dt = dt.tz_hour (figure.unwrap () as i8);
                    } else { return Err ( () ) }

                    continue;
                }

                if state & STATE_TZ_MINUTE == 0 && vlen > ptr {
                    if self.colon.contained_at (value, ptr) {
                        ptr += self.colon.len ();
                    } else { return Err ( () ) }

                    let figure = self.parse_figure (&mut ptr, value);

                    if figure.is_none () { return Err ( () ) }

                    if figure.unwrap () >= 0 && figure.unwrap () < 61 {
                        state = state | STATE_TZ_MINUTE;

                        dt = dt.tz_minute (figure.unwrap () as u8);
                    } else { return Err ( () ) }

                    continue;
                }

            } else { return Err ( () ) }

            break;
        }


        if state > 0 {
            if quote_state > 0 {
                if quote_state == 1 && self.s_quote.contained_at (value, ptr) {
                    // pass
                } else if quote_state == 2 && self.d_quote.contained_at (value, ptr) {
                    // pass
                } else { return Err ( () ) }
            }

            Ok ( TaggedValue::from (dt) )
        } else {
            Err ( () )
        }
    }
}




#[derive (Clone, Debug)]
pub struct TimestampValue {
    pub year: Option<i32>,
    pub month: Option<u8>,
    pub day: Option<u8>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
    pub second: Option<u8>,
    pub fraction: Option<FloatValue>,
    pub tz_hour: Option<i8>,
    pub tz_minute: Option<u8>
}



impl TimestampValue {
    pub fn new () -> TimestampValue {
        TimestampValue {
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
            second: None,
            fraction: None,
            tz_hour: None,
            tz_minute: None
        }
    }


    pub fn year (mut self, val: i32) -> TimestampValue {
        self.year = Some (val);
        self
    }


    pub fn month (mut self, val: u8) -> TimestampValue {
        self.month = Some (val);
        self
    }


    pub fn day (mut self, val: u8) -> TimestampValue {
        self.day = Some (val);
        self
    }


    pub fn hour (mut self, val: u8) -> TimestampValue {
        self.hour = Some (val);
        self
    }


    pub fn minute (mut self, val: u8) -> TimestampValue {
        self.minute = Some (val);
        self
    }


    pub fn second (mut self, val: u8) -> TimestampValue {
        self.second = Some (val);
        self
    }


    pub fn fraction (mut self, val: FloatValue) -> TimestampValue {
        self.fraction = Some (val);
        self
    }


    pub fn tz_hour (mut self, val: i8) -> TimestampValue {
        self.tz_hour = Some (val);
        self
    }


    pub fn tz_minute (mut self, val: u8) -> TimestampValue {
        self.tz_minute = Some (val);
        self
    }
}



impl Tagged for TimestampValue {
    fn get_tag (&self) -> &Twine { Timestamp::get_tag () }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}




pub struct TimestampFactory;



impl Factory for TimestampFactory {
    fn get_tag (&self) -> &Twine { Timestamp::get_tag () }

    fn build_model (&self, cset: &CharSet) -> Box<Model> { Box::new (Timestamp::new (cset)) }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    // TODO: tests on parsing failures

    use super::*;
    extern crate num;

    use super::fraction::Fraction;

    use model::{ Factory, Tagged, Renderer };
    use model::yaml::float::FloatValue;
    use txt::get_charset_utf8;

    use std::iter;



    #[test]
    fn tag () {
        let ts_coder = TimestampFactory.build_model (&get_charset_utf8 ());

        assert_eq! (ts_coder.get_tag (), TAG);
    }


    macro_rules! encoded_dt_is {
        ($coder:expr, $dt:expr, $str:expr) => {{
            let renderer = Renderer::new (&get_charset_utf8 ());
            if let Ok (rope) = $coder.encode (&renderer, TaggedValue::from ($dt), &mut iter::empty ()) {
                let encoded = rope.render (&renderer);
                assert_eq! ($str.to_string ().into_bytes (), encoded);
            } else { assert! (false) }
        }}
    }



    #[test]
    fn encode () {
        let ts_coder = Timestamp::new (&get_charset_utf8 ());

        encoded_dt_is! (ts_coder, TimestampValue::new ().year (2016).month (1).day (16), "2016-01-16");
        encoded_dt_is! (ts_coder, TimestampValue::new ().hour (18).minute (58).second (3), "18:58:03");

        let dt = TimestampValue::new ().year (2016).month (1).day (16).hour (18).minute (58).second (3);
        encoded_dt_is! (ts_coder, dt.clone (), "2016-01-16T18:58:03");

        let dt = dt.fraction (FloatValue::from (Fraction::new (25u8, 100u8))).tz_hour (12);
        encoded_dt_is! (ts_coder, dt.clone (), "2016-01-16T18:58:03.25+12");

        let dt = dt.tz_hour (-2);
        encoded_dt_is! (ts_coder, dt.clone (), "2016-01-16T18:58:03.25-02");

        let dt = dt.tz_minute (25);
        encoded_dt_is! (ts_coder, dt, "2016-01-16T18:58:03.25-02:25");
    }



    #[test]
    fn decode () {
        let ts_coder = Timestamp::new (&get_charset_utf8 ());

        if let Ok (tagged) = ts_coder.decode (true, "2016-01-16".as_bytes ()) {
            assert_eq! (tagged.get_tag (), Timestamp::get_tag ());

            if let Some (decoded) = tagged.as_any ().downcast_ref::<TimestampValue> () {
                assert! (decoded.year.is_some ());
                assert_eq! (decoded.year.unwrap (), 2016);

                assert! (decoded.month.is_some ());
                assert_eq! (decoded.month.unwrap (), 1);

                assert! (decoded.day.is_some ());
                assert_eq! (decoded.day.unwrap (), 16);

                assert! (decoded.hour.is_none ());
                assert! (decoded.minute.is_none ());
                assert! (decoded.second.is_none ());
                assert! (decoded.fraction.is_none ());
                assert! (decoded.tz_hour.is_none ());
                assert! (decoded.tz_minute.is_none ());
            } else { assert! (false) }
        } else { assert! (false) }


        if let Ok (tagged) = ts_coder.decode (true, "23:59:11".as_bytes ())  {
            assert_eq! (tagged.get_tag (), Timestamp::get_tag ());

            if let Some (decoded) = tagged.as_any ().downcast_ref::<TimestampValue> () {
                assert! (decoded.year.is_none ());
                assert! (decoded.month.is_none ());
                assert! (decoded.day.is_none ());

                assert! (decoded.hour.is_some ());
                assert_eq! (decoded.hour.unwrap (), 23);

                assert! (decoded.minute.is_some ());
                assert_eq! (decoded.minute.unwrap (), 59);

                assert! (decoded.second.is_some ());
                assert_eq! (decoded.second.unwrap (), 11);

                assert! (decoded.fraction.is_none ());
                assert! (decoded.tz_hour.is_none ());
                assert! (decoded.tz_minute.is_none ());
            } else { assert! (false) }
        } else { assert! (false) }


        if let Ok (tagged) = ts_coder.decode (true, "2016-06-03T23:59:11.0045-12:25".as_bytes ())  {
            assert_eq! (tagged.get_tag (), Timestamp::get_tag ());

            if let Some (decoded) = tagged.as_any ().downcast_ref::<TimestampValue> () {
                assert! (decoded.year.is_some ());
                assert_eq! (decoded.year.unwrap (), 2016);

                assert! (decoded.month.is_some ());
                assert_eq! (decoded.month.unwrap (), 6);

                assert! (decoded.day.is_some ());
                assert_eq! (decoded.day.unwrap (), 3);

                assert! (decoded.hour.is_some ());
                assert_eq! (decoded.hour.unwrap (), 23);

                assert! (decoded.minute.is_some ());
                assert_eq! (decoded.minute.unwrap (), 59);

                assert! (decoded.second.is_some ());
                assert_eq! (decoded.second.unwrap (), 11);

                assert! (decoded.fraction.is_some ());
                let frac = decoded.fraction.as_ref ().unwrap (); // 0.0045 == 9/2000
                assert_eq! (Fraction::new (9u8, 2000u16).format_as_float (), frac.format_as_float ());

                assert! (decoded.tz_hour.is_some ());
                assert_eq! (decoded.tz_hour.unwrap (), -12);

                assert! (decoded.tz_minute.is_some ());
                assert_eq! (decoded.tz_minute.unwrap (), 25);
            } else { assert! (false) }
        } else { assert! (false) }
    }
}
