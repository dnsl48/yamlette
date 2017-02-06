extern crate skimmer;

use self::skimmer::symbol::{ Char, Symbol };


use txt::{ CharSet, Twine };
use txt::encoding::{ Encoding, Unicode, UTF8 };

use model::{ model_issue_rope, Factory, Model, Rope, Tagged, TaggedValue };
use model::renderer::{ EncodedString, Node, Renderer };
use model::style::{ CommonStyles, Style };

use std::any::Any;
use std::mem;
use std::iter::Iterator;


pub const TAG: &'static str = "tag:yaml.org,2002:str";
static TWINE_TAG: Twine = Twine::Static (TAG);


// TODO: do warnings for incorrect escapes on decode (and encode)


pub struct Str {
    encoding: Encoding,

    s_quote: Char,
    d_quote: Char,

    backslash: Char,

    line_feed: Char,
    carriage_return: Char,

    slash: Char,
    space: Char,
    tab: Char,
    underscore: Char,

    digit_0: Char,
    digit_1: Char,
    digit_2: Char,
    digit_3: Char,
    digit_4: Char,
    digit_5: Char,
    digit_6: Char,
    digit_7: Char,
    digit_8: Char,
    digit_9: Char,

    letter_a: Char,
    letter_b: Char,
    letter_c: Char,
    letter_d: Char,
    letter_e: Char,
    letter_f: Char,
    letter_n: Char,
    letter_r: Char,
    letter_t: Char,
    letter_u: Char,
    letter_v: Char,
    letter_x: Char,

    letter_t_a: Char,
    letter_t_b: Char,
    letter_t_c: Char,
    letter_t_d: Char,
    letter_t_e: Char,
    letter_t_f: Char,
    letter_t_l: Char,
    letter_t_n: Char,
    letter_t_p: Char,
    letter_t_u: Char
}



impl Str {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }


    pub fn new (cset: &CharSet) -> Str {
        Str {
            encoding: cset.encoding,

            s_quote: cset.apostrophe.clone (),
            d_quote: cset.quotation.clone (),

            backslash: cset.backslash.clone (),

            line_feed: cset.line_feed.clone (),
            carriage_return: cset.carriage_return.clone (),

            slash: cset.slash.clone (),
            space: cset.space.clone (),
            tab: cset.tab_h.clone (),
            underscore: cset.low_line.clone (),

            digit_0: cset.digit_0.clone (),
            digit_1: cset.digit_1.clone (),
            digit_2: cset.digit_2.clone (),
            digit_3: cset.digit_3.clone (),
            digit_4: cset.digit_4.clone (),
            digit_5: cset.digit_5.clone (),
            digit_6: cset.digit_6.clone (),
            digit_7: cset.digit_7.clone (),
            digit_8: cset.digit_8.clone (),
            digit_9: cset.digit_9.clone (),

            letter_a: cset.letter_a.clone (),
            letter_b: cset.letter_b.clone (),
            letter_c: cset.letter_c.clone (),
            letter_d: cset.letter_d.clone (),
            letter_e: cset.letter_e.clone (),
            letter_f: cset.letter_f.clone (),
            letter_n: cset.letter_n.clone (),
            letter_r: cset.letter_r.clone (),
            letter_t: cset.letter_t.clone (),
            letter_u: cset.letter_u.clone (),
            letter_v: cset.letter_v.clone (),
            letter_x: cset.letter_x.clone (),

            letter_t_a: cset.letter_t_a.clone (),
            letter_t_b: cset.letter_t_b.clone (),
            letter_t_c: cset.letter_t_c.clone (),
            letter_t_d: cset.letter_t_d.clone (),
            letter_t_e: cset.letter_t_e.clone (),
            letter_t_f: cset.letter_t_f.clone (),
            letter_t_l: cset.letter_t_l.clone (),
            letter_t_n: cset.letter_t_n.clone (),
            letter_t_p: cset.letter_t_p.clone (),
            letter_t_u: cset.letter_t_u.clone ()
        }
    }


    fn extract_hex_at (&self, src: &[u8], at: usize) -> Option<(u8, usize)> {
        // TODO: effective specialisations for UTF-8, UTF-16 and UTF-32
        // TODO: or keep a charset copy and just use it here?

             if self.digit_0.contained_at (&src, at) { Some ( (0, self.digit_0.len ()) ) }
        else if self.digit_1.contained_at (&src, at) { Some ( (1, self.digit_1.len ()) ) }
        else if self.digit_2.contained_at (&src, at) { Some ( (2, self.digit_2.len ()) ) }
        else if self.digit_3.contained_at (&src, at) { Some ( (3, self.digit_3.len ()) ) }
        else if self.digit_4.contained_at (&src, at) { Some ( (4, self.digit_4.len ()) ) }
        else if self.digit_5.contained_at (&src, at) { Some ( (5, self.digit_5.len ()) ) }
        else if self.digit_6.contained_at (&src, at) { Some ( (6, self.digit_6.len ()) ) }
        else if self.digit_7.contained_at (&src, at) { Some ( (7, self.digit_7.len ()) ) }
        else if self.digit_8.contained_at (&src, at) { Some ( (8, self.digit_8.len ()) ) }
        else if self.digit_9.contained_at (&src, at) { Some ( (9, self.digit_9.len ()) ) }

        else if self.letter_a.contained_at (&src, at) { Some ( (10, self.letter_a.len ()) ) }
        else if self.letter_b.contained_at (&src, at) { Some ( (11, self.letter_b.len ()) ) }
        else if self.letter_c.contained_at (&src, at) { Some ( (12, self.letter_c.len ()) ) }
        else if self.letter_d.contained_at (&src, at) { Some ( (13, self.letter_d.len ()) ) }
        else if self.letter_e.contained_at (&src, at) { Some ( (14, self.letter_e.len ()) ) }
        else if self.letter_f.contained_at (&src, at) { Some ( (15, self.letter_f.len ()) ) }

        else if self.letter_t_a.contained_at (&src, at) { Some ( (10, self.letter_t_a.len ()) ) }
        else if self.letter_t_b.contained_at (&src, at) { Some ( (11, self.letter_t_b.len ()) ) }
        else if self.letter_t_c.contained_at (&src, at) { Some ( (12, self.letter_t_c.len ()) ) }
        else if self.letter_t_d.contained_at (&src, at) { Some ( (13, self.letter_t_d.len ()) ) }
        else if self.letter_t_e.contained_at (&src, at) { Some ( (14, self.letter_t_e.len ()) ) }
        else if self.letter_t_f.contained_at (&src, at) { Some ( (15, self.letter_t_f.len ()) ) }

        else { None }
    }


    unsafe fn encode_auto_quoted (&self, mut value: StrValue, tags: &mut Iterator<Item=&(Twine, Twine)>) -> Rope {
        let issue_tag = value.issue_tag ();
        let alias = value.take_alias ();
        let string = value.take_twine ();
        let bytes: &[u8] = string.as_bytes ();
        let utf8 = UTF8;
        let char_len = self.encoding.char_max_bytes_len () as usize; // max bytes for a character in the encoding
        let capacity = bytes.len () * char_len * 2;  // 2 bytes for escaped with a backslash chars
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

            let (code, len) = utf8.to_unicode_ptr (sptr, bytes.len () - slen);
            slen += len as usize;
            sptr = sptr.offset (len as isize);

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

                    rlen += self.backslash.len () + self.digit_0.len ();
                    result_string.set_len (rlen);

                    rptr = self.backslash.copy_to_ptr (rptr);
                    rptr = self.digit_0.copy_to_ptr (rptr);
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

                    rlen += self.backslash.len () + self.letter_a.len ();
                    result_string.set_len (rlen);

                    rptr = self.backslash.copy_to_ptr (rptr);
                    rptr = self.letter_a.copy_to_ptr (rptr);
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

                    rlen += self.backslash.len () + self.letter_b.len ();
                    result_string.set_len (rlen);

                    rptr = self.backslash.copy_to_ptr (rptr);
                    rptr = self.letter_b.copy_to_ptr (rptr);
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

                    rlen += self.backslash.len () + self.letter_t.len ();
                    result_string.set_len (rlen);

                    rptr = self.backslash.copy_to_ptr (rptr);
                    rptr = self.letter_t.copy_to_ptr (rptr);
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

                    rlen += self.backslash.len () + self.letter_n.len ();
                    result_string.set_len (rlen);

                    rptr = self.backslash.copy_to_ptr (rptr);
                    rptr = self.letter_n.copy_to_ptr (rptr);
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

                    rlen += self.backslash.len () + self.letter_v.len ();
                    result_string.set_len (rlen);

                    rptr = self.backslash.copy_to_ptr (rptr);
                    rptr = self.letter_v.copy_to_ptr (rptr);
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

                    rlen += self.backslash.len () + self.letter_f.len ();
                    result_string.set_len (rlen);

                    rptr = self.backslash.copy_to_ptr (rptr);
                    rptr = self.letter_f.copy_to_ptr (rptr);
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

                    rlen += self.backslash.len () + self.letter_r.len ();
                    result_string.set_len (rlen);

                    rptr = self.backslash.copy_to_ptr (rptr);
                    rptr = self.letter_r.copy_to_ptr (rptr);
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

                    rlen += self.backslash.len () + self.letter_e.len ();
                    result_string.set_len (rlen);

                    rptr = self.backslash.copy_to_ptr (rptr);
                    rptr = self.letter_e.copy_to_ptr (rptr);
                }


                34 => {
                    if quotes == 2 {
                        rlen += self.backslash.len () + self.d_quote.len ();
                        result_string.set_len (rlen);

                        rptr = self.backslash.copy_to_ptr (rptr);
                        rptr = self.d_quote.copy_to_ptr (rptr);

                        continue;
                    }

                    if first_rollback_at.is_none () {
                        first_rollback_at = Some ( (slen - len as usize, rlen) );
                    }

                    rlen += self.d_quote.len ();
                    result_string.set_len (rlen);

                    rptr = self.d_quote.copy_to_ptr (rptr);
                }


                39 => {
                    if quotes == 1 {
                        rlen += self.s_quote.len () * 2;
                        result_string.set_len (rlen);

                        rptr = self.s_quote.copy_to_ptr_times (rptr, 2);

                        continue;
                    }

                    if first_rollback_at.is_none () {
                        first_rollback_at = Some ( (slen - len as usize, rlen) );
                    }

                    rlen += self.s_quote.len ();
                    result_string.set_len (rlen);

                    rptr = self.s_quote.copy_to_ptr (rptr);
                }


                92 => {
                    if quotes == 2 {
                        rlen += self.backslash.len () * 2;
                        result_string.set_len (rlen);

                        rptr = self.backslash.copy_to_ptr_times (rptr, 2);

                        continue;
                    }

                    if first_rollback_at.is_none () {
                        first_rollback_at = Some ( (slen - len as usize, rlen) );
                    }

                    rlen += self.backslash.len ();
                    result_string.set_len (rlen);

                    rptr = self.backslash.copy_to_ptr (rptr);
                }


                _ => {
                    let bs = self.encoding.from_unicode (code);

                    rlen += bs[4] as usize;
                    result_string.set_len (rlen);

                    for i in 0 .. bs[4] as usize {
                        *rptr = bs[i];
                        rptr = rptr.offset (1);
                    }
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

    fn get_encoding (&self) -> Encoding { self.encoding }

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


    fn decode (&self, _explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let mut ptr: usize = 0;

        let utf8 = UTF8;
        let clen = match self.encoding { Encoding::UTF8 (_) => 1 };


        let mut buffer: Vec<u8> = Vec::with_capacity (value.len () * clen);
        let mut result: Vec<u8> = Vec::with_capacity (value.len () * clen);


        let mut state: u8 = 0;

        const STATE_SPACE: u8 = 1;
        const STATE_ESCNL: u8 = 2;
        const STATE_BREAK: u8 = 4;


        if self.d_quote.contained_at (value, ptr) {
            ptr += self.d_quote.len ();

            loop {
                if ptr >= value.len () { break; }


                let (code, len): (u32, u8) = if self.space.contained_at (value, ptr) {
                    state = state | STATE_SPACE;

                    (32, (self.space.len ()) as u8)
                } else

                if self.tab.contained_at (value, ptr) {
                    state = state | STATE_SPACE;

                    (9, (self.tab.len ()) as u8)
                } else

                if self.backslash.contained_at (value, ptr) && self.line_feed.contained_at (value, ptr + self.backslash.len ())
                {
                    state = state | STATE_ESCNL;

                    ptr += self.backslash.len () + self.line_feed.len ();

                    continue;
                } else

                if self.backslash.contained_at (value, ptr) && self.carriage_return.contained_at (value, ptr + self.backslash.len ())
                {
                    state = state | STATE_ESCNL;

                    ptr += self.backslash.len () + self.line_feed.len ();

                    continue;
                } else

                if self.line_feed.contained_at (value, ptr) {
                    if state & STATE_BREAK == 0 {
                        buffer.clear ();

                        state = state | STATE_BREAK | STATE_SPACE;

                        ptr += self.line_feed.len ();

                        continue;
                    }

                    (10, self.line_feed.len () as u8)
                } else

                if self.carriage_return.contained_at (value, ptr) {
                    if state & STATE_BREAK == 0 {
                        state = state | STATE_BREAK | STATE_SPACE;

                        buffer.clear ();

                        ptr += self.carriage_return.len ();

                        continue;
                    }

                    (13, self.carriage_return.len () as u8)
                } else


                if state & STATE_SPACE == STATE_SPACE {
                    if state & STATE_BREAK == STATE_BREAK && buffer.len () == 0 {
                        result.push (b' '); 
                    } else {
                        result.append (&mut buffer);
                    }

                    state = 0;

                    continue;
                } else { (0, 0) };



                if state & STATE_SPACE == STATE_SPACE {
                    ptr += len as usize;

                    if state & STATE_ESCNL == STATE_ESCNL {
                        continue;
                    } else {
                        if state & STATE_BREAK == STATE_BREAK && code != 10 && code != 13 {
                            continue;
                        } else {
                            let bs = utf8.from_unicode (code);
                            buffer.extend (&bs[.. bs[4] as usize]);
                        }
                    }

                    continue;
                }

                let (code, len): (u32, u8) = if self.backslash.contained_at (value, ptr) {
                    let ptr = ptr + self.backslash.len ();

                    if self.digit_0.contained_at (value, ptr) {
                        (0, (self.backslash.len () + self.digit_0.len ()) as u8)
                    } else

                    if self.letter_a.contained_at (value, ptr) {
                        (7, (self.backslash.len () + self.letter_a.len ()) as u8)
                    } else

                    if self.letter_b.contained_at (value, ptr) {
                        (8, (self.backslash.len () + self.letter_b.len ()) as u8)
                    } else

                    if self.letter_t.contained_at (value, ptr) {
                        (9, (self.backslash.len () + self.letter_t.len ()) as u8)
                    } else

                    if self.letter_n.contained_at (value, ptr) {
                        (10, (self.backslash.len () + self.letter_n.len ()) as u8)
                    } else

                    if self.letter_v.contained_at (value, ptr) {
                        (11, (self.backslash.len () + self.letter_v.len ()) as u8)
                    } else

                    if self.letter_f.contained_at (value, ptr) {
                        (12, (self.backslash.len () + self.letter_f.len ()) as u8)
                    } else

                    if self.letter_r.contained_at (value, ptr) {
                        (13, (self.backslash.len () + self.letter_r.len ()) as u8)
                    } else

                    if self.letter_e.contained_at (value, ptr) {
                        (27, (self.backslash.len () + self.letter_e.len ()) as u8)
                    } else

                    if self.space.contained_at (value, ptr) {
                        (32, (self.backslash.len () + self.space.len ()) as u8)
                    } else

                    if self.d_quote.contained_at (value, ptr) {
                        (34, (self.backslash.len () + self.d_quote.len ()) as u8)
                    } else

                    if self.slash.contained_at (value, ptr) {
                        (47, (self.backslash.len () + self.slash.len ()) as u8)
                    } else

                    if self.backslash.contained_at (value, ptr) {
                        (92, (self.backslash.len () + self.backslash.len ()) as u8)
                    } else

                    if self.letter_t_n.contained_at (value, ptr) {
                        (133, (self.backslash.len () + self.letter_t_n.len ()) as u8)
                    } else

                    if self.underscore.contained_at (value, ptr) {
                        (160, (self.backslash.len () + self.underscore.len ()) as u8)
                    } else

                    if self.letter_t_l.contained_at (value, ptr) {
                        (8232, (self.backslash.len () + self.letter_t_l.len ()) as u8)
                    } else

                    if self.letter_t_p.contained_at (value, ptr) {
                        (8233, (self.backslash.len () + self.letter_t_p.len ()) as u8)
                    } else

                    if self.letter_x.contained_at (value, ptr) {
                        let ptr = ptr + self.letter_x.len ();

                        self.extract_hex_at (value, ptr).and_then (|(d1, l1)| {
                        self.extract_hex_at (value, ptr + l1).and_then (|(d2, l2)| {
                            Some ( ((d1 * 16 + d2) as u32, (self.backslash.len () + self.letter_x.len () + l1 + l2) as u8) )
                        })
                        }).or_else (|| { Some ( (92, self.backslash.len () as u8) ) }).unwrap ()

                    } else

                    if self.letter_u.contained_at (value, ptr) {
                        let ptr = ptr + self.letter_u.len ();

                        self.extract_hex_at (value, ptr).and_then (|(d1, l1)| {
                        self.extract_hex_at (value, ptr + l1).and_then (|(d2, l2)| {
                        self.extract_hex_at (value, ptr + l1 + l2).and_then (|(d3, l3)| {
                        self.extract_hex_at (value, ptr + l1 + l2 + l3).and_then (|(d4, l4)| {
                            Some ( (
                                (d4 as u16 + (d3 as u16 * 16) + (d2 as u16 * 16 * 16) + (d1 as u16 * 16 * 16 * 16)) as u32,
                                (self.backslash.len () + self.letter_u.len () + l1 + l2 + l3 + l4) as u8
                            ) )
                        })
                        })
                        })
                        }).or_else (|| { Some ( (92, self.backslash.len () as u8) ) }).unwrap ()

                    } else

                    if self.letter_t_u.contained_at (value, ptr) {
                        let ptr = ptr + self.letter_t_u.len ();

                        self.extract_hex_at (value, ptr).and_then (|(d1, l1)| {
                        self.extract_hex_at (value, ptr + l1).and_then (|(d2, l2)| {
                        self.extract_hex_at (value, ptr + l1 + l2).and_then (|(d3, l3)| {
                        self.extract_hex_at (value, ptr + l1 + l2 + l3).and_then (|(d4, l4)| {
                        self.extract_hex_at (value, ptr + l1 + l2 + l3 + l4).and_then (|(d5, l5)| {
                        self.extract_hex_at (value, ptr + l1 + l2 + l3 + l4 + l5).and_then (|(d6, l6)| {
                        self.extract_hex_at (value, ptr + l1 + l2 + l3 + l4 + l5 + l6).and_then (|(d7, l7)| {
                        self.extract_hex_at (value, ptr + l1 + l2 + l3 + l4 + l5 + l6 + l7).and_then (|(d8, l8)| {
                            Some ( (
                                d8 as u32 +
                                d7 as u32 * 16 +
                                d6 as u32 * 16 * 16 +
                                d5 as u32 * 16 * 16 * 16 +
                                d4 as u32 * 16 * 16 * 16 * 16 +
                                d3 as u32 * 16 * 16 * 16 * 16 * 16 +
                                d2 as u32 * 16 * 16 * 16 * 16 * 16 * 16 +
                                d1 as u32 * 16 * 16 * 16 * 16 * 16 * 16 * 16,
                                (self.backslash.len () + self.letter_t_u.len () + l1 + l2 + l3 + l4 + l5 + l6 + l7 + l8) as u8
                            ) )
                        })
                        })
                        })
                        })
                        })
                        })
                        })
                        }).or_else (|| { Some ( (92, self.backslash.len () as u8) ) }).unwrap ()

                    } else


                    { (92, self.backslash.len () as u8) }
                } else {
                    self.encoding.to_unicode (&value[ptr ..])
                };


                ptr += len as usize;


                if ptr == value.len () && code == 34 { break; }


                let bs = utf8.from_unicode (code);
                result.extend (&bs[.. bs[4] as usize]);
            }

        } else if self.s_quote.contained_at (value, ptr) {
            ptr += self.s_quote.len ();

            loop {
                if ptr >= value.len () { break; }

                let (code, len): (u32, u8) = if self.space.contained_at (value, ptr) {
                    state = state | STATE_SPACE;

                    (32, (self.space.len ()) as u8)
                } else

                if self.tab.contained_at (value, ptr) {
                    state = state | STATE_SPACE;

                    (9, (self.tab.len ()) as u8)
                } else

                if self.backslash.contained_at (value, ptr) && self.line_feed.contained_at (value, ptr + self.backslash.len ())
                {
                    state = state | STATE_ESCNL;

                    ptr += self.backslash.len () + self.line_feed.len ();

                    continue;
                } else

                if self.backslash.contained_at (value, ptr) && self.carriage_return.contained_at (value, ptr + self.backslash.len ())
                {
                    state = state | STATE_ESCNL;

                    ptr += self.backslash.len () + self.line_feed.len ();

                    continue;
                } else

                if self.line_feed.contained_at (value, ptr) {
                    if state & STATE_BREAK == 0 {
                        buffer.clear ();

                        state = state | STATE_BREAK | STATE_SPACE;

                        ptr += self.line_feed.len ();

                        continue;
                    }

                    (10, self.line_feed.len () as u8)
                } else

                if self.carriage_return.contained_at (value, ptr) {
                    if state & STATE_BREAK == 0 {
                        state = state | STATE_BREAK | STATE_SPACE;

                        buffer.clear ();

                        ptr += self.carriage_return.len ();

                        continue;
                    }

                    (13, self.carriage_return.len () as u8)
                } else


                if state & STATE_SPACE == STATE_SPACE {
                    if state & STATE_BREAK == STATE_BREAK && buffer.len () == 0 {
                        result.push (b' '); 
                    } else {
                        result.append (&mut buffer);
                    }

                    state = 0;

                    continue;
                } else { (0, 0) };



                if state & STATE_SPACE == STATE_SPACE {
                    ptr += len as usize;

                    if state & STATE_ESCNL == STATE_ESCNL {
                        continue;
                    } else {
                        if state & STATE_BREAK == STATE_BREAK && code != 10 && code != 13 {
                            continue;
                        } else {
                            let bs = utf8.from_unicode (code);
                            buffer.extend (&bs[.. bs[4] as usize]);
                        }
                    }

                    continue;
                }

                let (code, len): (u32, u8) = if self.backslash.contained_at (value, ptr) && self.s_quote.contained_at (value, ptr + self.backslash.len ())
                {
                    (39, (self.backslash.len () + self.s_quote.len ()) as u8)
                } else if self.s_quote.contained_at (value, ptr) && self.s_quote.contained_at (value, ptr + self.s_quote.len ()) {
                    (39, (self.s_quote.len () * 2) as u8)
                } else {
                    self.encoding.to_unicode (&value[ptr ..])
                };


                ptr += len as usize;


                if ptr == value.len () && code == 39 { break; }


                let bs = utf8.from_unicode (code);
                result.extend (&bs[.. bs[4] as usize]);
            }
        } else {
            loop {
                if ptr >= value.len () { break; }

                let (code, len) = self.encoding.to_unicode (&value[ptr ..]);

                ptr += len as usize;

                let bs = utf8.from_unicode (code);
                result.extend (&bs[.. bs[4] as usize]);
            }
        }


        let s = String::from_utf8 (result);

        if s.is_err () { Err ( () ) }
        else { Ok ( TaggedValue::from (StrValue::from (s.unwrap ())) ) }
    }
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
    fn get_tag (&self) -> &Twine { Str::get_tag () }

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




pub struct StrFactory;



impl Factory for StrFactory {
    fn get_tag (&self) -> &Twine { Str::get_tag () }

    fn build_model (&self, cset: &CharSet) -> Box<Model> { Box::new (Str::new (cset)) }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use model::{ Factory, Tagged, Renderer };
    use txt::get_charset_utf8;

    use std::iter;



    #[test]
    fn tag () {
        let str = StrFactory.build_model (&get_charset_utf8 ());

        assert_eq! (str.get_tag (), TAG);
    }



    #[test]
    fn encode () {
        let renderer = Renderer::new (&get_charset_utf8 ());
        let str = Str::new (&get_charset_utf8 ());


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
        let str = Str::new (&get_charset_utf8 ());

        let ops = [
            ("Hey, this is a string!", "Hey, this is a string!"),
            (r"'Hey, that\'s the string!'", "Hey, that's the string!"),
            (r#""Hey,\n\ that's\tthe\0string\\""#, "Hey,\n that's\tthe\0string\\"),
            (r#""This\x0Ais\x09a\x2c\x20test""#, "This\nis\ta, test"),
            (r#""\u0422\u0435\u0441\u0442\x0a""#, "Тест\n"),
            (r#""\u30c6\u30b9\u30c8\x0a""#, "テスト\n"),
            (r#""\U00013000\U00013001\U00013002\U00013003\U00013004\U00013005\U00013006\U00013007""#, "𓀀𓀁𓀂𓀃𓀄𓀅𓀆𓀇")
        ];


        for i in 0 .. ops.len () {
            if let Ok (tagged) = str.decode (true, ops[i].0.as_bytes ()) {
                assert_eq! (tagged.get_tag (), Str::get_tag ());

                let val: &str = tagged.as_any ().downcast_ref::<StrValue> ().unwrap ().as_ref ();

                assert_eq! (val, ops[i].1);
            } else { assert! (false) }
        }
    }



    #[test]
    fn folding () {
        let str = Str::new (&get_charset_utf8 ());

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
                assert_eq! (tagged.get_tag (), Str::get_tag ());

                let val: &str = tagged.as_any ().downcast_ref::<StrValue> ().unwrap ().as_ref ();

                assert_eq! (val, ops[i].1);
            } else { assert! (false) }
        }
    }
}
