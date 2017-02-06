extern crate skimmer;


use self::skimmer::reader::Read;

use self::skimmer::scanner::{ Quote, scan, scan_one_at, scan_while, scan_until, scan_until_at };

use self::skimmer::symbol::{ Char, Word, Rune, Symbol };


use txt::CharSet;



#[derive (Debug)]
pub enum Token {
    BOM32BE,
    BOM32LE,
    BOM16BE,
    BOM16LE,
    BOM8,

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




pub struct Tokenizer {
    pub cset: CharSet,

    pub alias_stops: [Rune; 19],
    pub anchor_stops: [Rune; 9],
    pub tag_flow_stops: [Rune; 11],
    pub tag_stops: [Char; 1],
    pub line_breakers: [Rune; 3],
    pub raw_stops: [Rune; 10],
    pub spaces: [Char; 1],
    pub tabs: [Char; 1],

    pub triple_hyphen_minus: Word, // ---
    pub triple_full_stop: Word, // ...
    pub directive_tag: Word, // %TAG
    pub directive_yaml: Word, // %YAML
    pub directive_yaml_version: Word, // 1.2

    pub spaces_and_tabs: [Char; 2],
    pub spaces_and_line_breakers: [Rune; 5],
    pub colon_and_line_breakers: [Rune; 4],
    pub question_and_line_breakers: [Rune; 4],

    scfg_escape: [Char; 1],
    scfg_str_dbl_quotes: [(Quote, Char); 1],
    scfg_str_sgl_quotes: [(Quote, Char); 1],
}



impl Tokenizer {
    pub fn new (cset: CharSet) -> Tokenizer {
        Tokenizer {
            line_breakers: [
                Rune::from (cset.crlf.clone ()),
                Rune::from (cset.line_feed.clone ()),
                Rune::from (cset.carriage_return.clone ())
            ],

            colon_and_line_breakers: [
                Rune::from (cset.colon.clone ()),
                Rune::from (cset.crlf.clone ()),
                Rune::from (cset.line_feed.clone ()),
                Rune::from (cset.carriage_return.clone ())
            ],

            question_and_line_breakers: [
                Rune::from (cset.question.clone ()),
                Rune::from (cset.crlf.clone ()),
                Rune::from (cset.line_feed.clone ()),
                Rune::from (cset.carriage_return.clone ())
            ],

            alias_stops: [
                Rune::from (cset.space.clone ()),
                Rune::from (cset.tab_h.clone ()),
                Rune::from (cset.crlf.clone ()),
                Rune::from (cset.line_feed.clone ()),
                Rune::from (cset.carriage_return.clone ()),
                Rune::from (cset.bracket_curly_left.clone ()),
                Rune::from (cset.bracket_curly_right.clone ()),
                Rune::from (cset.bracket_square_left.clone ()),
                Rune::from (cset.bracket_square_right.clone ()),

                Rune::from (Word::combine (&[&cset.colon, &cset.space])),
                Rune::from (Word::combine (&[&cset.colon, &cset.tab_h])),
                Rune::from (Word::combine (&[&cset.colon, &cset.line_feed])),
                Rune::from (Word::combine (&[&cset.colon, &cset.carriage_return])),
                Rune::from (Word::combine (&[&cset.colon, &cset.comma])),

                Rune::from (Word::combine (&[&cset.comma, &cset.space])),
                Rune::from (Word::combine (&[&cset.comma, &cset.tab_h])),
                Rune::from (Word::combine (&[&cset.comma, &cset.line_feed])),
                Rune::from (Word::combine (&[&cset.comma, &cset.carriage_return])),
                Rune::from (Word::combine (&[&cset.comma, &cset.colon]))
            ],

            anchor_stops: [
                Rune::from (cset.space.clone ()),
                Rune::from (cset.tab_h.clone ()),
                Rune::from (cset.crlf.clone ()),
                Rune::from (cset.line_feed.clone ()),
                Rune::from (cset.carriage_return.clone ()),
                Rune::from (cset.bracket_curly_left.clone ()),
                Rune::from (cset.bracket_curly_right.clone ()),
                Rune::from (cset.bracket_square_left.clone ()),
                Rune::from (cset.bracket_square_right.clone ())
            ],

            tag_flow_stops: [
                Rune::from (cset.space.clone ()),
                Rune::from (cset.tab_h.clone ()),
                Rune::from (cset.crlf.clone ()),
                Rune::from (cset.line_feed.clone ()),
                Rune::from (cset.carriage_return.clone ()),
                Rune::from (cset.bracket_curly_left.clone ()),
                Rune::from (cset.bracket_curly_right.clone ()),
                Rune::from (cset.bracket_square_left.clone ()),
                Rune::from (cset.bracket_square_right.clone ()),
                Rune::from (cset.colon.clone ()),
                Rune::from (cset.comma.clone ())
            ],

            tag_stops: [cset.greater_than.clone ()],

            spaces: [cset.space.clone ()],
            tabs: [cset.tab_h.clone ()],
            spaces_and_tabs: [cset.space.clone (), cset.tab_h.clone ()],

            triple_hyphen_minus: Word::combine (&[&cset.hyphen_minus, &cset.hyphen_minus, &cset.hyphen_minus]),
            triple_full_stop: Word::combine (&[&cset.full_stop, &cset.full_stop, &cset.full_stop]),
            directive_tag: Word::combine (&[&cset.percent, &cset.letter_t_t, &cset.letter_t_a, &cset.letter_t_g]),
            directive_yaml: Word::combine (&[&cset.percent, &cset.letter_t_y, &cset.letter_t_a, &cset.letter_t_m, &cset.letter_t_l]),
            directive_yaml_version: Word::combine (&[&cset.digit_1, &cset.full_stop, &cset.digit_2]),

            spaces_and_line_breakers: [
                Rune::from (cset.space.clone ()),
                Rune::from (cset.tab_h.clone ()),
                Rune::from (cset.crlf.clone ()),
                Rune::from (cset.line_feed.clone ()),
                Rune::from (cset.carriage_return.clone ())
            ],


            raw_stops: [
                Rune::from (cset.bracket_curly_left.clone ()),
                Rune::from (cset.bracket_curly_right.clone ()),
                Rune::from (cset.bracket_square_left.clone ()),
                Rune::from (cset.bracket_square_right.clone ()),
                Rune::from (cset.hashtag.clone ()),
                Rune::from (cset.crlf.clone ()),
                Rune::from (cset.line_feed.clone ()),
                Rune::from (cset.carriage_return.clone ()),

                Rune::from (cset.colon.clone ()),
                Rune::from (cset.comma.clone ())
            ],

            scfg_escape: [cset.backslash.clone ()],
            scfg_str_dbl_quotes: [(Quote::new (), cset.quotation.clone ())],
            scfg_str_sgl_quotes: [(Quote::new (), cset.apostrophe.clone ())],

            cset: cset
        }
    }


    pub fn line<Reader: Read> (&self, reader: &mut Reader) -> usize {
        let (size, _) = scan_until (reader, &self.line_breakers);
        size
    }


    pub fn get_token<Reader: Read> (&self, reader: &mut Reader) -> Option<(Token, usize, usize)> {
        use self::skimmer::scanner::NO_STOPS;
        use self::skimmer::scanner::NO_BRACES;


        loop {
            /* --- WORDS --- */

            let (bytes, chars) = scan_while (reader, &self.line_breakers);
            if bytes > 0 { return Some ( (Token::Newline, bytes, chars) ) }


            let (bytes, chars) = scan_while (reader, &self.spaces);
            if bytes > 0 { return Some ( (Token::Indent, bytes, chars) ) }


            let (bytes, chars) = scan_while (reader, &self.tabs);
            if bytes > 0 { return Some ( (Token::Tab, bytes, chars) ) }


            if let Some (_) = self.cset.hashtag.read (reader) {
                let (comment, _) = scan_until (reader, &self.line_breakers);
                if comment > 0 { return Some ( (Token::Comment, comment, 1) ) }
                break;
            }


            if let Some (_) = self.cset.asterisk.read (reader) {
                let (alias, _) = scan_until (reader, &self.alias_stops);
                if alias > 0 { return Some ( (Token::Alias, alias, 1) ) }
                break;
            }


            if let Some (_) = self.cset.ampersand.read (reader) {
                let (anchor, _) = scan_until (reader, &self.anchor_stops);
                if anchor > 0 { return Some ( (Token::Anchor, anchor, 1) ) }
                break;
            }


            if let Some (_) = self.cset.quotation.read (reader) {
                if let Some ( (len, _) ) = scan (reader, &NO_STOPS, &self.scfg_escape, &self.scfg_str_dbl_quotes, &NO_BRACES, &mut []) {
                    return Some ( (Token::StringDouble, len, 1) )
                }
                break;
            }


            if let Some (_) = self.cset.apostrophe.read (reader) {
                if let Some ( (len, _) ) = scan (reader, &NO_STOPS, &self.scfg_escape, &self.scfg_str_sgl_quotes, &NO_BRACES, &mut []) {
                    return Some ( (Token::StringSingle, len, 1) )
                }
                break;
            }


            if let Some (len) = self.triple_hyphen_minus.read (reader) {
                if let Some (_) = scan_one_at (len, reader, &self.spaces_and_line_breakers) {
                    return Some ( (Token::DocumentStart, len, 0) )
                }
                break;
            }


            if let Some (ex_len) = self.cset.exclamation.read (reader) {
                let len = if let Some (lt_len) = self.cset.less_than.read_at (ex_len, reader) {
                    let (len, idx) = scan_until_at (ex_len + lt_len, reader, &self.tag_stops);
                    ex_len + lt_len + len + match idx { Some ( (_, len) ) => len, _ => 0 }

                } else {
                    let (len, _) = scan_until_at (ex_len, reader, &self.tag_flow_stops);
                    ex_len + len
                };

                return Some ( (Token::TagHandle, len, 0) )
            }


            if let Some (len) = self.cset.bracket_curly_left.read (reader) { return Some ((Token::DictionaryStart, len, 1) ) }

            if let Some (len) = self.cset.bracket_curly_right.read (reader) { return Some ((Token::DictionaryEnd, len, 1) ) }

            if let Some (len) = self.cset.bracket_square_left.read (reader) { return Some ( (Token::SequenceStart, len, 1) ) }

            if let Some (len) = self.cset.bracket_square_right.read (reader) { return Some ( (Token::SequenceEnd, len, 1) ) }

            if let Some (len) = self.cset.greater_than.read (reader) { return Some ( (Token::GT, len, 1) ) }

            if let Some (len) = self.cset.vertical_bar.read (reader) { return Some ( (Token::Pipe, len, 1) ) }


            if let Some (len) = self.triple_full_stop.read (reader) { return Some ( (Token::DocumentEnd, len, 0) ) }

            if let Some (len) = self.directive_tag.read (reader) { return Some ( (Token::DirectiveTag, len, 0) ) }

            if let Some (len) = self.directive_yaml.read (reader) { return Some ( (Token::DirectiveYaml, len, 0) ) }

            if let Some ( _ ) = self.cset.percent.read (reader) { return Some ( (Token::Directive, self.line (reader), 1) ) }


            if let Some (len) = self.cset.comma.read (reader) {
                return Some ( (Token::Comma, len, 1) )
            }


            if let Some (len) = self.cset.colon.read (reader) {
                return Some ( (Token::Colon, len, 1) )
            }


            if let Some (len) = self.cset.hyphen_minus.read (reader) {
                if !reader.has (self.cset.hyphen_minus.len () + 1) {
                    return Some ( (Token::Dash, len, 1) )
                } else {
                    if let Some (_) = scan_one_at (len, reader, &self.spaces_and_line_breakers) {
                        return Some ( (Token::Dash, len, 1) )
                    }
                }
            }


            if let Some (len) = self.cset.question.read (reader) {
                if !reader.has (self.cset.question.len () + 1) {
                    return Some ( (Token::Question, len, 1) )
                } else {
                    if let Some (_) = scan_one_at (len, reader, &self.spaces_and_line_breakers) {
                        return Some ( (Token::Question, len, 1) )
                    }
                }
            }


            if let Some (len) = self.cset.commercial_at.read (reader) {
                return Some ( (Token::ReservedCommercialAt, len, 1) )
            }


            if let Some (len) = self.cset.grave_accent.read (reader) {
                return Some ( (Token::ReservedGraveAccent, len, 1) )
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


            /* --- RAW --- */

            let (raw_len, _) = scan_until (reader, &self.raw_stops);
            if raw_len > 0 { return Some ( (Token::Raw, raw_len, 1) ) }


            break;
        }

        None
    }
}



#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use super::skimmer::reader::Read;
    use super::skimmer::reader::SliceReader;

    use txt::CharSet;
    use txt::get_charset_utf8;


    fn get_charset () -> CharSet { get_charset_utf8 () }



    #[test]
    fn test_tokenizer_general () {
        let src = "%YAML 1.2\n%TAG ! tag://example.com,2015:yamlette/\n---\n\"double string\"\n    \r'single string'\n\r[\"list\", 'of', tokens]\r\n{key: val, key: val} ...";

        // %TAG ! tag://example.com,2015:yamlette/\n

        let mut reader = SliceReader::new (src.as_bytes ());
        let tokenizer = Tokenizer::new (get_charset ());

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (5, len);
            assert_eq! (0, clen);

            if let Token::DirectiveYaml = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (3, len);
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (4, len);
            assert_eq! (0, clen);

            if let Token::DirectiveTag = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (0, clen);

            if let Token::TagHandle = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "tag".as_bytes ().len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, ":".as_bytes ().len ());
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "//example.com".as_bytes ().len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, ",".as_bytes ().len ());
            assert_eq! (1, clen);

            if let Token::Comma = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "2015".as_bytes ().len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, ":".as_bytes ().len ());
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "yamlette/".as_bytes ().len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (3, len);
            assert_eq! (0, clen);

            if let Token::DocumentStart = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "\"double string\"".len ());
            assert_eq! (1, clen);

            if let Token::StringDouble = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "\n".len ());
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "    ".len ());
            assert_eq! (4, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "\r".len ());
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "'single string'".len ());
            assert_eq! (1, clen);

            if let Token::StringSingle = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "\n\r".len ());
            assert_eq! (2, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (clen));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "[".len ());
            assert_eq! (1, clen);

            if let Token::SequenceStart = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "\"list\"".len ());
            assert_eq! (1, clen);

            if let Token::StringDouble = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Comma = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "'of'".len ());
            assert_eq! (1, clen);

            if let Token::StringSingle = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Comma = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "tokens".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "]".len ());
            assert_eq! (1, clen);

            if let Token::SequenceEnd = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "\r\n".len ());
            assert_eq! (2, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "{".len ());
            assert_eq! (1, clen);

            if let Token::DictionaryStart = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "key".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "key".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Comma = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "key".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "key".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "{".len ());
            assert_eq! (1, clen);

            if let Token::DictionaryEnd = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, 1);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (3, len);
            assert_eq! (0, clen);

            if let Token::DocumentEnd = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some (_) = tokenizer.get_token (&mut reader) { assert! (false, "Unexpected token") }
    }



    #[test]
    fn test_tokenizer_anchor () {
        let src = "- &anchor string";

        let mut reader = SliceReader::new (src.as_bytes ());
        let tokenizer = Tokenizer::new (get_charset ());

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (7, len);
            assert_eq! (1, clen);

            if let Token::Anchor = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (6, len);
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }
    }



    #[test]
    fn test_tokenizer_alias () {
        let src = "- *anchor string";

        let mut reader = SliceReader::new (src.as_bytes ());
        let tokenizer = Tokenizer::new (get_charset ());

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (7, len);
            assert_eq! (1, clen); 

            if let Token::Alias = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (6, len);
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        } else { assert! (false, "Unexpected result!") }
    }



    #[test]
    fn test_tokenizer_e_2_1 () {
        let src =
r"- Mark McGwire
- Sammy Sosa
- Ken Griffey";


        let mut reader = SliceReader::new (src.as_bytes ());
        let tokenizer = Tokenizer::new (get_charset ());

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "Mark McGwire".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (clen, reader.skip (clen));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "Sammy Sosa".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "Ken Griffey".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }
    }



    #[test]
    fn test_tokenizer_e_2_2 () {
        let src =
r"hr:  65    # Home runs
avg: 0.278 # Batting average
rbi: 147   # Runs Batted In";

        let mut reader = SliceReader::new (src.as_bytes ());
        let tokenizer = Tokenizer::new (get_charset ());

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "hr".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "65    ".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "# Home runs".len ());
            assert_eq! (1, clen);

            if let Token::Comment = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "avg".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "0.278 ".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "# Batting average".len ());
            assert_eq! (1, clen);

            if let Token::Comment = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "rbi".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "147   ".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "# Runs Batted In".len ());
            assert_eq! (1, clen);

            if let Token::Comment = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
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
        let tokenizer = Tokenizer::new (get_charset ());

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "american".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "Boston Red Sox".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "Detroit Tigers".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "New York Yankees".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "national".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "New York Mets".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "Chicago Cubs".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "Atlanta Braves".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
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
        let tokenizer = Tokenizer::new (get_charset ());

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (len, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "name".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "Mark McGwire".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "hr".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (3, len);
            assert_eq! (3, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "65".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "avg".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "0.278".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Dash = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "name".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "Sammy Sosa".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "hr".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (3, len);
            assert_eq! (3, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "63".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }


        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Newline = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "avg".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (1, len);
            assert_eq! (1, clen);

            if let Token::Colon = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (2, len);
            assert_eq! (2, clen);

            if let Token::Indent = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }

        if let Some ( (token, len, clen) ) = tokenizer.get_token (&mut reader) {
            assert_eq! (len, "0.288".len ());
            assert_eq! (1, clen);

            if let Token::Raw = token {  } else { assert! (false, "Unexpected token!") }

            assert_eq! (len, reader.skip (len));
        }
    }
}
