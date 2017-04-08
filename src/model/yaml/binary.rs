extern crate skimmer;

use txt::Twine;

use model::{ model_issue_rope, EncodedString, Model, Node, Rope, Renderer, Tagged, TaggedValue };
use model::style::CommonStyles;

use std::any::Any;
use std::iter::Iterator;
// use std::marker::PhantomData;



pub const TAG: &'static str = "tag:yaml.org,2002:binary";
static TWINE_TAG: Twine = Twine::Static (TAG);




#[derive (Clone, Copy)]
pub struct Binary; /* {
    // tbl: [Char; 64],
    // pad: Char,
    // line_feed: Char,
    // carriage_return: Char,
    // space: Char,
    // tab_h: Char,

    // s_quote: Char,
    // d_quote: Char,

    // tcl: usize,

    encoding: Encoding,

    // _dchr: PhantomData<DoubleChar>
}
*/



impl Binary {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }

    fn tbl (byte: u8) -> u8 {
        match byte {
            0  ... 25 => byte + b'A',
            26 ... 51 => byte + b'a' - 26,
            52 ... 61 => byte + b'0' - 52,
            62        => b'+',
            63        => b'/',
            _ => unreachable! ()
        }
    }

    fn lbt (byte: u8) -> u8 {
        match byte {
            v @ b'A' ... b'Z' => v - b'A',
            v @ b'a' ... b'z' => v - b'a' + 26,
            v @ b'0' ... b'9' => v - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => unreachable! ()
        }
    }

/*
    pub fn new (cset: &CharSet<Char, DoubleChar>) -> Binary<Char, DoubleChar> {
        let pad = cset.equal;

        let tbl = [
            cset.letter_t_a,
            cset.letter_t_b,
            cset.letter_t_c,
            cset.letter_t_d,
            cset.letter_t_e,
            cset.letter_t_f,
            cset.letter_t_g,
            cset.letter_t_h,
            cset.letter_t_i,
            cset.letter_t_j,
            cset.letter_t_k,
            cset.letter_t_l,
            cset.letter_t_m,
            cset.letter_t_n,
            cset.letter_t_o,
            cset.letter_t_p,
            cset.letter_t_q,
            cset.letter_t_r,
            cset.letter_t_s,
            cset.letter_t_t,
            cset.letter_t_u,
            cset.letter_t_v,
            cset.letter_t_w,
            cset.letter_t_x,
            cset.letter_t_y,
            cset.letter_t_z,

            cset.letter_a,
            cset.letter_b,
            cset.letter_c,
            cset.letter_d,
            cset.letter_e,
            cset.letter_f,
            cset.letter_g,
            cset.letter_h,
            cset.letter_i,
            cset.letter_j,
            cset.letter_k,
            cset.letter_l,
            cset.letter_m,
            cset.letter_n,
            cset.letter_o,
            cset.letter_p,
            cset.letter_q,
            cset.letter_r,
            cset.letter_s,
            cset.letter_t,
            cset.letter_u,
            cset.letter_v,
            cset.letter_w,
            cset.letter_x,
            cset.letter_y,
            cset.letter_z,
            
            cset.digit_0,
            cset.digit_1,
            cset.digit_2,
            cset.digit_3,
            cset.digit_4,
            cset.digit_5,
            cset.digit_6,
            cset.digit_7,
            cset.digit_8,
            cset.digit_9,

            cset.plus,
            cset.slash
        ];

        Binary {
            encoding: cset.encoding,

            line_feed: cset.line_feed,
            carriage_return: cset.carriage_return,
            space: cset.space,
            tab_h: cset.tab_h,

            s_quote: cset.apostrophe,
            d_quote: cset.quotation,

            tbl: tbl,
            pad: pad,
            tcl: cset.longest_char,

            _dchr: PhantomData
        }
    }
*/
}



impl Model for Binary {
    fn get_tag (&self) -> &Twine { &TWINE_TAG }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }


    fn is_decodable (&self) -> bool { true }

    fn is_encodable (&self) -> bool { true }


    fn encode (&self, _renderer: &Renderer, value: TaggedValue, tags: &mut Iterator<Item=&(Twine, Twine)>) -> Result<Rope, TaggedValue> {
        let mut value: BinaryValue = match <TaggedValue as Into<Result<BinaryValue, TaggedValue>>>::into (value) {
            Ok (value) => value,
            Err (value) => return Err (value)
        };

        let issue_tag = value.issue_tag ();
        let alias = value.take_alias ();
        let value = value.to_vec ();

        let res_len = (value.len () + if value.len () % 3 > 0 { 3 - (value.len () % 3) } else { 0 }) / 3 * 4;

        let mut production: Vec<u8> = Vec::with_capacity (res_len);

        let mut rem: u8 = 0;

        for b in value {
            let b = b.to_be ();

            /*
            let idx = if rem & 0b1000_0000 == 0b1000_0000 {
                let idx = ((rem & 0b0000_1111) << 2) | (b >> 6);
                production.extend (self.tbl[idx as usize].as_slice ());
                rem = 0;
                b & 0b0011_1111
            } else if rem & 0b0100_0000 == 0b0100_0000 {
                let idx = ((rem & 0b0000_0011) << 4) | (b >> 4);
                rem = 0b1000_0000 | (b & 0b0000_1111);
                idx
            } else {
                rem = 0b0100_0000 | (b & 0b0000_0011);

                b >> 2
            };
             */
            let idx = if rem & 0b1000_0000 == 0b1000_0000 {
                let idx = ((rem & 0b0000_1111) << 2) | (b >> 6);
                production.push (Self::tbl(idx));
                rem = 0;
                b & 0b0011_1111
            } else if rem & 0b0100_0000 == 0b0100_0000 {
                let idx = ((rem & 0b0000_0011) << 4) | (b >> 4);
                rem = 0b1000_0000 | (b & 0b0000_1111);
                idx
            } else {
                rem = 0b0100_0000 | (b & 0b0000_0011);

                b >> 2
            };

            production.push (Self::tbl(idx));
        }

        for _ in 0 .. (production.capacity () - production.len ()) {
            if rem > 0 {
                let idx = if rem & 0b1000_0000 == 0b1000_0000 {
                    (rem & 0b0000_1111) << 2
                } else if rem & 0b0100_0000 == 0b0100_0000 {
                    (rem & 0b0000_0011) << 4
                } else { 0 };

                rem = 0;

                production.push (Self::tbl(idx));
            } else {
                production.push (b'=');
            }
        }

        let node = Node::String (EncodedString::from (production));

        Ok (model_issue_rope (self, node, issue_tag, alias, tags))
    }


    fn decode (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        // let vlen = value.len ();

        let mut production: Vec<u8> = Vec::with_capacity (value.len () / 4 * 3);

        // if vlen % (4 * self.tcl) != 0 { return Err ( () ) } // TODO: warning?

        let mut rem: u8 = 0;
        let mut ptr: usize = 0;

        let mut quote_state = 0; // 1 - single, 2 - double, 3 - finished


        if explicit && quote_state == 0 {
            match value.get (ptr).map (|b| *b) {
                Some (b'\'') => { ptr += 1; quote_state = 1; }
                Some (b'"')  => { ptr += 1; quote_state = 2; }
                _ => ()
            };
        }


        loop {
            match value.get (ptr).map (|b| *b) {
                None => break,

                Some (b'\'') => {
                    if quote_state == 1 {
                        ptr += 1;
                        quote_state = 3;
                        break;
                    } else { return Err ( () ) }
                }

                Some (b'"') => {
                    if quote_state == 2 {
                        ptr += 1;
                        quote_state = 3;
                        break;
                    } else { return Err ( () ) }
                }

                Some (v @ b'a' ... b'z') |
                Some (v @ b'A' ... b'Z') |
                Some (v @ b'0' ... b'9') |
                Some (v @ b'+') |
                Some (v @ b'/') => {
                    ptr += 1;
                    let i = Self::lbt (v);

                    let idx: u8 = if rem & 0b1100_0000 == 0b1100_0000 {
                        let idx = ((rem & 0b0011_1111) << 2) | (i >> 4);
                        rem = (i & 0b0000_1111) | 0b1000_0000;
                        idx
                    } else if rem & 0b1000_0000 == 0b1000_0000 {
                        let idx = ((rem & 0b0011_1111) << 4) | (i >> 2);
                        rem = (i & 0b0000_0011) | 0b0100_0000;
                        idx
                    } else if rem & 0b0100_0000 == 0b0100_0000 {
                        let idx = ((rem & 0b0011_1111) << 6) | i;
                        rem = 0;
                        idx
                    } else {
                        rem = i | 0b1100_0000;
                        continue;
                    };

                    production.push (idx);
                }

                Some (b'=') => {
                    ptr += 1;

                    let i: u8 = 0;

                    let idx: u8 = if rem & 0b1100_0000 == 0b1100_0000 {
                        let idx = ((rem & 0b0011_1111) << 2) | (i >> 4);
                        rem = (i & 0b0000_1111) | 0b1000_0000;
                        idx
                    } else if rem & 0b1000_0000 == 0b1000_0000 {
                        let idx = ((rem & 0b0011_1111) << 4) | (i >> 2);
                        rem = (i & 0b0000_0011) | 0b0100_0000;
                        idx
                    } else if rem & 0b0100_0000 == 0b0100_0000 {
                        let idx = ((rem & 0b0011_1111) << 6) | i;
                        rem = 0;
                        idx
                    } else {
                        rem = i | 0b1100_0000;
                        continue;
                    };

                    if idx == 0 { break }

                    production.push (idx);
                }

                Some (b' ') |
                Some (b'\t') |
                Some (b'\n') |
                Some (b'\r') => { ptr += 1; }

                _ => return Err ( () )
            }
        }

        if quote_state == 3 {
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
        }

        Ok ( TaggedValue::from (BinaryValue::from (production)) )
    }
}




#[derive (Clone, Debug)]
pub struct BinaryValue {
    style: u8,
    alias: Option<Twine>,
    value: Vec<u8>
}



impl BinaryValue {
    pub fn new (value: Vec<u8>, styles: CommonStyles, alias: Option<Twine>) -> BinaryValue { BinaryValue {
        style: if styles.issue_tag () { 1 } else { 0 },
        value: value,
        alias: alias
    } }

    pub fn to_vec (self) -> Vec<u8> { self.value }

    pub fn take_alias (&mut self) -> Option<Twine> { self.alias.take () }

    pub fn issue_tag (&self) -> bool { self.style & 1 == 1 }

    pub fn set_issue_tag (&mut self, val: bool) { if val { self.style |= 1; } else { self.style &= !1; } }
}



impl Tagged for BinaryValue {
    fn get_tag (&self) -> &Twine { &TWINE_TAG }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



impl From<Vec<u8>> for BinaryValue {
    fn from (val: Vec<u8>) -> BinaryValue { BinaryValue { style: 0, value:val, alias: None } }
}



impl AsRef<Vec<u8>> for BinaryValue {
    fn as_ref (&self) -> &Vec<u8> { &self.value }
}



impl AsMut<Vec<u8>> for BinaryValue {
    fn as_mut (&mut self) -> &mut Vec<u8> { &mut self.value }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use model::{ Tagged, Renderer };
    // use txt::get_charset_utf8;

    use std::iter;



    #[test]
    fn tag () {
        let bin = Binary; // ::new (&get_charset_utf8 ());

        assert_eq! (bin.get_tag (), TAG);
    }



    #[test]
    fn encode () {
        let renderer = Renderer; // ::new (&get_charset_utf8 ());
        let bin = Binary; // ::new (&get_charset_utf8 ());

        let pairs = pairs ();

        for idx in 0 .. pairs.len () {
            let p = pairs[idx];

            if let Ok (rope) = bin.encode (&renderer, TaggedValue::from (BinaryValue::from (p.0.to_string ().into_bytes ())), &mut iter::empty ()) {

                println! ("rope: {:?}", rope);
                // let encoded = rope.render (&renderer);
                // let expected = p.1.as_bytes ();

                let encoded = unsafe { String::from_utf8_unchecked (rope.render (&renderer)) };
                let expected = p.1;

                assert_eq! (encoded, expected);
            } else { assert! (false, "Unexpected result") }
        }
    }



    #[test]
    fn decode () {
        let bin = Binary; // ::new (&get_charset_utf8 ());

        let pairs = pairs ();

        for idx in 0 .. pairs.len () {
            let p = pairs[idx];


            if let Ok (tagged) = bin.decode (true, &p.1.to_string ().into_bytes ()) {
                assert_eq! (tagged.get_tag (), &TWINE_TAG);

                let expected = p.0.as_bytes ();

                if let Some (bin) = tagged.as_any ().downcast_ref::<BinaryValue> () {
                    let val: &Vec<u8> = bin.as_ref ();
                    assert_eq! (*val, expected);
                } else { assert! (false) }
            } else { assert! (false, "Unexpected result") }
        }


        if let Ok (tagged) = bin.decode (true, &"".to_string ().into_bytes ()) {
            assert_eq! (tagged.get_tag (), &TWINE_TAG);

            let vec: &Vec<u8> = tagged.as_any ().downcast_ref::<BinaryValue> ().unwrap ().as_ref ();
            assert_eq! (0, vec.len ());
        } else { assert! (false, "Unexpected result") }


        // TODO: warning?
        // let decoded = bin.decode (&"=".to_string ().into_bytes ());
        // assert! (decoded.is_err ());

        // let decoded = bin.decode (&"c3VyZS4".to_string ().into_bytes ());
        // assert! (decoded.is_err ());
    }



    fn pairs () -> [(&'static str, &'static str); 11] {
        [
            ("sure.", "c3VyZS4="),
            ("asure.", "YXN1cmUu"),
            ("easure.", "ZWFzdXJlLg=="),
            ("leasure.", "bGVhc3VyZS4="),
            ("pleasure.", "cGxlYXN1cmUu"),
            ("any carnal pleas", "YW55IGNhcm5hbCBwbGVhcw=="),
            ("any carnal pleasu", "YW55IGNhcm5hbCBwbGVhc3U="),
            ("any carnal pleasur", "YW55IGNhcm5hbCBwbGVhc3Vy"),
            ("any carnal pleasure", "YW55IGNhcm5hbCBwbGVhc3VyZQ=="),
            ("any carnal pleasure.", "YW55IGNhcm5hbCBwbGVhc3VyZS4="),
            ("Man is distinguished, not only by his reason, but by this singular passion from other animals, which is a lust of the mind, that by a perseverance of delight in the continued and indefatigable generation of knowledge, exceeds the short vehemence of any carnal pleasure.", "TWFuIGlzIGRpc3Rpbmd1aXNoZWQsIG5vdCBvbmx5IGJ5IGhpcyByZWFzb24sIGJ1dCBieSB0aGlzIHNpbmd1bGFyIHBhc3Npb24gZnJvbSBvdGhlciBhbmltYWxzLCB3aGljaCBpcyBhIGx1c3Qgb2YgdGhlIG1pbmQsIHRoYXQgYnkgYSBwZXJzZXZlcmFuY2Ugb2YgZGVsaWdodCBpbiB0aGUgY29udGludWVkIGFuZCBpbmRlZmF0aWdhYmxlIGdlbmVyYXRpb24gb2Yga25vd2xlZGdlLCBleGNlZWRzIHRoZSBzaG9ydCB2ZWhlbWVuY2Ugb2YgYW55IGNhcm5hbCBwbGVhc3VyZS4=")
        ]
    }
}
