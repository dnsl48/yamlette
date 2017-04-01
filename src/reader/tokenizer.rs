extern crate skimmer;


// extern crate gauger;
// use self::gauger::sample::{ Sample, Timer };


use self::skimmer::reader::Read;
// use self::skimmer::scanner::{ Quote, scan, scan_one_at, scan_quoted, scan_quoted_selfescape, scan_while, scan_until, scan_until_noidx, scan_until_at, scan_until_at_noidx };
// use self::skimmer::scanner::{ scan_quoted, scan_quoted_selfescape };
// use self::skimmer::symbol::{ /*Char, Word, Rune,*/ Combo, CopySymbol /*, Symbol, Word*/ };


// use txt::CharSet;



#[derive (Debug)]
pub enum Token {
    /*
    BOM32BE,
    BOM32LE,
    BOM16BE,
    BOM16LE,
    BOM8,
    */

    DirectiveTag,  // %TAG
    DirectiveYaml, // %YAML
    Directive, // % // TODO: directives

    DocumentStart, // ---
    DocumentEnd, // ...

    Comment, // #

    Newline, // \n+
    Indent, // \s+
    Tab,   // \t+

    GT, // >
    Dash, // -
    Colon, // :
    Comma, // ,
    Pipe, // |
    Question, // ?

    Anchor, // &
    Alias, // *
    TagHandle, // !

    DictionaryStart, // {
    DictionaryEnd, // }

    SequenceStart, // [
    SequenceEnd, // ]

    StringDouble, // "
    StringSingle, // '

    Raw,

    ReservedCommercialAt, // @
    ReservedGraveAccent // `
}



/*
pub struct Tokenizer<C1, C2>
  where
    C1: CopySymbol,
    C2: CopySymbol
{
    pub cset: CharSet<C1, C2>,
    // pub timer: Timer
}
*/

/*
impl<C1, C2> Tokenizer<C1, C2>
  where
    C1: CopySymbol,
    C2: CopySymbol + Combo
{
    pub fn new (cset: CharSet<C1, C2>) -> Tokenizer<C1, C2> {
        Tokenizer {
            cset: cset,
            // timer: Timer::new ()
        }
    }


    pub fn scan_while_spaces_and_tabs<Reader: Read> (&self, reader: &mut Reader, at: usize) -> usize {
        let mut scanned = at;

        loop {
            if !reader.has (scanned + 1) { break; }

            if reader.contains_copy_at (self.cset.space, scanned) {
                scanned += self.cset.space.len ();
                continue;
            } else if reader.contains_copy_at (self.cset.tab_h, scanned) {
                scanned += self.cset.tab_h.len ();
                continue;
            }

            break;
        }

        scanned - at
    }


    pub fn scan_one_colon_and_line_breakers<Reader: Read> (&self, reader: &mut Reader, at: usize) -> usize {
        if reader.contains_copy_at (self.cset.colon, at) { self.cset.colon.len () }
        else if reader.contains_copy_at (self.cset.crlf, at) { self.cset.crlf.len () }
        else if reader.contains_copy_at (self.cset.line_feed, at) { self.cset.line_feed.len () }
        else if reader.contains_copy_at (self.cset.carriage_return, at) { self.cset.carriage_return.len () }
        else { 0 }
    }


    pub fn scan_until_colon_and_line_breakers<Reader: Read> (&self, reader: &mut Reader, at: usize) -> usize {
        let mut scanned = at;
        loop {
            if !reader.has (scanned + 1) { break; }
            if reader.contains_copy_at (self.cset.colon, scanned) { break; }
            if reader.contains_copy_at (self.cset.crlf, scanned) { break; }
            if reader.contains_copy_at (self.cset.line_feed, scanned) { break; }
            if reader.contains_copy_at (self.cset.carriage_return, scanned) { break; }
            scanned += 1;
        }
        scanned - at
    }


    pub fn scan_until_question_and_line_breakers<Reader: Read> (&self, reader: &mut Reader, at: usize) -> usize {
        let mut scanned = at;
        loop {
            if !reader.has (scanned + 1) { break; }
            if reader.contains_copy_at (self.cset.question, scanned) { break; }
            if reader.contains_copy_at (self.cset.crlf, scanned) { break; }
            if reader.contains_copy_at (self.cset.line_feed, scanned) { break; }
            if reader.contains_copy_at (self.cset.carriage_return, scanned) { break; }
            scanned += 1;
        }
        scanned - at
    }


    pub fn scan_one_spaces_and_line_breakers<Reader: Read> (&self, reader: &mut Reader, at: usize) -> usize {
        if reader.contains_copy_at (self.cset.space, at) { self.cset.space.len () }
        else if reader.contains_copy_at (self.cset.crlf, at) { self.cset.crlf.len () }
        else if reader.contains_copy_at (self.cset.line_feed, at) { self.cset.line_feed.len () }
        else if reader.contains_copy_at (self.cset.tab_h, at) { self.cset.tab_h.len () }
        else if reader.contains_copy_at (self.cset.carriage_return, at) { self.cset.carriage_return.len () }
        else { 0 }
    }


    pub fn scan_while_spaces_and_line_breakers<Reader: Read> (&self, reader: &mut Reader, at: usize) -> usize {
        let mut scanned = at;

        loop {
            if !reader.has (scanned + 1) { break; }

            if reader.contains_copy_at (self.cset.space, scanned) {
                scanned += self.cset.space.len ();
                continue;
            } else if reader.contains_copy_at (self.cset.crlf, scanned) {
                scanned += self.cset.crlf.len ();
                continue;
            } else if reader.contains_copy_at (self.cset.line_feed, scanned) {
                scanned += self.cset.line_feed.len ();
                continue;
            } else if reader.contains_copy_at (self.cset.tab_h, scanned) {
                scanned += self.cset.tab_h.len ();
                continue;
            } else if reader.contains_copy_at (self.cset.carriage_return, scanned) {
                scanned += self.cset.carriage_return.len ();
                continue;
            }

            break;
        }

        scanned - at
    }


    pub fn scan_one_line_breaker<Reader: Read> (&self, reader: &mut Reader, at: usize) -> usize {
        if reader.contains_copy_at (self.cset.crlf, at) { self.cset.crlf.len () }
        else if reader.contains_copy_at (self.cset.line_feed, at) { self.cset.line_feed.len () }
        else if reader.contains_copy_at (self.cset.carriage_return, at) { self.cset.carriage_return.len () }
        else { 0 }
    }


    fn alias_stops<Reader: Read> (&self, reader: &mut Reader) -> usize {
        let mut scanned = 0;

        loop {
            if !reader.has (scanned + 1) { break; }

            if reader.contains_copy_at (self.cset.space, scanned) { break; }
            if reader.contains_copy_at (self.cset.tab_h, scanned) { break; }
            if reader.contains_copy_at (self.cset.crlf, scanned) { break; }
            if reader.contains_copy_at (self.cset.line_feed, scanned) { break; }
            if reader.contains_copy_at (self.cset.carriage_return, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_curly_left, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_curly_right, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_square_left, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_square_right, scanned) { break; }

            if reader.contains_copy_at (self.cset.colon, scanned) {
                if reader.contains_copy_at (self.cset.space, scanned + self.cset.colon.len ()) { break; }
                if reader.contains_copy_at (self.cset.tab_h, scanned + self.cset.colon.len ()) { break; }
                if reader.contains_copy_at (self.cset.line_feed, scanned + self.cset.colon.len ()) { break; }
                if reader.contains_copy_at (self.cset.carriage_return, scanned + self.cset.colon.len ()) { break; }
                if reader.contains_copy_at (self.cset.comma, scanned + self.cset.colon.len ()) { break; }
            }

            if reader.contains_copy_at (self.cset.comma, scanned) {
                if reader.contains_copy_at (self.cset.space, scanned + self.cset.comma.len ()) { break; }
                if reader.contains_copy_at (self.cset.tab_h, scanned + self.cset.comma.len ()) { break; }
                if reader.contains_copy_at (self.cset.line_feed, scanned + self.cset.comma.len ()) { break; }
                if reader.contains_copy_at (self.cset.carriage_return, scanned + self.cset.comma.len ()) { break; }
                if reader.contains_copy_at (self.cset.colon, scanned + self.cset.comma.len ()) { break; }
            }

            scanned += 1;
        }

        scanned
    }


    pub fn anchor_stops<Reader: Read> (&self, reader: &mut Reader, at: usize) -> usize {
        let mut scanned = at;

        loop {
            if !reader.has (scanned + 1) { break; }

            if reader.contains_copy_at (self.cset.space, scanned) { break; }
            if reader.contains_copy_at (self.cset.tab_h, scanned) { break; }
            if reader.contains_copy_at (self.cset.crlf, scanned) { break; }
            if reader.contains_copy_at (self.cset.line_feed, scanned) { break; }
            if reader.contains_copy_at (self.cset.carriage_return, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_curly_left, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_curly_right, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_square_left, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_square_right, scanned) { break; }

            scanned += 1;
        }

        scanned - at
    }


    pub fn raw_stops<Reader: Read> (&self, reader: &mut Reader) -> usize {
        let mut scanned = 0;

        loop {
            if !reader.has (scanned + 1) { break; }

            if reader.contains_copy_at (self.cset.crlf, scanned) { break; }
            if reader.contains_copy_at (self.cset.line_feed, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_curly_left, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_curly_right, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_square_left, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_square_right, scanned) { break; }
            if reader.contains_copy_at (self.cset.hashtag, scanned) { break; }
            if reader.contains_copy_at (self.cset.colon, scanned) { break; }
            if reader.contains_copy_at (self.cset.comma, scanned) { break; }
            if reader.contains_copy_at (self.cset.carriage_return, scanned) { break; }

            scanned += 1;
        }

        scanned
    }


    fn tag_flow_stops<Reader: Read> (&self, reader: &mut Reader, at: usize) -> usize {
        let mut scanned = at;

        loop {
            if !reader.has (scanned + 1) { break; }

            if reader.contains_copy_at (self.cset.space, scanned) { break; }
            if reader.contains_copy_at (self.cset.tab_h, scanned) { break; }
            if reader.contains_copy_at (self.cset.crlf, scanned) { break; }
            if reader.contains_copy_at (self.cset.line_feed, scanned) { break; }
            if reader.contains_copy_at (self.cset.carriage_return, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_curly_left, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_curly_right, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_square_left, scanned) { break; }
            if reader.contains_copy_at (self.cset.bracket_square_right, scanned) { break; }
            if reader.contains_copy_at (self.cset.colon, scanned) { break; }
            if reader.contains_copy_at (self.cset.comma, scanned) { break; }

            scanned += 1;
        }

        scanned - at
    }


    fn directive_tag<Reader: Read> (&self, reader: &mut Reader, at: usize) -> usize {
        if reader.contains_copy_at (self.cset.letter_t_t, at) &&
           reader.contains_copy_at (self.cset.letter_t_a, at + self.cset.letter_t_t.len ()) &&
           reader.contains_copy_at (self.cset.letter_t_g, at + self.cset.letter_t_t.len () + self.cset.letter_t_a.len ())
        {
            return self.cset.letter_t_t.len () + self.cset.letter_t_a.len () + self.cset.letter_t_g.len ()
        } else { return 0 }
    }


    fn directive_yaml<Reader: Read> (&self, reader: &mut Reader, at: usize) -> usize {
        if reader.contains_copy_at (self.cset.letter_t_y, at) &&
           reader.contains_copy_at (self.cset.letter_t_a, at + self.cset.letter_t_y.len ()) &&
           reader.contains_copy_at (self.cset.letter_t_m, at + self.cset.letter_t_y.len () + self.cset.letter_t_a.len ()) &&
           reader.contains_copy_at (self.cset.letter_t_l, at + self.cset.letter_t_y.len () + self.cset.letter_t_a.len () + self.cset.letter_t_m.len ())
        {
            return self.cset.letter_t_y.len () + self.cset.letter_t_a.len () + self.cset.letter_t_m.len () + self.cset.letter_t_l.len ()
        } else { return 0 }
    }


    pub fn line<Reader: Read> (&self, reader: &mut Reader) -> usize {
        let mut scanned = 0;
        loop {
            if !reader.has (scanned + 1) { break; }
            if reader.contains_copy_at (self.cset.crlf, scanned) { break; }
            if reader.contains_copy_at (self.cset.line_feed, scanned) { break; }
            if reader.contains_copy_at (self.cset.carriage_return, scanned) { break; }
            scanned += 1;
        }
        scanned
    }


    pub fn line_at<Reader: Read> (&self, reader: &mut Reader, at: usize) -> usize {
        let mut scanned = at;
        loop {
            if !reader.has (scanned + 1) { break; }
            if reader.contains_copy_at (self.cset.crlf, scanned) { break; }
            if reader.contains_copy_at (self.cset.line_feed, scanned) { break; }
            if reader.contains_copy_at (self.cset.carriage_return, scanned) { break; }
            scanned += 1;
        }
        scanned - at
    }


    pub fn get_token<Reader: Read> (&self, reader: &mut Reader) -> Option<(Token, usize, usize)> {
        let mut scanned: usize = 0;
        let mut chars: usize = 0;

        loop {
            /* --- WORDS --- */
            loop {
                if reader.contains_copy_at (self.cset.space, scanned) {
                    scanned += self.cset.space.len ();
                    chars += 1;
                    continue;
                }
                break;
            }
            if scanned > 0 { return Some ( (Token::Indent, scanned, chars) ) }

            let quoted = scan_quoted (reader, self.cset.quotation, self.cset.backslash);
            if quoted > 0 { return Some ( (Token::StringDouble, quoted, 1) ) }

            let quoted = scan_quoted_selfescape (reader, self.cset.apostrophe, self.cset.backslash);
            if quoted > 0 { return Some ( (Token::StringSingle, quoted, 1) ) }

            if reader.contains_copy_at_start (self.cset.comma) { return Some ( (Token::Comma, self.cset.comma.len (), 1) ) }
            if reader.contains_copy_at_start (self.cset.colon) { return Some ( (Token::Colon, self.cset.colon.len (), 1) ) }

            if reader.contains_copy_at_start (self.cset.bracket_curly_left) { return Some ((Token::DictionaryStart, self.cset.bracket_curly_left.len (), 1) ) }
            if reader.contains_copy_at_start (self.cset.bracket_curly_right) { return Some ((Token::DictionaryEnd, self.cset.bracket_curly_right.len (), 1) ) }

            if reader.contains_copy_at_start (self.cset.bracket_square_left) { return Some ( (Token::SequenceStart, self.cset.bracket_square_left.len (), 1) ) }
            if reader.contains_copy_at_start (self.cset.bracket_square_right) { return Some ( (Token::SequenceEnd, self.cset.bracket_square_right.len (), 1) ) }


            loop {
                if reader.contains_copy_at (self.cset.crlf, scanned) {
                    scanned += self.cset.crlf.len ();
                    chars += 1;
                    continue;
                }

                if reader.contains_copy_at (self.cset.line_feed, scanned) {
                    scanned += self.cset.line_feed.len ();
                    chars += 1;
                    continue;
                }

                if reader.contains_copy_at (self.cset.carriage_return, scanned) {
                    scanned += self.cset.carriage_return.len ();
                    chars += 1;
                    continue;
                }
                break;
            }
            if scanned > 0 { return Some ( (Token::Newline, scanned, chars) ) }


            if reader.contains_copy_at_start (self.cset.hashtag) {
                let comment = self.line (reader);
                if comment > 0 { return Some ( (Token::Comment, comment, 1) ) }
                break;
            }


            if reader.contains_copy_at_start (self.cset.asterisk) {
                let alias = self.alias_stops (reader);
                if alias > 0 { return Some ( (Token::Alias, alias, 1) ) }
                break;
            }


            if reader.contains_copy_at_start (self.cset.ampersand) {
                let anchor = self.anchor_stops (reader, self.cset.ampersand.len ());
                if anchor > 0 { return Some ( (Token::Anchor, anchor + self.cset.ampersand.len (), 1) ) }
                break;
            }


            if reader.contains_copy_at_start (self.cset.greater_than) { return Some ( (Token::GT, self.cset.greater_than.len (), 1) ) }

            if reader.contains_copy_at_start (self.cset.vertical_bar) { return Some ( (Token::Pipe, self.cset.vertical_bar.len (), 1) ) }

            if reader.contains_copy_at_start (self.cset.exclamation) {
                let len = if reader.contains_copy_at (self.cset.less_than, self.cset.exclamation.len ()) {
                    loop {
                        if !reader.has (scanned + 1) { break; }
                        if reader.contains_copy_at (self.cset.greater_than, scanned + self.cset.exclamation.len () + self.cset.less_than.len ()) { break; }
                        scanned += 1;
                    }

                    self.cset.exclamation.len () + self.cset.less_than.len () + self.cset.exclamation.len () + scanned

                } else {
                    let len = self.tag_flow_stops (reader, self.cset.exclamation.len ());
                    self.cset.exclamation.len () + len
                };

                return Some ( (Token::TagHandle, len, 0) )
            }

            if reader.contains_copy_at_start (self.cset.full_stop) {
                if reader.contains_copy_at (self.cset.full_stop, self.cset.full_stop.len ()) &&
                    reader.contains_copy_at (self.cset.full_stop, self.cset.full_stop.len () * 2) {
                        return Some ( (Token::DocumentEnd, self.cset.full_stop.len () * 3, 0) )
                    }
            }

            if reader.contains_copy_at_start (self.cset.percent) {
                let len = self.directive_tag (reader, self.cset.percent.len ());
                if len > 0 {
                    return Some ( (Token::DirectiveTag, self.cset.percent.len () + len, 0) )
                } else {
                    let len = self.directive_yaml (reader, self.cset.percent.len ());
                    if len > 0 {
                        return Some ( (Token::DirectiveYaml, self.cset.percent.len () + len, 0) )
                    }
                }

                return Some ( (Token::Directive, self.line (reader), 1) )
            }

            if reader.contains_copy_at_start (self.cset.hyphen_minus) {
                if !reader.has (self.cset.hyphen_minus.len () + 1) {
                    return Some ( (Token::Dash, self.cset.hyphen_minus.len (), 1) )
                } else {
                    if reader.contains_copy_at (self.cset.hyphen_minus, self.cset.hyphen_minus.len ()) {
                        if reader.contains_copy_at (self.cset.hyphen_minus, self.cset.hyphen_minus.len () * 2) {
                            if self.scan_one_spaces_and_line_breakers (reader, self.cset.hyphen_minus.len () * 3) > 0 {
                                return Some ( (Token::DocumentStart, self.cset.hyphen_minus.len () * 3, 0) )
                            }
                        }
                    }

                    if self.scan_one_spaces_and_line_breakers (reader, self.cset.hyphen_minus.len ()) > 0 {
                        return Some ( (Token::Dash, self.cset.hyphen_minus.len (), 1) )
                    }
                }
            }

            if reader.contains_copy_at_start (self.cset.question) {
                if !reader.has (self.cset.question.len () + 1) {
                    return Some ( (Token::Question, self.cset.question.len (), 1) )
                } else {
                    if self.scan_one_spaces_and_line_breakers (reader, self.cset.question.len ()) > 0 {
                        return Some ( (Token::Question, self.cset.question.len (), 1) )
                    }
                }
            }

            if reader.contains_copy_at_start (self.cset.commercial_at) {
                return Some ( (Token::ReservedCommercialAt, self.cset.commercial_at.len (), 1) )
            }

            if reader.contains_copy_at_start (self.cset.grave_accent) {
                return Some ( (Token::ReservedGraveAccent, self.cset.grave_accent.len (), 1) )
            }


            /* --- BOM --- */

            if reader.has (4) {
                let slice = reader.slice (4).unwrap ();

                if slice == &[ 0x00, 0x00, 0xFE, 0xFF ] {
                    return Some ( (Token::BOM32BE, 4, 1) )
                } else if &slice[0 .. 3] == &[ 0x00, 0x00, 0x00 ] {
                    return Some ( (Token::BOM32BE, 4, 1) )
                } else if slice == &[ 0xFF, 0xFE, 0x00, 0x00 ] {
                    return Some ( (Token::BOM32LE, 4, 1) )
                } else if &slice[1 .. 3] == &[ 0x00, 0x00, 0x00 ] {
                    return Some ( (Token::BOM32LE, 4, 1) )
                } else if slice[0] == 0x00 {
                    return Some ( (Token::BOM16BE, 2, 1) )
                } else if slice[1] == 0x00 {
                    return Some ( (Token::BOM16LE, 2, 1) )
                } else if &slice[0 .. 2] == &[ 0xFE, 0xFF ] {
                    return Some ( (Token::BOM16BE, 2, 1) )
                } else if &slice[0 .. 2] == &[ 0xFF, 0xFE ] {
                    return Some ( (Token::BOM16LE, 2, 1) )
                } else if &slice[0 .. 3] == &[ 0xEF, 0xBB, 0xBF ] {
                    return Some ( (Token::BOM8, 3, 1) )
                }
            } else if reader.has (3) {
                let slice = reader.slice (3).unwrap ();

                if slice == &[ 0xEF, 0xBB, 0xBF ] {
                    return Some ( (Token::BOM8, 3, 1) )
                } else if slice[0] == 0x00 {
                    return Some ( (Token::BOM16BE, 2, 1) )
                } else if slice[1] == 0x00 {
                    return Some ( (Token::BOM16LE, 2, 1) )
                } else if &slice[0 .. 2] == &[ 0xFE, 0xFF ] {
                    return Some ( (Token::BOM16BE, 2, 1) )
                } else if &slice[0 .. 2] == &[ 0xFF, 0xFE ] {
                    return Some ( (Token::BOM16LE, 2, 1) )
                }
            } else if reader.has (2) {
                let slice = reader.slice (2).unwrap ();

                if slice[0] == 0x00 {
                    return Some ( (Token::BOM16BE, 2, 1) )
                } else if slice[1] == 0x00 {
                    return Some ( (Token::BOM16LE, 2, 1) )
                } else if slice == &[ 0xFE, 0xFF ] {
                    return Some ( (Token::BOM16BE, 2, 1) )
                } else if slice == &[ 0xFF, 0xFE ] {
                    return Some ( (Token::BOM16LE, 2, 1) )
                }
            }


            loop {
                if reader.contains_copy_at (self.cset.tab_h, scanned) {
                    scanned += self.cset.tab_h.len ();
                    chars += 1;
                    continue;
                }
                break;
            }
            if scanned > 0 { return Some ( (Token::Tab, scanned, chars) ) }


            /* --- RAW --- */

            loop {
                if !reader.has (scanned + 1) { break; }

                if reader.contains_copy_at (self.cset.crlf, scanned) { break; }
                if reader.contains_copy_at (self.cset.line_feed, scanned) { break; }
                if reader.contains_copy_at (self.cset.bracket_curly_left, scanned) { break; }
                if reader.contains_copy_at (self.cset.bracket_curly_right, scanned) { break; }
                if reader.contains_copy_at (self.cset.bracket_square_left, scanned) { break; }
                if reader.contains_copy_at (self.cset.bracket_square_right, scanned) { break; }
                if reader.contains_copy_at (self.cset.hashtag, scanned) { break; }
                if reader.contains_copy_at (self.cset.colon, scanned) { break; }
                if reader.contains_copy_at (self.cset.comma, scanned) { break; }
                if reader.contains_copy_at (self.cset.carriage_return, scanned) { break; }

                scanned += 1;
            }
            if scanned > 0 { return Some ( (Token::Raw, scanned, 1) ) }

            break;
        }

        None
    }
}
*/


pub fn scan_double_quoted<R: Read> (reader: &mut R) -> usize {
    let mut scanned = if let Some (b'"') = reader.get_byte_at_start () {
        1
    } else { return 0 };

    loop {
        match reader.get_byte_at (scanned) {
            None => break,
            Some (b'"') => { scanned += 1; break },
            Some (b'\\') => match reader.get_byte_at (scanned + 1) {
                Some (b'"') |
                Some (b'\\') => { scanned += 2; },
                None => { scanned += 1; break; }
                _ => { scanned += 1; }
            },
            _ => scanned += 1
        };
    }

    scanned
}


pub fn scan_single_quoted<R: Read> (reader: &mut R) -> usize {
    let mut scanned = if let Some (b'\'') = reader.get_byte_at_start () {
        1
    } else { return 0 };

    loop {
        match reader.get_byte_at (scanned) {
            None => break,
            Some (b'\'') => match reader.get_byte_at (scanned + 1) {
                Some (b'\'') => { scanned += 2 }
                _ => { scanned += 1; break }
            },
            _ => scanned += 1
        }
    }

    scanned
}



pub fn scan_while_spaces_and_tabs<Reader: Read> (reader: &mut Reader, at: usize) -> usize {
    let mut scanned = at;

    loop {
        match reader.get_byte_at (scanned) {
            Some (b' ') |
            Some (b'\t') => { scanned += 1; continue }
            _ => break
        }
    }

    scanned - at
}



pub fn scan_one_colon_and_line_breakers<Reader: Read> (reader: &mut Reader, at: usize) -> usize {
    match reader.get_byte_at (at) {
        Some (b':') => 1,
        Some (b'\n') => 1,
        Some (b'\r') => {
            if let Some (b'\n') = reader.get_byte_at (at + 1) { 2 } else { 1 }
        }
        _ => 0
    }
}



pub fn scan_until_colon_and_line_breakers<Reader: Read> (reader: &mut Reader, at: usize) -> usize {
    let mut scanned = at;
    loop {
        match reader.get_byte_at (scanned) {
            None |
            Some (b':') |
            Some (b'\n') |
            Some (b'\r') => break,
            _ => scanned += 1
        };
    }
    scanned - at
}



pub fn scan_until_question_and_line_breakers<Reader: Read> (reader: &mut Reader, at: usize) -> usize {
    let mut scanned = at;
    loop {
        match reader.get_byte_at (scanned) {
            None |
            Some (b'?') |
            Some (b'\n') |
            Some (b'\r') => break,
            _ => scanned += 1
        };
    }
    scanned - at
}




pub fn scan_one_spaces_and_line_breakers<Reader: Read> (reader: &mut Reader, at: usize) -> usize {
    match reader.get_byte_at (at) {
        Some (b' ') => 1,
        Some (b'\n') => 1,
        Some (b'\t') => 1,
        Some (b'\r') => if let Some (b'\n') = reader.get_byte_at (at + 1) { 2 } else { 1 },
        _ => 0
    }
}



pub fn scan_while_spaces_and_line_breakers<Reader: Read> (reader: &mut Reader, at: usize) -> usize {
    let mut scanned = at;

    loop {
        match reader.get_byte_at (at) {
            Some (b' ') => scanned += 1,
            Some (b'\n') => scanned += 1,
            Some (b'\t') => scanned += 1,
            Some (b'\r') => scanned += if let Some (b'\n') = reader.get_byte_at (scanned + 1) { 2 } else { 1 },
            _ => break
        };
    }

    scanned - at
}



pub fn scan_one_line_breaker<Reader: Read> (reader: &mut Reader, at: usize) -> usize {
    match reader.get_byte_at (at) {
        Some (b'\n') => 1,
        Some (b'\r') => if let Some (b'\n') = reader.get_byte_at (at + 1) { 2 } else { 1 },
        _ => 0
    }
}




fn alias_stops<Reader: Read> (reader: &mut Reader) -> usize {
    let mut scanned = 0;

    loop {
        match reader.get_byte_at (scanned) {
            None => break,

            Some (b' ') |
            Some (b'\t') |
            Some (b'\n') |
            Some (b'\r') |
            Some (b'{') |
            Some (b'}') |
            Some (b'[') |
            Some (b']') => break,

            Some (b':') => match reader.get_byte_at (scanned + 1) {
                Some (b' ') |
                Some (b'\t') |
                Some (b'\n') |
                Some (b'\r') |
                Some (b',') => break,
                _ => ()
            },

            Some (b',') => match reader.get_byte_at (scanned + 1) {
                Some (b' ') |
                Some (b'\t') |
                Some (b'\n') |
                Some (b'\r') |
                Some (b':') => break,
                _ => ()
            },

            _ => ()
        };

        scanned += 1
    }

    scanned
}



pub fn anchor_stops<Reader: Read> (reader: &mut Reader, at: usize) -> usize {
    let mut scanned = at;

    loop {
        match reader.get_byte_at (scanned) {
            None |
            Some (b' ') |
            Some (b'\t') |
            Some (b'\n') |
            Some (b'\r') |
            Some (b'{') |
            Some (b'}') |
            Some (b'[') |
            Some (b']') => break,
            _ => scanned += 1
        };
    }

    scanned - at
}



pub fn raw_stops<Reader: Read> (reader: &mut Reader) -> usize {
    let mut scanned = 0;

    loop {
        match reader.get_byte_at (scanned) {
            None |
            Some (b'\n') |
            Some (b'\r') |
            Some (b'{') |
            Some (b'}') |
            Some (b'[') |
            Some (b']') |
            Some (b'#') |
            Some (b':') |
            Some (b',') => break,
            _ => scanned += 1
        };
    }

    scanned
}



fn tag_flow_stops<Reader: Read> (reader: &mut Reader, at: usize) -> usize {
    let mut scanned = at;

    loop {
        match reader.get_byte_at (scanned) {
            None |
            Some (b' ') |
            Some (b'\t') |
            Some (b'\n') |
            Some (b'\r') |
            Some (b'{') |
            Some (b'}') |
            Some (b'[') |
            Some (b']') |
            Some (b':') |
            Some (b',') => break,
            _ => scanned += 1
        };
    }

    scanned - at
}



pub fn line<Reader: Read> (reader: &mut Reader) -> usize {
    let mut scanned = 0;
    loop {
        match reader.get_byte_at (scanned) {
            None |
            Some (b'\n') |
            Some (b'\r') => break,
            _ => scanned += 1
        };
    }
    scanned
}



pub fn line_at<Reader: Read> (reader: &mut Reader, at: usize) -> usize {
    let mut scanned = at;
    loop {
        match reader.get_byte_at (scanned) {
            None |
            Some (b'\n') |
           Some (b'\r') => break,
            _ => scanned += 1
        };
    }
    scanned - at
}




pub fn get_token<Reader: Read> (reader: &mut Reader) -> Option<(Token, usize, usize)> {
    match reader.get_byte_at_start () {
        Some (b',') => return Some ((Token::Comma, 1, 1)),
        Some (b':') => return Some ((Token::Colon, 1, 1)),
        Some (b'{') => return Some ((Token::DictionaryStart, 1, 1)),
        Some (b'}') => return Some ((Token::DictionaryEnd, 1, 1)),
        Some (b'[') => return Some ((Token::SequenceStart, 1, 1)),
        Some (b']') => return Some ((Token::SequenceEnd, 1, 1)),
        Some (b'>') => return Some ((Token::GT, 1, 1)),
        Some (b'|') => return Some ((Token::Pipe, 1, 1)),
        Some (b'"') => return Some ((Token::StringDouble, scan_double_quoted (reader), 1)),
        Some (b'\'') => return Some ((Token::StringSingle, scan_single_quoted (reader), 1)),
        Some (b'#') => return Some ((Token::Comment, line_at (reader, 1) + 1, 1)),
        Some (b'*') => return Some ((Token::Alias, alias_stops (reader), 1)),
        Some (b'&') => return Some ((Token::Anchor, anchor_stops (reader, 1) + 1, 1)),
        Some (b'.') => {
            if let Some ((b'.', b'.')) = reader.get_bytes_2_at (1) { return Some ((Token::DocumentEnd, 3, 0)) };
        },

        Some (b' ') => {
            let mut scanned = 1;
            loop {
                match reader.get_byte_at (scanned) {
                    Some (b' ') => { scanned += 1; }
                    _ => break
                }
            }
            return Some ((Token::Indent, scanned, scanned))
        }

        Some (b'\n') |
        Some (b'\r') => {
            let mut scanned = 1;

            loop {
                match reader.get_byte_at (scanned) {
                    Some (b'\n') |
                    Some (b'\r') => { scanned += 1; }
                    _ => break
                }
            }

            return Some ( (Token::Newline, scanned, scanned) )
        }

        Some (b'!') => {
            let mut scanned = 0;
            if let Some (b'<') = reader.get_byte_at (1) {
                scanned += 2;
                loop {
                    match reader.get_byte_at (scanned) {
                        Some (b'>') => { scanned += 1; break },
                        _ => scanned += 1
                    };
                }
            } else {
                scanned = tag_flow_stops (reader, 1) + 1;
            };

            return Some ((Token::TagHandle, scanned, 0))
        }

        Some (b'%') => match reader.get_bytes_4_at (1) {
            None => match reader.get_bytes_3_at (1) {
                Some ((b'T', b'A', b'G')) => return Some ((Token::DirectiveTag, 4, 0)),
                _ => return Some ((Token::DirectiveYaml, line_at (reader, 1) + 1, 0))
            },
            Some ((b'Y', b'A', b'M', b'L')) => return Some ((Token::DirectiveYaml, 5, 0)),
            Some ((b'T', b'A', b'G', _)) => return Some ((Token::DirectiveTag, 4, 0)),
            _ => return Some ((Token::DirectiveYaml, line_at (reader, 1) + 1, 0))
        },

        Some (b'-') => match reader.get_byte_at (1) {
            None => return Some ((Token::Dash, 1, 1)),
            Some (b'-') => { if let Some (b'-') = reader.get_byte_at (2) { return Some ((Token::DocumentStart, 3, 0)) }; }
            _ => { if scan_one_spaces_and_line_breakers (reader, 1) > 0 { return Some ((Token::Dash, 1, 1)) }; }
        },

        Some (b'?') => match reader.get_byte_at (1) {
            None => return Some ((Token::Question, 1, 1)),
            _ => { if scan_one_spaces_and_line_breakers (reader, 1) > 0 { return Some ((Token::Question, 1, 1)) }; }
        },

        Some (b'@') => return Some ((Token::ReservedCommercialAt, 1, 1)),
        Some (b'`') => return Some ((Token::ReservedGraveAccent, 1, 1)),

        Some (b'\t') => {
            let mut scanned = 1;
            loop {
                match reader.get_byte_at (scanned) {
                    Some (b't') => { scanned += 1; }
                    _ => break
                }
            }
            return Some ((Token::Tab, scanned, scanned))
        }

        _ => ()
    };

    let scanned = raw_stops (reader);
    if scanned > 0 { Some ((Token::Raw, scanned, 1)) } else { None }
}



#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use super::skimmer::reader::Read;
    use super::skimmer::reader::SliceReader;

    // use txt::get_charset_utf8;


    #[test]
    fn test_tokenizer_general () {
        let src = "%YAML 1.2\n%TAG ! tag://example.com,2015:yamlette/\n---\n\"double string\"\n    \r'single string'\n\r[\"list\", 'of', tokens]\r\n{key: val, key: val} ...";

        // %TAG ! tag://example.com,2015:yamlette/\n

        let mut reader = SliceReader::new (src.as_bytes ());
        // let tokenizer = Tokenizer; // ::new (get_charset_utf8 ());

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (5, len);
            assert_eq! (0, clen);

            if let Token::DirectiveYaml = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (3, len);
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (4, len);
            assert_eq! (0, clen);

            if let Token::DirectiveTag = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (0, clen);

            if let Token::TagHandle = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "tag".as_bytes ().len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, ":".as_bytes ().len ());
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "//example.com".as_bytes ().len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, ",".as_bytes ().len ());
            assert_eq! (1, clen);

            if let Token::Comma = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "2015".as_bytes ().len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, ":".as_bytes ().len ());
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "yamlette/".as_bytes ().len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (3, len);
            assert_eq! (0, clen);

            if let Token::DocumentStart = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "\"double string\"".len ());
            assert_eq! (1, clen);

            if let Token::StringDouble = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "\n".len ());
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "    ".len ());
            assert_eq! (4, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "\r".len ());
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "'single string'".len ());
            assert_eq! (1, clen);

            if let Token::StringSingle = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "\n\r".len ());
            assert_eq! (2, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (clen));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "[".len ());
            assert_eq! (1, clen);

            if let Token::SequenceStart = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "\"list\"".len ());
            assert_eq! (1, clen);

            if let Token::StringDouble = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Comma = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "'of'".len ());
            assert_eq! (1, clen);

            if let Token::StringSingle = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Comma = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "tokens".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "]".len ());
            assert_eq! (1, clen);

            if let Token::SequenceEnd = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "\r\n".len ());
            assert_eq! (2, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "{".len ());
            assert_eq! (1, clen);

            if let Token::DictionaryStart = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "key".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "key".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Comma = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "key".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "key".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "{".len ());
            assert_eq! (1, clen);

            if let Token::DictionaryEnd = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (3, len);
            assert_eq! (0, clen);

            if let Token::DocumentEnd = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some (_) = get_token (&mut reader) { assert! (false, "Unexpected token") }
    }



    #[test]
    fn test_tokenizer_anchor () {
        let src = "- &anchor string";

        let mut reader = SliceReader::new (src.as_bytes ());
        // let tokenizer = Tokenizer; // ::new (get_charset_utf8 ());

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (7, len);
            assert_eq! (1, clen);

            if let Token::Anchor = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (6, len);
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }
    }



    #[test]
    fn test_tokenizer_alias () {
        let src = "- *anchor string";

        let mut reader = SliceReader::new (src.as_bytes ());
        // let tokenizer = Tokenizer; // ::new (get_charset_utf8 ());

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (7, len);
            assert_eq! (1, clen); 

            if let Token::Alias = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (6, len);
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        } else { assert! (false, "Unexpected result!") }
    }



    #[test]
    fn test_tokenizer_e_2_1 () {
        let src =
r"- Mark McGwire
- Sammy Sosa
- Ken Griffey";


        let mut reader = SliceReader::new (src.as_bytes ());
        // let tokenizer = Tokenizer; // ::new (get_charset_utf8 ());

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "Mark McGwire".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (clen, reader.skip_long (clen));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "Sammy Sosa".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "Ken Griffey".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }
    }



    #[test]
    fn test_tokenizer_e_2_2 () {
        let src =
r"hr:  65    # Home runs
avg: 0.278 # Batting average
rbi: 147   # Runs Batted In";

        let mut reader = SliceReader::new (src.as_bytes ());
        // let tokenizer = Tokenizer; // ::new (get_charset_utf8 ());

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "hr".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "65    ".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "# Home runs".len ());
            assert_eq! (1, clen);

            if let Token::Comment = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "avg".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "0.278 ".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "# Batting average".len ());
            assert_eq! (1, clen);

            if let Token::Comment = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "rbi".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "147   ".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "# Runs Batted In".len ());
            assert_eq! (1, clen);

            if let Token::Comment = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }
    }



    #[test]
    fn test_tokenizer_e_2_3 () {
        let src =
r"american:
  - Boston Red Sox
  - Detroit Tigers
  - New York Yankees
national:
  - New York Mets
  - Chicago Cubs
  - Atlanta Braves";

        let mut reader = SliceReader::new (src.as_bytes ());
        // let tokenizer = Tokenizer; // ::new (get_charset_utf8 ());

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "american".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "Boston Red Sox".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "Detroit Tigers".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "New York Yankees".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "national".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "New York Mets".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "Chicago Cubs".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "Atlanta Braves".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }
    }



    #[test]
    fn test_tokenizer_e_2_4 () {
        let src =
r"-
  name: Mark McGwire
  hr:   65
  avg:  0.278
-
  name: Sammy Sosa
  hr:   63
  avg:  0.288";

        let mut reader = SliceReader::new (src.as_bytes ());
        // let tokenizer = Tokenizer; // ::new (get_charset_utf8 ());

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (len, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "name".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "Mark McGwire".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "hr".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (3, len);
            assert_eq! (3, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "65".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "avg".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "0.278".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "name".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "Sammy Sosa".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "hr".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (3, len);
            assert_eq! (3, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "63".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }


        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "avg".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }

        if let Some ( (token, len, clen) ) = get_token (&mut reader) {
            assert_eq! (len, "0.288".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip_long (len));
        }
    }
}
