extern crate skimmer;


use txt::Twine;
use txt::encoding::{ Unicode, UTF8 };

use model::{ model_issue_rope, Model, Rope, Tagged, TaggedValue };
use model::renderer::{ EncodedString, Node, Renderer };
use model::style::{ CommonStyles, Style };

use std::any::Any;
use std::mem;
use std::iter::Iterator;
// use std::marker::PhantomData;

use std::ptr;


pub const TAG: &'static str = "tag:yaml.org,2002:str";
static TWINE_TAG: Twine = Twine::Static (TAG);


// TODO: do warnings for incorrect escapes on decode (and encode)



#[derive (Clone, Copy)]
pub struct Str;


impl Str {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }

    fn extract_hex_at (&self, src: &[u8], mut at: usize, mut len_limit: u8) -> Option<u32> {
        let mut result: u32 = 0;

        loop {
            if len_limit == 0 { break; }
            len_limit -= 1;

            let val = match src.get (at).map (|b| *b) {
                Some (val @ b'0' ... b'9') => { val - b'0' }
                Some (val @ b'a' ... b'f') => { 10 + (val - b'a') }
                Some (val @ b'A' ... b'F') => { 10 + (val - b'A') }
                _ => return None
            };

            at += 1;

            result = if let Some (nv) = result.checked_mul (16) {
                if let Some (nv) = nv.checked_add (val as u32) {
                    nv
                } else { return None }
            } else { return None }
        }

        return Some (result)
    }


    // TODO: redo it safely
    unsafe fn encode_auto_quoted (&self, mut value: StrValue, tags: &mut Iterator<Item=&(Twine, Twine)>) -> Rope {
        let issue_tag = value.issue_tag ();
        let alias = value.take_alias ();
        let string = value.take_twine ();
        let bytes: &[u8] = string.as_bytes ();
        // let utf8 = UTF8;
        // let char_len = self.encoding.char_max_bytes_len () as usize; // max bytes for a character in the encoding
        let capacity = bytes.len () *  2;  // reserve 2 bytes for escaped with a backslash chars
        let mut result_string: Vec<u8> = Vec::with_capacity (capacity);

        /*
           0 - no quotes
           1 - singles
           2 - doubles
        */
        let mut quotes: u8 = if value.force_quotes () {
            if value.prefer_double_quotes () {
                2
            } else {
                1
            }
        } else { 0 };


        let mut first_rollback_at: Option<(usize, usize)> = None;

        let mut sptr: *const u8 = bytes.as_ptr ();
        let mut slen: usize = 0;

        let mut rptr: *mut u8 = result_string.as_mut_ptr ();
        let mut rlen: usize = 0;

        'main_loop: loop {
            if slen >= bytes.len () { break; }
            if rlen >= capacity { unreachable! () /* overflow! */ }

            // let (code, len) = utf8.to_unicode_ptr (sptr, bytes.len () - slen);
            let code = *sptr;
            slen += 1; // len as usize;
            sptr = sptr.offset (1 as isize);

            match code {
                0 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some ( (rollback_slen, rollback_rlen) ) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr ().offset (slen as isize);
                            rptr = result_string.as_mut_ptr ().offset (rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2; // self.backslash.len () + self.digit_0.len ();
                    result_string.set_len (rlen);

                    ptr::copy_nonoverlapping ("\\0".as_ptr (), rptr, 2);
                    rptr = rptr.offset (2);

                    // rptr = self.backslash.copy_to_ptr (rptr);
                    // rptr = self.digit_0.copy_to_ptr (rptr);
                }


                7 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some ( (rollback_slen, rollback_rlen) ) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr ().offset (slen as isize);
                            rptr = result_string.as_mut_ptr ().offset (rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2; // self.backslash.len () + self.letter_a.len ();
                    result_string.set_len (rlen);

                    ptr::copy_nonoverlapping ("\\a".as_ptr (), rptr, 2);
                    rptr = rptr.offset (2);

                    // rptr = self.backslash.copy_to_ptr (rptr);
                    // rptr = self.letter_a.copy_to_ptr (rptr);
                    
                }


                8 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some ( (rollback_slen, rollback_rlen) ) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr ().offset (slen as isize);
                            rptr = result_string.as_mut_ptr ().offset (rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2; // self.backslash.len () + self.letter_b.len ();
                    result_string.set_len (rlen);

                    ptr::copy_nonoverlapping ("\\b".as_ptr (), rptr, 2);
                    rptr = rptr.offset (2);

                    // rptr = self.backslash.copy_to_ptr (rptr);
                    // rptr = self.letter_b.copy_to_ptr (rptr);
                }


                9 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some ( (rollback_slen, rollback_rlen) ) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr ().offset (slen as isize);
                            rptr = result_string.as_mut_ptr ().offset (rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2; // self.backslash.len () + self.letter_t.len ();
                    result_string.set_len (rlen);

                    ptr::copy_nonoverlapping ("\\t".as_ptr (), rptr, 2);
                    rptr = rptr.offset (2);

                    // rptr = self.backslash.copy_to_ptr (rptr);
                    // rptr = self.letter_t.copy_to_ptr (rptr);
                }


                10 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some ( (rollback_slen, rollback_rlen) ) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr ().offset (slen as isize);
                            rptr = result_string.as_mut_ptr ().offset (rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2; // self.backslash.len () + self.letter_n.len ();
                    result_string.set_len (rlen);

                    ptr::copy_nonoverlapping ("\\n".as_ptr (), rptr, 2);
                    rptr = rptr.offset (2);

                    // rptr = self.backslash.copy_to_ptr (rptr);
                    // rptr = self.letter_n.copy_to_ptr (rptr);
                }


                11 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some ( (rollback_slen, rollback_rlen) ) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr ().offset (slen as isize);
                            rptr = result_string.as_mut_ptr ().offset (rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2; // self.backslash.len () + self.letter_v.len ();
                    result_string.set_len (rlen);

                    ptr::copy_nonoverlapping ("\\v".as_ptr (), rptr, 2);
                    rptr = rptr.offset (2);

                    // rptr = self.backslash.copy_to_ptr (rptr);
                    // rptr = self.letter_v.copy_to_ptr (rptr);
                }


                12 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some ( (rollback_slen, rollback_rlen) ) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr ().offset (slen as isize);
                            rptr = result_string.as_mut_ptr ().offset (rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2; // self.backslash.len () + self.letter_f.len ();
                    result_string.set_len (rlen);

                    ptr::copy_nonoverlapping ("\\f".as_ptr (), rptr, 2);
                    rptr = rptr.offset (2);

                    // rptr = self.backslash.copy_to_ptr (rptr);
                    // rptr = self.letter_f.copy_to_ptr (rptr);
                }


                13 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some ( (rollback_slen, rollback_rlen) ) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr ().offset (slen as isize);
                            rptr = result_string.as_mut_ptr ().offset (rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2; // self.backslash.len () + self.letter_r.len ();
                    result_string.set_len (rlen);

                    ptr::copy_nonoverlapping ("\\r".as_ptr (), rptr, 2);
                    rptr = rptr.offset (2);

                    // rptr = self.backslash.copy_to_ptr (rptr);
                    // rptr = self.letter_r.copy_to_ptr (rptr);
                }


                27 => {
                    if quotes == 0 {
                        quotes = 2;
                    } else if quotes == 1 {
                        quotes = 2;

                        if let Some ( (rollback_slen, rollback_rlen) ) = first_rollback_at {
                            slen = rollback_slen;
                            rlen = rollback_rlen;
                            sptr = bytes.as_ptr ().offset (slen as isize);
                            rptr = result_string.as_mut_ptr ().offset (rlen as isize);
                            continue 'main_loop;
                        }
                    }

                    rlen += 2; // self.backslash.len () + self.letter_e.len ();
                    result_string.set_len (rlen);

                    ptr::copy_nonoverlapping ("\\e".as_ptr (), rptr, 2);
                    rptr = rptr.offset (2);

                    // rptr = self.backslash.copy_to_ptr (rptr);
                    // rptr = self.letter_e.copy_to_ptr (rptr);
                }


                34 => {
                    if quotes == 2 {
                        rlen += 2; // self.backslash.len () + self.d_quote.len ();
                        result_string.set_len (rlen);

                        ptr::copy_nonoverlapping ("\\\"".as_ptr (), rptr, 2);
                        rptr = rptr.offset (2);

                        // rptr = self.backslash.copy_to_ptr (rptr);
                        // rptr = self.d_quote.copy_to_ptr (rptr);

                        continue;
                    }

                    if first_rollback_at.is_none () {
                        first_rollback_at = Some ( (slen - 1, rlen) );
                    }

                    rlen += 1; // self.d_quote.len ();
                    result_string.set_len (rlen);

                    *rptr = b'"';
                    rptr = rptr.offset (1);

                    // rptr = self.d_quote.copy_to_ptr (rptr);
                    // ptr::copy_nonoverlapping ("\"".as_ptr (), rptr, 2);
                    // rptr = rptr.offset (1);
                }


                39 => {
                    if quotes == 1 {
                        rlen += 2; // self.s_quote.len () * 2;
                        result_string.set_len (rlen);

                        // rptr = self.s_quote.copy_to_ptr_times (rptr, 2);
                        ptr::copy_nonoverlapping ("''".as_ptr (), rptr, 2);
                        rptr = rptr.offset (2);

                        continue;
                    }

                    if first_rollback_at.is_none () {
                        first_rollback_at = Some ( (slen - 1, rlen) );
                    }

                    rlen += 1; // self.s_quote.len ();
                    result_string.set_len (rlen);

                    // rptr = self.s_quote.copy_to_ptr (rptr);
                    *rptr = b'\'';
                    rptr = rptr.offset (1);
                }


                92 => {
                    if quotes == 2 {
                        rlen += 2; // self.backslash.len () * 2;
                        result_string.set_len (rlen);

                        // rptr = self.backslash.copy_to_ptr_times (rptr, 2);
                        ptr::copy_nonoverlapping ("\\\\".as_ptr (), rptr, 2);
                        rptr = rptr.offset (2);

                        continue;
                    }

                    if first_rollback_at.is_none () {
                        first_rollback_at = Some ( (slen - 1, rlen) );
                    }

                    rlen += 1; // self.backslash.len ();
                    result_string.set_len (rlen);

                    // rptr = self.backslash.copy_to_ptr (rptr);
                    *rptr = b'\\';
                    rptr = rptr.offset (1);
                }


                _ => {
                    rlen += 1;
                    result_string.set_len (rlen);

                    *rptr = code;
                    rptr = rptr.offset (1);

                    /*
                    let bs = self.encoding.from_unicode (code);

                    rlen += bs[4] as usize;
                    result_string.set_len (rlen);

                    for i in 0 .. bs[4] as usize {
                        *rptr = bs[i];
                        rptr = rptr.offset (1);
                    }
                    */
                }
            };
        }

        let node = if quotes == 2 {
            Node::DoubleQuotedString (EncodedString::from (result_string))
        } else if quotes == 1 {
            Node::SingleQuotedString (EncodedString::from (result_string))
        } else {
            Node::String (EncodedString::from (result_string))
        };

        model_issue_rope (self, node, issue_tag, alias, tags)
    }
}



impl Model for Str {
    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }


    fn is_decodable (&self) -> bool { true }

    fn is_encodable (&self) -> bool { true }


    fn has_default (&self) -> bool { true }

    fn get_default (&self) -> TaggedValue { TaggedValue::from (StrValue::from (String::new ())) }


    fn encode (&self, _renderer: &Renderer, value: TaggedValue, tags: &mut Iterator<Item=&(Twine, Twine)>) -> Result<Rope, TaggedValue> {
        let value: StrValue = match <TaggedValue as Into<Result<StrValue, TaggedValue>>>::into (value) {
            Ok (value) => value,
            Err (value) => return Err (value)
        };

        unsafe { Ok (self.encode_auto_quoted (value, tags)) }
    }


    // TODO: check if value.get_unchecked goes faster
    fn decode (&self, _explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let mut ptr: usize = 0;
        let mut state: u8 = 0;

        const STATE_SPACE: u8 = 1;
        const STATE_BREAK: u8 = 2;

        match value.get (ptr).map (|b| *b) {
            Some (b'"') => {
                let mut result: Vec<u8> = Vec::with_capacity (value.len ());
                let mut spaces: usize = 0;

                ptr += 1;

                loop {
                    match value.get (ptr).map (|b| *b) {
                        None => break,

                        Some (b @ b' ') |
                        Some (b @ b'\t') => {
                            ptr += 1;

                            if state & STATE_BREAK > 0 { continue }

                            spaces += 1;

                            state = state | STATE_SPACE;
                            result.push (b);
                        }

                        Some (b @ b'\n') |
                        Some (b @ b'\r') => {
                            ptr += if b == b'\r' && value.get (ptr + 1).map (|b| *b) == Some (b'\n') { 2 } else { 1 };

                            if spaces > 1 {
                                let len = result.len () - spaces + 1;
                                result.truncate (len);
                            } else if spaces == 0 {
                                spaces = 1;
                                result.push (b' ');
                            }

                            if state & STATE_BREAK > 0 {
                                if spaces > 0 {
                                    let len = result.len () - spaces;
                                    result.truncate (len);
                                    spaces = 0;
                                }

                                result.push (b'\n');
                            }

                            state = state | STATE_BREAK;
                            state = state & !STATE_SPACE;
                        }

                        Some (b'"') => {
                            ptr += 1;

                            if ptr == value.len () {
                                break
                            } else {
                                return Err ( () )
                            }
                        }

                        Some (b'\\') => {
                            ptr += 1;

                            if state & STATE_BREAK > 0 { state = state & !STATE_BREAK; spaces = 0; }
                            if state & STATE_SPACE > 0 { state = state & !STATE_SPACE; spaces = 0; }

                            match value.get (ptr).map (|b| *b) {
                                Some (b @ b'\n') |
                                Some (b @ b'\r') => {
                                    ptr += if b == b'\r' && value.get (ptr + 1).map (|b| *b) == Some (b'\n') { 2 } else { 1 };

                                    spaces = 0;

                                    state = state | STATE_BREAK;
                                    state = state & !STATE_SPACE;
                                }

                                Some (b @ b'\t') |
                                Some (b @ b' ') => {
                                    ptr += 1;

                                    spaces = 0;

                                    state = state & !STATE_BREAK;
                                    state = state & !STATE_SPACE;

                                    result.push (b);
                                }

                                Some (b'0') => { ptr += 1; result.push (0); }
                                Some (b'a') => { ptr += 1; result.push (7); }
                                Some (b'b') => { ptr += 1; result.push (8); }
                                Some (b't') => { ptr += 1; result.push (9); }
                                Some (b'n') => { ptr += 1; result.push (10); }
                                Some (b'v') => { ptr += 1; result.push (11); }
                                Some (b'f') => { ptr += 1; result.push (12); }
                                Some (b'r') => { ptr += 1; result.push (13); }
                                Some (b'e') => { ptr += 1; result.push (27); }
                                Some (b'"') => { ptr += 1; result.push (34); }
                                Some (b'/') => { ptr += 1; result.push (47); }
                                Some (b'\\') => { ptr += 1; result.push (92); }
                                Some (b'N') => { ptr += 1; result.push (0xC2); result.push (0x85); }
                                Some (b'_') => { ptr += 1; result.push (0xC2); result.push (0xA0); }
                                Some (b'L') => { ptr += 1; result.push (0xE2); result.push (0x80); result.push (0xA8); }
                                Some (b'P') => { ptr += 1; result.push (0xE2); result.push (0x80); result.push (0xA9); }

                                Some (b'x') => {
                                    // ptr += 1;

                                    if let Some (code) = self.extract_hex_at (value, ptr + 1, 2) {
                                        ptr += 3;
                                        let code = UTF8.from_unicode (code);
                                        result.extend (&code[ .. code[4] as usize]);
                                    } else { result.push (b'\\'); }
                                }

                                Some (b'u') => {
                                    // ptr += 1;

                                    if let Some (code) = self.extract_hex_at (value, ptr + 1, 4) {
                                        ptr += 5;
                                        let code = UTF8.from_unicode (code);
                                        result.extend (&code[ .. code[4] as usize]);
                                    } else { result.push (b'\\'); }
                                }

                                Some (b'U') => {
                                    // ptr += 1;

                                    if let Some (code) = self.extract_hex_at (value, ptr + 1, 8) {
                                        ptr += 9;
                                        let code = UTF8.from_unicode (code);
                                        result.extend (&code[ .. code[4] as usize]);
                                    } else { result.push (b'\\'); }
                                }

                                _ => { result.push (b'\\'); }
                            }
                        }

                        Some (b @ _) => {
                            ptr += 1;

                            if state & STATE_BREAK > 0 { state = state & !STATE_BREAK; spaces = 0; }
                            if state & STATE_SPACE > 0 { state = state & !STATE_SPACE; spaces = 0; }

                            result.push (b);
                        }
                    }
                }

                Ok ( TaggedValue::from (StrValue::from (unsafe { String::from_utf8_unchecked (result) })) )
            }

            Some (b'\'') => {
                let mut result: Vec<u8> = Vec::with_capacity (value.len ());

                let mut spaces: usize = 0;

                ptr += 1;

                loop {
                    match value.get (ptr).map (|b| *b) {
                        None => break,

                        Some (b @ b' ') |
                        Some (b @ b'\t') => {
                            ptr += 1;

                            if state & STATE_BREAK > 0 { continue }

                            spaces += 1;

                            state = state | STATE_SPACE;
                            result.push (b);
                        }

                        Some (b @ b'\n') |
                        Some (b @ b'\r') => {
                            ptr += if b == b'\r' && value.get (ptr + 1).map (|b| *b) == Some (b'\n') { 2 } else { 1 };

                            if spaces > 1 {
                                let len = result.len () - spaces + 1;
                                result.truncate (len);
                            } else if spaces == 0 {
                                spaces = 1;
                                result.push (b' ');
                            }

                            if state & STATE_BREAK > 0 {
                                if spaces > 0 {
                                    let len = result.len () - spaces;
                                    result.truncate (len);
                                    spaces = 0;
                                }

                                result.push (b'\n');
                            }

                            state = state | STATE_BREAK;
                            state = state & !STATE_SPACE;
                        }

                        Some (b'\'') => {
                            ptr += 1;

                            match value.get (ptr).map (|b| *b) {
                                Some (b'\'') => {
                                    ptr += 1;
                                    result.push (b'\'');
                                }
                                None => break,
                                _ => return Err ( () )
                            };
                        }

                        Some (b @ _) => {
                            ptr += 1;

                            if state & STATE_BREAK > 0 { state = state & !STATE_BREAK; spaces = 0; }
                            if state & STATE_SPACE > 0 { state = state & !STATE_SPACE; spaces = 0; }

                            result.push (b);
                        }
                    }
                }

                Ok ( TaggedValue::from (StrValue::from (unsafe { String::from_utf8_unchecked (result) })) )
            }

            _ => Ok ( TaggedValue::from (StrValue::from (unsafe { String::from_utf8_unchecked (Vec::from (value)) })) )
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



pub const FORCE_QUOTES: ForceQuotes = ForceQuotes (true);
pub const NO_FORCE_QUOTES: ForceQuotes = ForceQuotes (false);

pub struct ForceQuotes (pub bool);

impl Style for ForceQuotes {
    fn tagged_styles_apply (&mut self, value: &mut Tagged) {
        if value.get_tag ().as_ref () != TAG { return }

        if let Some (ref mut str_val) = value.as_mut_any ().downcast_mut::<StrValue> () {
            str_val.set_force_quotes (self.0);
        }
    }
}



pub const PREFER_DOUBLE_QUOTES: PreferDoubleQuotes = PreferDoubleQuotes (true);
pub const NO_PREFER_DOUBLE_QUOTES: PreferDoubleQuotes = PreferDoubleQuotes (false);


pub struct PreferDoubleQuotes (pub bool);

impl Style for PreferDoubleQuotes {
    fn tagged_styles_apply (&mut self, value: &mut Tagged) {
        if value.get_tag ().as_ref () != TAG { return }

        if let Some (ref mut str_val) = value.as_mut_any ().downcast_mut::<StrValue> () {
            str_val.set_prefer_double_quotes (self.0);
        }
    }
}




#[derive (Debug)]
pub struct StrValue {
    style: u8,

    alias: Option<Twine>,

    value: Twine
}



impl StrValue {
    pub fn new (val: Twine, styles: CommonStyles, alias: Option<Twine>) -> StrValue { StrValue {
        style: if styles.issue_tag () { 1 } else { 0 },
        alias: alias,
        value: val
    } }

    pub fn issue_tag (&self) -> bool { self.style & 1 == 1 }

    pub fn set_issue_tag (&mut self, val: bool) { if val { self.style |= 1; } else { self.style &= !1; } }

    pub fn force_quotes (&self) -> bool { self.style & 2 == 2 }

    pub fn set_force_quotes (&mut self, val: bool) { if val { self.style |= 2; } else { self.style &= !2; } }

    pub fn prefer_double_quotes (&self) -> bool { self.style & 4 == 4 }

    pub fn set_prefer_double_quotes (&mut self, val: bool) { if val { self.style |= 4; } else { self.style &= !4; } }

    pub fn take_twine (&mut self) -> Twine { mem::replace (&mut self.value, Twine::empty ()) }

    pub fn take_alias (&mut self) -> Option<Twine> { self.alias.take () }

    pub fn to_twine (self) -> Twine { self.value }
}



impl Tagged for StrValue {
    fn get_tag (&self) -> &Twine { &TWINE_TAG }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



impl From<char> for StrValue {
    fn from (value: char) -> StrValue { StrValue { style: 0, alias: None, value: Twine::from (value.to_string ()) } }
}



impl From<Twine> for StrValue {
    fn from (value: Twine) -> StrValue { StrValue { style: 0, alias: None, value: value } }
}



impl From<String> for StrValue {
    fn from (value: String) -> StrValue { StrValue { style: 0, alias: None, value: Twine::from (value) } }
}


impl From<&'static str> for StrValue {
    fn from (value: &'static str) -> StrValue { StrValue { style: 0, alias: None, value: Twine::from (value) } }
}



impl AsRef<str> for StrValue {
    fn as_ref (&self) -> &str { self.value.as_ref () }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use model::{ Tagged, Renderer };
    // use txt::get_charset_utf8;

    use std::iter;



    #[test]
    fn tag () {
        // let str = StrFactory.build_model (&get_charset_utf8 ());
        let str = Str; // ::new (&get_charset_utf8 ());

        assert_eq! (str.get_tag (), TAG);
    }



    #[test]
    fn encode () {
        let renderer = Renderer; // ::new (&get_charset_utf8 ());
        let str = Str; // ::new (&get_charset_utf8 ());


        let ops = [
            ("Hey, this is a string!", "Hey, this is a string!"),
            ("Hey,\nthis is\tanother\" one", r#""Hey,\nthis is\tanother\" one""#)
        ];


        for i in 0 .. ops.len () {
            if let Ok (rope) = str.encode (&renderer, TaggedValue::from (StrValue::from (ops[i].0.to_string ())), &mut iter::empty ()) {
                let vec = rope.render (&renderer);
                assert_eq! (vec, ops[i].1.to_string ().into_bytes ().to_vec ());
            } else { assert! (false) }

            if let Ok (rope) = str.encode (&renderer, TaggedValue::from (StrValue::from (ops[i].0)), &mut iter::empty ()) {
                let vec = rope.render (&renderer);
                assert_eq! (vec, ops[i].1.to_string ().into_bytes ().to_vec ());
            } else { assert! (false) }
        }
    }



    #[test]
    fn decode () {
        let str = Str; // ::new (&get_charset_utf8 ());

        let ops = [
            ("Hey, this is a string!", "Hey, this is a string!"),
            (r"'Hey, that''s the string!'", "Hey, that's the string!"),
            (r#""Hey, that\"s the string!""#, "Hey, that\"s the string!"),
            (r#""Hey,\n\ that's\tthe\0string\\""#, "Hey,\n that's\tthe\0string\\"),
            (r#""This\x0Ais\x09a\x2c\x20test""#, "This\nis\ta, test"),
            (r#""\u0422\u0435\u0441\u0442\x0a""#, "Тест\n"),
            (r#""\u30c6\u30b9\u30c8\x0a""#, "テスト\n"),
            (r#""\U00013000\U00013001\U00013002\U00013003\U00013004\U00013005\U00013006\U00013007""#, "𓀀𓀁𓀂𓀃𓀄𓀅𓀆𓀇")
        ];


        for i in 0 .. ops.len () {
            if let Ok (tagged) = str.decode (true, ops[i].0.as_bytes ()) {
                assert_eq! (tagged.get_tag (), &TWINE_TAG);

                let val: &str = tagged.as_any ().downcast_ref::<StrValue> ().unwrap ().as_ref ();

                assert_eq! (val, ops[i].1);
            } else { assert! (false) }
        }
    }



    #[test]
    fn folding () {
        let str = Str; // ::new (&get_charset_utf8 ());

        let ops = [
            (r#""Four    spaces""#, "Four    spaces"),
            (r#""Four  \  spaces""#, "Four    spaces"),
            (r#""Four  
  spaces""#, "Four spaces"),
(r#"" 1st non-empty

 2nd non-empty 
	3rd non-empty ""#, " 1st non-empty\n2nd non-empty 3rd non-empty "),

            (r#""folded 
to a space,	
 
to a line feed, or 	\
 \ 	non-content""#, "folded to a space,\nto a line feed, or \t \tnon-content"),


(r#""
  foo 
 
  	 bar

  baz
""#, " foo\nbar\nbaz ")
        ];


        for i in 0 .. ops.len () {
            if let Ok (tagged) = str.decode (true, ops[i].0.as_bytes ()) {
                assert_eq! (tagged.get_tag (), &TWINE_TAG);

                let val: &str = tagged.as_any ().downcast_ref::<StrValue> ().unwrap ().as_ref ();

                assert_eq! (val, ops[i].1);
            } else { assert! (false) }
        }
    }
}
