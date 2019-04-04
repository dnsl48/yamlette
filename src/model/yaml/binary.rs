extern crate skimmer;

use model::style::CommonStyles;
use model::{model_issue_rope, EncodedString, Model, Node, Renderer, Rope, Tagged, TaggedValue};

use std::any::Any;
use std::borrow::Cow;
use std::iter::Iterator;

pub static TAG: &'static str = "tag:yaml.org,2002:binary";

#[derive(Clone, Copy)]
pub struct Binary;

impl Binary {
    pub fn get_tag() -> Cow<'static, str> {
        Cow::from(TAG)
    }

    fn tbl(byte: u8) -> u8 {
        match byte {
            0...25 => byte + b'A',
            26...51 => byte + b'a' - 26,
            52...61 => byte + b'0' - 52,
            62 => b'+',
            63 => b'/',
            _ => unreachable!(),
        }
    }

    fn lbt(byte: u8) -> u8 {
        match byte {
            v @ b'A'...b'Z' => v - b'A',
            v @ b'a'...b'z' => v - b'a' + 26,
            v @ b'0'...b'9' => v - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => unreachable!(),
        }
    }
}

impl Model for Binary {
    fn get_tag(&self) -> Cow<'static, str> {
        Cow::from(TAG)
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
        let mut value: BinaryValue =
            match <TaggedValue as Into<Result<BinaryValue, TaggedValue>>>::into(value) {
                Ok(value) => value,
                Err(value) => return Err(value),
            };

        let issue_tag = value.issue_tag();
        let alias = value.take_alias();
        let value = value.to_vec();

        let res_len = (value.len()
            + if value.len() % 3 > 0 {
                3 - (value.len() % 3)
            } else {
                0
            })
            / 3
            * 4;

        let mut production: Vec<u8> = Vec::with_capacity(res_len);

        let mut rem: u8 = 0;

        for b in value {
            let b = b.to_be();

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
                production.push(Self::tbl(idx));
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

            production.push(Self::tbl(idx));
        }

        for _ in 0..(production.capacity() - production.len()) {
            if rem > 0 {
                let idx = if rem & 0b1000_0000 == 0b1000_0000 {
                    (rem & 0b0000_1111) << 2
                } else if rem & 0b0100_0000 == 0b0100_0000 {
                    (rem & 0b0000_0011) << 4
                } else {
                    0
                };

                rem = 0;

                production.push(Self::tbl(idx));
            } else {
                production.push(b'=');
            }
        }

        let node = Node::String(EncodedString::from(production));

        Ok(model_issue_rope(self, node, issue_tag, alias, tags))
    }

    fn decode(&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        // let vlen = value.len ();

        let mut production: Vec<u8> = Vec::with_capacity(value.len() / 4 * 3);

        // if vlen % (4 * self.tcl) != 0 { return Err ( () ) } // TODO: warning?

        let mut rem: u8 = 0;
        let mut ptr: usize = 0;

        let mut quote_state = 0; // 1 - single, 2 - double, 3 - finished

        if explicit && quote_state == 0 {
            match value.get(ptr).map(|b| *b) {
                Some(b'\'') => {
                    ptr += 1;
                    quote_state = 1;
                }
                Some(b'"') => {
                    ptr += 1;
                    quote_state = 2;
                }
                _ => (),
            };
        }

        loop {
            match value.get(ptr).map(|b| *b) {
                None => break,

                Some(b'\'') => {
                    if quote_state == 1 {
                        ptr += 1;
                        quote_state = 3;
                        break;
                    } else {
                        return Err(());
                    }
                }

                Some(b'"') => {
                    if quote_state == 2 {
                        ptr += 1;
                        quote_state = 3;
                        break;
                    } else {
                        return Err(());
                    }
                }

                Some(v @ b'a'...b'z')
                | Some(v @ b'A'...b'Z')
                | Some(v @ b'0'...b'9')
                | Some(v @ b'+')
                | Some(v @ b'/') => {
                    ptr += 1;
                    let i = Self::lbt(v);

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

                    production.push(idx);
                }

                Some(b'=') => {
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

                    if idx == 0 {
                        break;
                    }

                    production.push(idx);
                }

                Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') => {
                    ptr += 1;
                }

                _ => return Err(()),
            }
        }

        if quote_state == 3 {
            loop {
                match value.get(ptr).map(|b| *b) {
                    None => break,

                    Some(b' ') | Some(b'\n') | Some(b'\t') | Some(b'\r') => {
                        ptr += 1;
                    }

                    _ => return Err(()),
                }
            }
        }

        Ok(TaggedValue::from(BinaryValue::from(production)))
    }
}

#[derive(Clone, Debug)]
pub struct BinaryValue {
    style: u8,
    alias: Option<Cow<'static, str>>,
    value: Vec<u8>,
}

impl BinaryValue {
    pub fn new(
        value: Vec<u8>,
        styles: CommonStyles,
        alias: Option<Cow<'static, str>>,
    ) -> BinaryValue {
        BinaryValue {
            style: if styles.issue_tag() { 1 } else { 0 },
            value: value,
            alias: alias,
        }
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.value
    }

    pub fn take_alias(&mut self) -> Option<Cow<'static, str>> {
        self.alias.take()
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
}

impl Tagged for BinaryValue {
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

impl From<Vec<u8>> for BinaryValue {
    fn from(val: Vec<u8>) -> BinaryValue {
        BinaryValue {
            style: 0,
            value: val,
            alias: None,
        }
    }
}

impl AsRef<Vec<u8>> for BinaryValue {
    fn as_ref(&self) -> &Vec<u8> {
        &self.value
    }
}

impl AsMut<Vec<u8>> for BinaryValue {
    fn as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.value
    }
}

#[cfg(all(test, not(feature = "dev")))]
mod tests {
    use super::*;

    use model::{Renderer, Tagged};
    // use txt::get_charset_utf8;

    use std::iter;

    #[test]
    fn tag() {
        let bin = Binary; // ::new (&get_charset_utf8 ());

        assert_eq!(bin.get_tag(), TAG);
    }

    #[test]
    fn encode() {
        let renderer = Renderer; // ::new (&get_charset_utf8 ());
        let bin = Binary; // ::new (&get_charset_utf8 ());

        let pairs = pairs();

        for idx in 0..pairs.len() {
            let p = pairs[idx];

            if let Ok(rope) = bin.encode(
                &renderer,
                TaggedValue::from(BinaryValue::from(p.0.to_string().into_bytes())),
                &mut iter::empty(),
            ) {
                println!("rope: {:?}", rope);
                // let encoded = rope.render (&renderer);
                // let expected = p.1.as_bytes ();

                let encoded = unsafe { String::from_utf8_unchecked(rope.render(&renderer)) };
                let expected = p.1;

                assert_eq!(encoded, expected);
            } else {
                assert!(false, "Unexpected result")
            }
        }
    }

    #[test]
    fn decode() {
        let bin = Binary; // ::new (&get_charset_utf8 ());

        let pairs = pairs();

        for idx in 0..pairs.len() {
            let p = pairs[idx];

            if let Ok(tagged) = bin.decode(true, &p.1.to_string().into_bytes()) {
                assert_eq!(tagged.get_tag(), Cow::from(TAG));

                let expected = p.0.as_bytes();

                if let Some(bin) = tagged.as_any().downcast_ref::<BinaryValue>() {
                    let val: &Vec<u8> = bin.as_ref();
                    assert_eq!(*val, expected);
                } else {
                    assert!(false)
                }
            } else {
                assert!(false, "Unexpected result")
            }
        }

        if let Ok(tagged) = bin.decode(true, &"".to_string().into_bytes()) {
            assert_eq!(tagged.get_tag(), Cow::from(TAG));

            let vec: &Vec<u8> = tagged
                .as_any()
                .downcast_ref::<BinaryValue>()
                .unwrap()
                .as_ref();
            assert_eq!(0, vec.len());
        } else {
            assert!(false, "Unexpected result")
        }

        // TODO: warning?
        // let decoded = bin.decode (&"=".to_string ().into_bytes ());
        // assert! (decoded.is_err ());

        // let decoded = bin.decode (&"c3VyZS4".to_string ().into_bytes ());
        // assert! (decoded.is_err ());
    }

    fn pairs() -> [(&'static str, &'static str); 11] {
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
