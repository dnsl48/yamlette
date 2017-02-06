extern crate skimmer;

use self::skimmer::symbol::{ Char, Symbol };

use txt::{ CharSet, Encoding, Twine };

use model::{ model_issue_rope, EncodedString, Factory, Model, Node, Rope, Renderer, Tagged, TaggedValue };
use model::style::CommonStyles;

use std::any::Any;
use std::iter::Iterator;



pub const TAG: &'static str = "tag:yaml.org,2002:binary";
static TWINE_TAG: Twine = Twine::Static (TAG);




pub struct Binary {
    encoding: Encoding,

    tbl: [Char; 64],
    pad: Char,
    line_feed: Char,
    carriage_return: Char,
    space: Char,
    tab_h: Char,

    s_quote: Char,
    d_quote: Char,

    tcl: usize
}



impl Binary {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }


    pub fn new (cset: &CharSet) -> Binary {
        let pad = cset.equal.clone ();

        let tbl = [
            cset.letter_t_a.clone (),
            cset.letter_t_b.clone (),
            cset.letter_t_c.clone (),
            cset.letter_t_d.clone (),
            cset.letter_t_e.clone (),
            cset.letter_t_f.clone (),
            cset.letter_t_g.clone (),
            cset.letter_t_h.clone (),
            cset.letter_t_i.clone (),
            cset.letter_t_j.clone (),
            cset.letter_t_k.clone (),
            cset.letter_t_l.clone (),
            cset.letter_t_m.clone (),
            cset.letter_t_n.clone (),
            cset.letter_t_o.clone (),
            cset.letter_t_p.clone (),
            cset.letter_t_q.clone (),
            cset.letter_t_r.clone (),
            cset.letter_t_s.clone (),
            cset.letter_t_t.clone (),
            cset.letter_t_u.clone (),
            cset.letter_t_v.clone (),
            cset.letter_t_w.clone (),
            cset.letter_t_x.clone (),
            cset.letter_t_y.clone (),
            cset.letter_t_z.clone (),

            cset.letter_a.clone (),
            cset.letter_b.clone (),
            cset.letter_c.clone (),
            cset.letter_d.clone (),
            cset.letter_e.clone (),
            cset.letter_f.clone (),
            cset.letter_g.clone (),
            cset.letter_h.clone (),
            cset.letter_i.clone (),
            cset.letter_j.clone (),
            cset.letter_k.clone (),
            cset.letter_l.clone (),
            cset.letter_m.clone (),
            cset.letter_n.clone (),
            cset.letter_o.clone (),
            cset.letter_p.clone (),
            cset.letter_q.clone (),
            cset.letter_r.clone (),
            cset.letter_s.clone (),
            cset.letter_t.clone (),
            cset.letter_u.clone (),
            cset.letter_v.clone (),
            cset.letter_w.clone (),
            cset.letter_x.clone (),
            cset.letter_y.clone (),
            cset.letter_z.clone (),
            
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

            cset.plus.clone (),
            cset.slash.clone ()
        ];

        let mut tbl_chr_len: usize = pad.len ();

        for i in 0 .. tbl.len () {
            if tbl[i].len () > tbl_chr_len { tbl_chr_len = tbl[i].len (); }
        }

        Binary {
            encoding: cset.encoding,

            line_feed: cset.line_feed.clone (),
            carriage_return: cset.carriage_return.clone (),
            space: cset.space.clone (),
            tab_h: cset.tab_h.clone (),

            s_quote: cset.apostrophe.clone (),
            d_quote: cset.quotation.clone (),

            tbl: tbl,
            pad: pad,
            tcl: tbl_chr_len
        }
    }
}



impl Model for Binary {
    fn get_tag (&self) -> &Twine { Binary::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

    fn get_encoding (&self) -> Encoding { self.encoding }


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

        let mut production: Vec<u8> = Vec::with_capacity (res_len * self.tcl);

        let mut rem: u8 = 0;

        for b in value {
            let b = b.to_be ();

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

            production.extend (self.tbl[idx as usize].as_slice ());
        }

        for _ in 0 .. (production.capacity () - production.len ()) {
            if rem > 0 {
                let idx = if rem & 0b1000_0000 == 0b1000_0000 {
                    (rem & 0b0000_1111) << 2
                } else if rem & 0b0100_0000 == 0b0100_0000 {
                    (rem & 0b0000_0011) << 4
                } else { 0 };

                rem = 0;

                production.extend (self.tbl[idx as usize].as_slice ());
            } else {
                production.extend (self.pad.as_slice ());
            }
        }

        let node = Node::String (EncodedString::from (production));

        Ok (model_issue_rope (self, node, issue_tag, alias, tags))
    }


    fn decode (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let vlen = value.len ();

        let mut production: Vec<u8> = Vec::with_capacity (vlen / (4 * self.tcl) * 3);

        // if vlen % (4 * self.tcl) != 0 { return Err ( () ) } // TODO: warning?

        let mut rem: u8 = 0;
        let mut ptr: usize = 0;

        let mut quote_state = 0; // 1 - single, 2 - double, 3 - finished


        'top: loop {
            if ptr >= vlen { break; }


            if quote_state == 1 {
                if self.s_quote.contained_at (value, ptr) {
                    ptr += self.s_quote.len ();
                    quote_state = 3;
                    continue;
                }
            }


            if quote_state == 2 {
                if self.d_quote.contained_at (value, ptr) {
                    ptr += self.d_quote.len ();
                    quote_state = 3;
                    continue;
                }
            }


            if quote_state == 0 && explicit {
                if self.s_quote.contained_at (value, ptr) {
                    ptr += self.s_quote.len ();
                    quote_state = 1;
                }
            }


            if quote_state == 0 && explicit {
                if self.d_quote.contained_at (value, ptr) {
                    ptr += self.d_quote.len ();
                    quote_state = 2;
                }
            }


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


            if quote_state == 3 { return Err ( () ) }


            for i in 0u8 .. 64u8 {
                if self.tbl[i as usize].contained_at (value, ptr) {
                    ptr += self.tbl[i as usize].len ();

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
                        continue 'top;
                    };

                    production.push (idx);
                    continue 'top;
                }
            }

            if self.pad.contained_at (value, ptr) {
                ptr += self.pad.len ();

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
                        continue 'top;
                    };

                if idx == 0 { break 'top; }

                production.push (idx);
            } else { return Err ( () ) }
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
    fn get_tag (&self) -> &Twine { Binary::get_tag () }

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




pub struct BinaryFactory;


impl Factory for BinaryFactory {
    fn get_tag (&self) -> &Twine { Binary::get_tag () }

    fn build_model (&self, cset: &CharSet) -> Box<Model> { Box::new (Binary::new (cset)) }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use model::{ Factory, Tagged, Renderer };
    use txt::get_charset_utf8;

    use std::iter;



    #[test]
    fn tag () {
        let bin = BinaryFactory.build_model (&get_charset_utf8 ());

        assert_eq! (bin.get_tag (), TAG);
    }



    #[test]
    fn encode () {
        let renderer = Renderer::new (&get_charset_utf8 ());
        let bin = BinaryFactory.build_model (&get_charset_utf8 ());

        let pairs = pairs ();

        for idx in 0 .. pairs.len () {
            let p = pairs[idx];

            if let Ok (rope) = bin.encode (&renderer, TaggedValue::from (BinaryValue::from (p.0.to_string ().into_bytes ())), &mut iter::empty ()) {
                let encoded = rope.render (&renderer);
                let expected = p.1.as_bytes ();

                assert_eq! (encoded, expected);
            } else { assert! (false, "Unexpected result") }
        }
    }



    #[test]
    fn decode () {
        let bin = BinaryFactory.build_model (&get_charset_utf8 ());

        let pairs = pairs ();

        for idx in 0 .. pairs.len () {
            let p = pairs[idx];


            if let Ok (tagged) = bin.decode (true, &p.1.to_string ().into_bytes ()) {
                assert_eq! (tagged.get_tag (), Binary::get_tag ());

                let expected = p.0.as_bytes ();

                if let Some (bin) = tagged.as_any ().downcast_ref::<BinaryValue> () {
                    let val: &Vec<u8> = bin.as_ref ();
                    assert_eq! (*val, expected);
                } else { assert! (false) }
            } else { assert! (false, "Unexpected result") }
        }


        if let Ok (tagged) = bin.decode (true, &"".to_string ().into_bytes ()) {
            assert_eq! (tagged.get_tag (), Binary::get_tag ());

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
