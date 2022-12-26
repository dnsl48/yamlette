extern crate skimmer;

use crate::txt::encoding::{Unicode, UTF8};

use crate::model::renderer::{EncodedString, Node, Renderer};
use crate::model::style::{CommonStyles, Style};
use crate::model::{model_issue_rope, Model, Rope, Tagged, TaggedValue};

use std::any::Any;
use std::borrow::Cow;
use std::iter::Iterator;
use std::mem;

use std::ptr;

pub static TAG: &'static str = "tag:yaml.org,2002:str";

// TODO: do warnings for incorrect escapes on decode (and encode)

#[derive(Clone, Copy)]
pub struct Str;

impl Str {
    pub fn get_tag() -> Cow<'static, str> {
        Cow::from(TAG)
    }

    fn extract_hex_at(&self, src: &[u8], mut at: usize, mut len_limit: u8) -> Option<u32> {
        let mut result: u32 = 0;

        loop {
            if len_limit == 0 {
                break;
            }
            len_limit -= 1;

            let val = match src.get(at).map(|b| *b) {
                Some(val @ b'0'..=b'9') => val - b'0',
                Some(val @ b'a'..=b'f') => 10 + (val - b'a'),
                Some(val @ b'A'..=b'F') => 10 + (val - b'A'),
                _ => return None,
            };

            at += 1;

            result = if let Some(nv) = result.checked_mul(16) {
                if let Some(nv) = nv.checked_add(val as u32) {
                    nv
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        return Some(result);
    }

    // TODO: redo it safely
    unsafe fn encode_auto_quoted(
        &self,
        mut value: StrValue,
        tags: &mut dyn Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
    ) -> Rope {
        let issue_tag = value.issue_tag();
        let alias = value.take_alias();
        let string = value.take_twine();
        let bytes: &[u8] = string.as_bytes();

        let capacity = bytes.len() * 2; // reserve 2 bytes for escaped with a backslash chars
        let mut result_string: Vec<u8> = Vec::with_capacity(capacity);

        /*
           0 - no quotes
           1 - singles
           2 - doubles
        */
        let mut quotes: u8 = if value.force_quotes() {
            if value.prefer_double_quotes() {
                2
            } else {
                1
            }
        } else {
            0
        };

        let mut first_rollback_at: Option<(usize, usize)> = None;

        let mut sptr: *const u8 = bytes.as_ptr();
        let mut slen: usize = 0;

        let mut rptr: *mut u8 = result_string.as_mut_ptr();
        let mut rlen: usize = 0;

        'main_loop: loop {
            if slen >= bytes.len() {
                break;
            }
            if rlen >= capacity {
                unreachable!() /* overflow! */
            }

            let code = *sptr;
            slen += 1; // len as usize;
            sptr = sptr.offset(1 as isize);

            match code {
                0 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some((rollback_slen, rollback_rlen)) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr().offset(slen as isize);
                            rptr = result_string.as_mut_ptr().offset(rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2; // self.backslash.len () + self.digit_0.len ();
                    result_string.set_len(rlen);

                    ptr::copy_nonoverlapping("\\0".as_ptr(), rptr, 2);
                    rptr = rptr.offset(2);
                }

                7 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some((rollback_slen, rollback_rlen)) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr().offset(slen as isize);
                            rptr = result_string.as_mut_ptr().offset(rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2; // self.backslash.len () + self.letter_a.len ();
                    result_string.set_len(rlen);

                    ptr::copy_nonoverlapping("\\a".as_ptr(), rptr, 2);
                    rptr = rptr.offset(2);
                }

                8 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some((rollback_slen, rollback_rlen)) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr().offset(slen as isize);
                            rptr = result_string.as_mut_ptr().offset(rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2;
                    result_string.set_len(rlen);

                    ptr::copy_nonoverlapping("\\b".as_ptr(), rptr, 2);
                    rptr = rptr.offset(2);
                }

                9 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some((rollback_slen, rollback_rlen)) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr().offset(slen as isize);
                            rptr = result_string.as_mut_ptr().offset(rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2; // self.backslash.len () + self.letter_t.len ();
                    result_string.set_len(rlen);

                    ptr::copy_nonoverlapping("\\t".as_ptr(), rptr, 2);
                    rptr = rptr.offset(2);
                }

                10 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some((rollback_slen, rollback_rlen)) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr().offset(slen as isize);
                            rptr = result_string.as_mut_ptr().offset(rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2;
                    result_string.set_len(rlen);

                    ptr::copy_nonoverlapping("\\n".as_ptr(), rptr, 2);
                    rptr = rptr.offset(2);
                }

                11 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some((rollback_slen, rollback_rlen)) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr().offset(slen as isize);
                            rptr = result_string.as_mut_ptr().offset(rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2;
                    result_string.set_len(rlen);

                    ptr::copy_nonoverlapping("\\v".as_ptr(), rptr, 2);
                    rptr = rptr.offset(2);
                }

                12 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some((rollback_slen, rollback_rlen)) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr().offset(slen as isize);
                            rptr = result_string.as_mut_ptr().offset(rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2;
                    result_string.set_len(rlen);

                    ptr::copy_nonoverlapping("\\f".as_ptr(), rptr, 2);
                    rptr = rptr.offset(2);
                }

                13 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some((rollback_slen, rollback_rlen)) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr().offset(slen as isize);
                            rptr = result_string.as_mut_ptr().offset(rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2;
                    result_string.set_len(rlen);

                    ptr::copy_nonoverlapping("\\r".as_ptr(), rptr, 2);
                    rptr = rptr.offset(2);
                }

                27 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some((rollback_slen, rollback_rlen)) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr().offset(slen as isize);
                            rptr = result_string.as_mut_ptr().offset(rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2;
                    result_string.set_len(rlen);

                    ptr::copy_nonoverlapping("\\e".as_ptr(), rptr, 2);
                    rptr = rptr.offset(2);
                }

                34 => {
                    if quotes == 2 {
                        rlen += 2;
                        result_string.set_len(rlen);

                        ptr::copy_nonoverlapping("\\\"".as_ptr(), rptr, 2);
                        rptr = rptr.offset(2);

                        continue;
                    }

                    if first_rollback_at.is_none() {
                        first_rollback_at = Some((slen - 1, rlen));
                    }

                    rlen += 1;
                    result_string.set_len(rlen);

                    *rptr = b'"';
                    rptr = rptr.offset(1);
                }

                39 => {
                    if quotes == 1 {
                        rlen += 2;
                        result_string.set_len(rlen);

                        ptr::copy_nonoverlapping("''".as_ptr(), rptr, 2);
                        rptr = rptr.offset(2);

                        continue;
                    }

                    if first_rollback_at.is_none() {
                        first_rollback_at = Some((slen - 1, rlen));
                    }

                    rlen += 1;
                    result_string.set_len(rlen);

                    *rptr = b'\'';
                    rptr = rptr.offset(1);
                }

                92 => {
                    if quotes == 2 {
                        rlen += 2;
                        result_string.set_len(rlen);

                        ptr::copy_nonoverlapping("\\\\".as_ptr(), rptr, 2);
                        rptr = rptr.offset(2);

                        continue;
                    }

                    if first_rollback_at.is_none() {
                        first_rollback_at = Some((slen - 1, rlen));
                    }

                    rlen += 1;
                    result_string.set_len(rlen);

                    *rptr = b'\\';
                    rptr = rptr.offset(1);
                }

                _ => {
                    rlen += 1;
                    result_string.set_len(rlen);

                    *rptr = code;
                    rptr = rptr.offset(1);
                }
            };
        }

        let node = if quotes == 2 {
            Node::DoubleQuotedString(EncodedString::from(result_string))
        } else if quotes == 1 {
            Node::SingleQuotedString(EncodedString::from(result_string))
        } else {
            Node::String(EncodedString::from(result_string))
        };

        model_issue_rope(self, node, issue_tag, alias, tags)
    }
}

impl Model for Str {
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

    fn has_default(&self) -> bool {
        true
    }

    fn get_default(&self) -> TaggedValue {
        TaggedValue::from(StrValue::from(String::new()))
    }

    fn encode(
        &self,
        _renderer: &Renderer,
        value: TaggedValue,
        tags: &mut dyn Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
    ) -> Result<Rope, TaggedValue> {
        let value: StrValue =
            match <TaggedValue as Into<Result<StrValue, TaggedValue>>>::into(value) {
                Ok(value) => value,
                Err(value) => return Err(value),
            };

        unsafe { Ok(self.encode_auto_quoted(value, tags)) }
    }

    // TODO: check if value.get_unchecked goes faster
    fn decode(&self, _explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let mut ptr: usize = 0;
        let mut state: u8 = 0;

        const STATE_SPACE: u8 = 1;
        const STATE_BREAK: u8 = 2;

        match value.get(ptr).map(|b| *b) {
            Some(b'"') => {
                let mut result: Vec<u8> = Vec::with_capacity(value.len());
                let mut spaces: usize = 0;

                ptr += 1;

                loop {
                    match value.get(ptr).map(|b| *b) {
                        None => break,

                        Some(b @ b' ') | Some(b @ b'\t') => {
                            ptr += 1;

                            if state & STATE_BREAK > 0 {
                                continue;
                            }

                            spaces += 1;

                            state = state | STATE_SPACE;
                            result.push(b);
                        }

                        Some(b @ b'\n') | Some(b @ b'\r') => {
                            ptr += if b == b'\r' && value.get(ptr + 1).map(|b| *b) == Some(b'\n') {
                                2
                            } else {
                                1
                            };

                            if spaces > 1 {
                                let len = result.len() - spaces + 1;
                                result.truncate(len);
                            } else if spaces == 0 {
                                spaces = 1;
                                result.push(b' ');
                            }

                            if state & STATE_BREAK > 0 {
                                if spaces > 0 {
                                    let len = result.len() - spaces;
                                    result.truncate(len);
                                    spaces = 0;
                                }

                                result.push(b'\n');
                            }

                            state = state | STATE_BREAK;
                            state = state & !STATE_SPACE;
                        }

                        Some(b'"') => {
                            ptr += 1;

                            if ptr == value.len() {
                                break;
                            } else {
                                return Err(());
                            }
                        }

                        Some(b'\\') => {
                            ptr += 1;

                            if state & STATE_BREAK > 0 {
                                state = state & !STATE_BREAK;
                                spaces = 0;
                            }
                            if state & STATE_SPACE > 0 {
                                state = state & !STATE_SPACE;
                                spaces = 0;
                            }

                            match value.get(ptr).map(|b| *b) {
                                Some(b @ b'\n') | Some(b @ b'\r') => {
                                    ptr += if b == b'\r'
                                        && value.get(ptr + 1).map(|b| *b) == Some(b'\n')
                                    {
                                        2
                                    } else {
                                        1
                                    };

                                    spaces = 0;

                                    state = state | STATE_BREAK;
                                    state = state & !STATE_SPACE;
                                }

                                Some(b @ b'\t') | Some(b @ b' ') => {
                                    ptr += 1;

                                    spaces = 0;

                                    state = state & !STATE_BREAK;
                                    state = state & !STATE_SPACE;

                                    result.push(b);
                                }

                                Some(b'0') => {
                                    ptr += 1;
                                    result.push(0);
                                }
                                Some(b'a') => {
                                    ptr += 1;
                                    result.push(7);
                                }
                                Some(b'b') => {
                                    ptr += 1;
                                    result.push(8);
                                }
                                Some(b't') => {
                                    ptr += 1;
                                    result.push(9);
                                }
                                Some(b'n') => {
                                    ptr += 1;
                                    result.push(10);
                                }
                                Some(b'v') => {
                                    ptr += 1;
                                    result.push(11);
                                }
                                Some(b'f') => {
                                    ptr += 1;
                                    result.push(12);
                                }
                                Some(b'r') => {
                                    ptr += 1;
                                    result.push(13);
                                }
                                Some(b'e') => {
                                    ptr += 1;
                                    result.push(27);
                                }
                                Some(b'"') => {
                                    ptr += 1;
                                    result.push(34);
                                }
                                Some(b'/') => {
                                    ptr += 1;
                                    result.push(47);
                                }
                                Some(b'\\') => {
                                    ptr += 1;
                                    result.push(92);
                                }
                                Some(b'N') => {
                                    ptr += 1;
                                    result.push(0xC2);
                                    result.push(0x85);
                                }
                                Some(b'_') => {
                                    ptr += 1;
                                    result.push(0xC2);
                                    result.push(0xA0);
                                }
                                Some(b'L') => {
                                    ptr += 1;
                                    result.push(0xE2);
                                    result.push(0x80);
                                    result.push(0xA8);
                                }
                                Some(b'P') => {
                                    ptr += 1;
                                    result.push(0xE2);
                                    result.push(0x80);
                                    result.push(0xA9);
                                }

                                Some(b'x') => {
                                    // ptr += 1;

                                    if let Some(code) = self.extract_hex_at(value, ptr + 1, 2) {
                                        ptr += 3;
                                        let code = UTF8.from_unicode(code);
                                        result.extend(&code[..code[4] as usize]);
                                    } else {
                                        result.push(b'\\');
                                    }
                                }

                                Some(b'u') => {
                                    // ptr += 1;

                                    if let Some(code) = self.extract_hex_at(value, ptr + 1, 4) {
                                        ptr += 5;
                                        let code = UTF8.from_unicode(code);
                                        result.extend(&code[..code[4] as usize]);
                                    } else {
                                        result.push(b'\\');
                                    }
                                }

                                Some(b'U') => {
                                    // ptr += 1;

                                    if let Some(code) = self.extract_hex_at(value, ptr + 1, 8) {
                                        ptr += 9;
                                        let code = UTF8.from_unicode(code);
                                        result.extend(&code[..code[4] as usize]);
                                    } else {
                                        result.push(b'\\');
                                    }
                                }

                                _ => {
                                    result.push(b'\\');
                                }
                            }
                        }

                        Some(b @ _) => {
                            ptr += 1;

                            if state & STATE_BREAK > 0 {
                                state = state & !STATE_BREAK;
                                spaces = 0;
                            }
                            if state & STATE_SPACE > 0 {
                                state = state & !STATE_SPACE;
                                spaces = 0;
                            }

                            result.push(b);
                        }
                    }
                }

                match String::from_utf8(result) {
                    Ok(s) => Ok(TaggedValue::from(StrValue::from(s))),
                    _ => Err(()),
                }
            }

            Some(b'\'') => {
                let mut result: Vec<u8> = Vec::with_capacity(value.len());

                let mut spaces: usize = 0;

                ptr += 1;

                loop {
                    match value.get(ptr).map(|b| *b) {
                        None => break,

                        Some(b @ b' ') | Some(b @ b'\t') => {
                            ptr += 1;

                            if state & STATE_BREAK > 0 {
                                continue;
                            }

                            spaces += 1;

                            state = state | STATE_SPACE;
                            result.push(b);
                        }

                        Some(b @ b'\n') | Some(b @ b'\r') => {
                            ptr += if b == b'\r' && value.get(ptr + 1).map(|b| *b) == Some(b'\n') {
                                2
                            } else {
                                1
                            };

                            if spaces > 1 {
                                let len = result.len() - spaces + 1;
                                result.truncate(len);
                            } else if spaces == 0 {
                                spaces = 1;
                                result.push(b' ');
                            }

                            if state & STATE_BREAK > 0 {
                                if spaces > 0 {
                                    let len = result.len() - spaces;
                                    result.truncate(len);
                                    spaces = 0;
                                }

                                result.push(b'\n');
                            }

                            state = state | STATE_BREAK;
                            state = state & !STATE_SPACE;
                        }

                        Some(b'\'') => {
                            ptr += 1;

                            match value.get(ptr).map(|b| *b) {
                                Some(b'\'') => {
                                    ptr += 1;
                                    result.push(b'\'');
                                }
                                None => break,
                                _ => return Err(()),
                            };
                        }

                        Some(b @ _) => {
                            ptr += 1;

                            if state & STATE_BREAK > 0 {
                                state = state & !STATE_BREAK;
                                spaces = 0;
                            }
                            if state & STATE_SPACE > 0 {
                                state = state & !STATE_SPACE;
                                spaces = 0;
                            }

                            result.push(b);
                        }
                    }
                }

                match String::from_utf8(result) {
                    Ok(s) => Ok(TaggedValue::from(StrValue::from(s))),
                    _ => Err(()),
                }
            }

            _ => match String::from_utf8(Vec::from(value)) {
                Ok(s) => Ok(TaggedValue::from(StrValue::from(s))),
                _ => Err(()),
            },
        }
    }

    /*
        fn decode (&self, _explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
            let mut ptr: usize = 0;
            let mut state: u8 = 0;

            const STATE_SPACE: u8 = 1;
            const STATE_ESCNL: u8 = 2;
            const STATE_BREAK: u8 = 4;


            match value.get (ptr).map (|b| *b) {
                Some (b'"') => {
                    // let utf8 = UTF8;
                    let mut buffer: Vec<u8> = Vec::with_capacity (value.len ());
                    let mut result: Vec<u8> = Vec::with_capacity (value.len ());

                    ptr += 1; // self.d_quote.len ();

                    let mut byte: u8 = value.get (ptr).map (|b| *b);

                    loop {
                        match byte {
                            None => break,

                            Some (b' ') => {
                                ptr += 1;

                                state = state | STATE_SPACE;

                                if state & (STATE_ESCNL | STATE_BREAK) != (STATE_ESCNL | STATE_BREAK) {
                                    buffer.push (b' ');
                                }
                            }

                            Some (b'\t') => {
                                ptr += 1;

                                state = state | STATE_SPACE;
                                if state & (STATE_ESCNL | STATE_BREAK) != (STATE_ESCNL | STATE_BREAK) {
                                    buffer.push (b'\t');
                                }
                            }

                            Some (b'\n') => {
                                ptr += 1;

                                if state & STATE_BREAK == 0 {
                                    buffer.clear ();
                                    state = state | STATE_BREAK | STATE_SPACE;
                                    if state & STATE_ESCNL == 0 { buffer.push (b'\n'); }
                                } else if state & (STATE_ESCNL | STATE_BREAK) != (STATE_ESCNL | STATE_BREAK) {
                                    buffer.push (b'\n');
                                } else {
                                    unimplemented! ()
                                }
                            }

                            Some (b'\r') => {
                                ptr += if let Some (b'\n') = value.get (ptr + 1).map (|b| *b) { 2 } else { 1 };

                                if state & STATE_BREAK == 0 {
                                    buffer.clear ();
                                    state = state | STATE_BREAK | STATE_SPACE;
                                    if state & STATE_ESCNL == 0 { buffer.push (b'\n'); }
                                } else if state & (STATE_ESCNL | STATE_BREAK) != (STATE_ESCNL | STATE_BREAK) {
                                    buffer.push (b'\n');
                                } else {
                                    unimplemented! ()
                                }
                            }

                            Some (b'\\') => match value.get (ptr + 1).map (|b| *b) {
                                Some (b'\n') => {
                                    ptr += 2;
                                    state = state | STATE_ESCNL;
                                }
                                Some (b'\r') => {
                                    ptr += if let Some (b'\n') = value.get (ptr + 2).map (|b| *b) { 3 } else { 2 };
                                    state = state | STATE_ESCNL;
                                }
                                Some (b'0') => { ptr += 2; result.push (0); }
                                Some (b'a') => { ptr += 2; result.push (7); }
                                Some (b'b') => { ptr += 2; result.push (8); }
                                Some (b't') => { ptr += 2; result.push (9); }
                                Some (b'n') => { ptr += 2; result.push (10); }
                                Some (b'v') => { ptr += 2; result.push (11); }
                                Some (b'f') => { ptr += 2; result.push (12); }
                                Some (b'r') => { ptr += 2; result.push (13); }
                                Some (b'e') => { ptr += 2; result.push (27); }
                                Some (b' ') => { ptr += 2; result.push (32); }
                                Some (b'"') => { ptr += 2; result.push (34); }
                                Some (b'/') => { ptr += 2; result.push (47); }
                                Some (b'\\') => { ptr += 2; result.push (92); }
                                Some (b'N') => { ptr += 2; result.push (0xC2); result.push (0x85); }
                                Some (b'_') => { ptr += 2; result.push (0xC2); result.push (0xA0); }
                                Some (b'L') => { ptr += 3; result.push (0xE2); result.push (0x80); result.push (0xA8); }
                                Some (b'P') => { ptr += 3; result.push (0xE2); result.push (0x80); result.push (0xA9); }

                                Some (b'x') => {
                                    ptr += 1;

                                    if let Some ((code, len)) = self.extract_hex_at (value, ptr) {
                                        if len - ptr == 2 {
                                            ptr += 2;
                                            let code = UTF8.from_unicode (code);
                                            result.extend (&code[ .. code[4] as usize]);
                                        } else { result.push (b'\\'); }
                                    } else { result.push (b'\\'); }
                                }

                                Some (b'u') => {
                                    ptr += 1;

                                    if let Some ((code, len)) = self.extract_hex_at (value, ptr) {
                                        if len - ptr == 4 {
                                            ptr += 4;
                                            let code = UTF8.from_unicode (code);
                                            result.extend (&code[ .. code[4] as usize]);
                                        } else { result.push (b'\\'); }
                                    } else { result.push (b'\\'); }
                                }

                                Some (b'U') => {
                                    ptr += 1;

                                    if let Some ((code, len)) = self.extract_hex_at (value, ptr) {
                                        if len - ptr == 8 {
                                            ptr += 8;
                                            let code = UTF8.from_unicode (code);
                                            result.extend (&code[ .. code[4] as usize]);
                                        } else { result.push (b'\\'); }
                                    } else { result.push (b'\\'); }
                                }

                                _ => { ptr += 1; result.push (b'\\'); }
                            },

                            _ if state & STATE_SPACE == STATE_SPACE => {
                                if state & STATE_BREAK == STATE_BREAK && buffer.len () == 0 {
                                    result.push (b' ');
                                } else {
                                    result.append (&mut buffer);
                                }

                                state = 0;
                            }

                            Some (b'"') if ptr + 1 == value.len () => { break }

                            Some (b @ _) => {
                                ptr += 1;
                                result.push (b);
                            }
                        };
                    }

                    Ok (TaggedValue::from (StrValue::from (unsafe { String::from_utf8_unchecked (result) })))
                }

                Some (b'\'') => {
                    // let utf8 = UTF8;
                    let mut buffer: Vec<u8> = Vec::with_capacity (value.len ());
                    let mut result: Vec<u8> = Vec::with_capacity (value.len ());

                    ptr += 1;

                    loop {
                        match value.get (ptr).map (|b| *b) {
                            None => break,

                            Some (b' ') => {
                                ptr += 1;

                                state = state | STATE_SPACE;

                                if state & (STATE_ESCNL | STATE_BREAK) != (STATE_ESCNL | STATE_BREAK) {
                                    buffer.push (b' ');
                                }
                            }

                            Some (b'\t') => {
                                ptr += 1;

                                state = state | STATE_SPACE;
                                if state & (STATE_ESCNL | STATE_BREAK) != (STATE_ESCNL | STATE_BREAK) {
                                    buffer.push (b'\t');
                                }
                            }

                            Some (b'\n') => {
                                ptr += 1;

                                if state & STATE_BREAK == 0 {
                                    buffer.clear ();
                                    state = state | STATE_BREAK | STATE_SPACE;
                                } else if state & (STATE_ESCNL | STATE_BREAK) != (STATE_ESCNL | STATE_BREAK) {
                                    buffer.push (b'\n');
                                } else {
                                    unimplemented! ()
                                }
                            }

                            Some (b'\r') => {
                                ptr += if let Some (b'\n') = value.get (ptr + 1).map (|b| *b) { 2 } else { 1 };

                                if state & STATE_BREAK == 0 {
                                    buffer.clear ();
                                    state = state | STATE_BREAK | STATE_SPACE;
                                } else if state & (STATE_ESCNL | STATE_BREAK) != (STATE_ESCNL | STATE_BREAK) {
                                    buffer.push (b'\n');
                                } else {
                                    unimplemented! ()
                                }
                            }

                            Some (b'\\') => match value.get (ptr + 1).map (|b| *b) {
                                Some (b'\n') => {
                                    ptr += 2;
                                    state = state | STATE_ESCNL;
                                }

                                Some (b'\r') => {
                                    ptr += if let Some (b'\n') = value.get (ptr + 2).map (|b| *b) { 3 } else { 2 };
                                    state = state | STATE_ESCNL;
                                }

                                _ => { ptr += 1; result.push (b'\\'); }
                            },

                            _ if state & STATE_SPACE == STATE_SPACE => {
                                if state & STATE_BREAK == STATE_BREAK && buffer.len () == 0 {
                                    result.push (b' ');
                                } else {
                                    result.append (&mut buffer);
                                }

                                state = 0;
                            }

                            Some (b'\'') => match value.get (ptr + 1).map (|b| *b) {
                                Some (b'\'') => {
                                    ptr += 1;
                                    result.push (b'\'');
                                }
                                None => { break }
                                _ => return Err ( () )
                            },

                            // Some (b'"') if ptr + 1 == value.len () => { ptr += 1; break }

                            Some (b @ _) => {
                                ptr += 1;
                                result.push (b);
                            }
                        };
                    }

                    let s = String::from_utf8 (result);

                    if s.is_err () { Err ( () ) }
                    else { Ok ( TaggedValue::from (StrValue::from (s.unwrap ())) ) }

                }

                _ => {
                    Ok ( TaggedValue::from (StrValue::from (unsafe { String::from_utf8_unchecked (Vec::from (value)) })) )
                }
            }

        }
    */
}

pub const FORCE_QUOTES: ForceQuotes = ForceQuotes(true);
pub const NO_FORCE_QUOTES: ForceQuotes = ForceQuotes(false);

pub struct ForceQuotes(pub bool);

impl Style for ForceQuotes {
    fn tagged_styles_apply(&mut self, value: &mut dyn Tagged) {
        if value.get_tag().as_ref() != TAG {
            return;
        }

        if let Some(ref mut str_val) = value.as_mut_any().downcast_mut::<StrValue>() {
            str_val.set_force_quotes(self.0);
        }
    }
}

pub const PREFER_DOUBLE_QUOTES: PreferDoubleQuotes = PreferDoubleQuotes(true);
pub const NO_PREFER_DOUBLE_QUOTES: PreferDoubleQuotes = PreferDoubleQuotes(false);

pub struct PreferDoubleQuotes(pub bool);

impl Style for PreferDoubleQuotes {
    fn tagged_styles_apply(&mut self, value: &mut dyn Tagged) {
        if value.get_tag().as_ref() != TAG {
            return;
        }

        if let Some(ref mut str_val) = value.as_mut_any().downcast_mut::<StrValue>() {
            str_val.set_prefer_double_quotes(self.0);
        }
    }
}

#[derive(Debug)]
pub struct StrValue {
    style: u8,

    alias: Option<Cow<'static, str>>,

    value: Cow<'static, str>,
}

impl StrValue {
    pub fn new(
        val: Cow<'static, str>,
        styles: CommonStyles,
        alias: Option<Cow<'static, str>>,
    ) -> StrValue {
        StrValue {
            style: if styles.issue_tag() { 1 } else { 0 },
            alias: alias,
            value: val,
        }
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

    pub fn force_quotes(&self) -> bool {
        self.style & 2 == 2
    }

    pub fn set_force_quotes(&mut self, val: bool) {
        if val {
            self.style |= 2;
        } else {
            self.style &= !2;
        }
    }

    pub fn prefer_double_quotes(&self) -> bool {
        self.style & 4 == 4
    }

    pub fn set_prefer_double_quotes(&mut self, val: bool) {
        if val {
            self.style |= 4;
        } else {
            self.style &= !4;
        }
    }

    pub fn take_twine(&mut self) -> Cow<'static, str> {
        mem::replace(&mut self.value, Cow::from(String::with_capacity(0)))
    }

    pub fn take_alias(&mut self) -> Option<Cow<'static, str>> {
        self.alias.take()
    }

    pub fn to_twine(self) -> Cow<'static, str> {
        self.value
    }
}

impl Tagged for StrValue {
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

impl From<char> for StrValue {
    fn from(value: char) -> StrValue {
        StrValue {
            style: 0,
            alias: None,
            value: Cow::from(value.to_string()),
        }
    }
}

impl From<Cow<'static, str>> for StrValue {
    fn from(value: Cow<'static, str>) -> StrValue {
        StrValue {
            style: 0,
            alias: None,
            value: value,
        }
    }
}

impl From<String> for StrValue {
    fn from(value: String) -> StrValue {
        StrValue {
            style: 0,
            alias: None,
            value: Cow::from(value),
        }
    }
}

impl From<&'static str> for StrValue {
    fn from(value: &'static str) -> StrValue {
        StrValue {
            style: 0,
            alias: None,
            value: Cow::from(value),
        }
    }
}

impl AsRef<str> for StrValue {
    fn as_ref(&self) -> &str {
        self.value.as_ref()
    }
}

#[cfg(all(test, not(feature = "dev")))]
mod tests {
    use super::*;

    use crate::model::{Renderer, Tagged};

    use std::iter;

    #[test]
    fn tag() {
        // let str = StrFactory.build_model (&get_charset_utf8 ());
        let str = Str; // ::new (&get_charset_utf8 ());

        assert_eq!(str.get_tag(), TAG);
    }

    #[test]
    fn encode() {
        let renderer = Renderer; // ::new (&get_charset_utf8 ());
        let str = Str; // ::new (&get_charset_utf8 ());

        let ops = [
            ("Hey, this is a string!", "Hey, this is a string!"),
            (
                "Hey,\nthis is\tanother\" one",
                r#""Hey,\nthis is\tanother\" one""#,
            ),
        ];

        for i in 0..ops.len() {
            if let Ok(rope) = str.encode(
                &renderer,
                TaggedValue::from(StrValue::from(ops[i].0.to_string())),
                &mut iter::empty(),
            ) {
                let vec = rope.render(&renderer);
                assert_eq!(vec, ops[i].1.to_string().into_bytes().to_vec());
            } else {
                assert!(false)
            }

            if let Ok(rope) = str.encode(
                &renderer,
                TaggedValue::from(StrValue::from(ops[i].0)),
                &mut iter::empty(),
            ) {
                let vec = rope.render(&renderer);
                assert_eq!(vec, ops[i].1.to_string().into_bytes().to_vec());
            } else {
                assert!(false)
            }
        }
    }

    #[test]
    fn decode() {
        let str = Str; // ::new (&get_charset_utf8 ());

        let ops = [
            ("Hey, this is a string!", "Hey, this is a string!"),
            (r"'Hey, that''s the string!'", "Hey, that's the string!"),
            (r#""Hey, that\"s the string!""#, "Hey, that\"s the string!"),
            (
                r#""Hey,\n\ that's\tthe\0string\\""#,
                "Hey,\n that's\tthe\0string\\",
            ),
            (r#""This\x0Ais\x09a\x2c\x20test""#, "This\nis\ta, test"),
            (r#""\u0422\u0435\u0441\u0442\x0a""#, "Тест\n"),
            (r#""\u30c6\u30b9\u30c8\x0a""#, "テスト\n"),
            (
                r#""\U00013000\U00013001\U00013002\U00013003\U00013004\U00013005\U00013006\U00013007""#,
                "𓀀𓀁𓀂𓀃𓀄𓀅𓀆𓀇",
            ),
        ];

        for i in 0..ops.len() {
            if let Ok(tagged) = str.decode(true, ops[i].0.as_bytes()) {
                assert_eq!(tagged.get_tag(), Cow::from(TAG));

                let val: &str = tagged.as_any().downcast_ref::<StrValue>().unwrap().as_ref();

                assert_eq!(val, ops[i].1);
            } else {
                assert!(false)
            }
        }
    }

    #[test]
    fn folding() {
        let str = Str; // ::new (&get_charset_utf8 ());

        let ops = [
            (r#""Four    spaces""#, "Four    spaces"),
            (r#""Four  \  spaces""#, "Four    spaces"),
            (
                r#""Four  
  spaces""#,
                "Four spaces",
            ),
            (
                r#"" 1st non-empty

 2nd non-empty 
	3rd non-empty ""#,
                " 1st non-empty\n2nd non-empty 3rd non-empty ",
            ),
            (
                r#""folded 
to a space,	
 
to a line feed, or 	\
 \ 	non-content""#,
                "folded to a space,\nto a line feed, or \t \tnon-content",
            ),
            (
                r#""
  foo 
 
  	 bar

  baz
""#,
                " foo\nbar\nbaz ",
            ),
        ];

        for i in 0..ops.len() {
            if let Ok(tagged) = str.decode(true, ops[i].0.as_bytes()) {
                assert_eq!(tagged.get_tag(), Cow::from(TAG));

                let val: &str = tagged.as_any().downcast_ref::<StrValue>().unwrap().as_ref();

                assert_eq!(val, ops[i].1);
            } else {
                assert!(false)
            }
        }
    }
}
