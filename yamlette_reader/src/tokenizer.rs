pub use skimmer::reader::Read;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TokenLabel {
    BOM8,

    DirectiveTag,
    DirectiveYaml,
    DirectiveUnknown,

    DocumentStart, // ---
    DocumentEnd,   // ...

    Comment, // #

    Newline, // \n+
    Indent,  // \s+
    Tab,     // \t+

    GT,       // >
    Dash,     // -
    Colon,    // :
    Comma,    // ,
    Pipe,     // |
    Question, // ?

    Anchor,    // &
    Alias,     // *
    TagHandle, // !

    DictionaryStart, // {
    DictionaryEnd,   // }

    SequenceStart, // [
    SequenceEnd,   // ]

    StringDouble, // "
    StringSingle, // '

    Raw,

    ReservedCommercialAt, // @
    ReservedGraveAccent,  // `
}

#[derive(Debug)]
pub struct Token {
    pub label: TokenLabel,
    pub length: usize,
}

impl Token {
    pub fn new(label: TokenLabel, length: usize) -> Self {
        Self { label, length }
    }
}

/// Scan the following bytes until the closing double-quote character (").
/// Takes into account backslash as the escape character, thus it will skip
/// the escaped double-quotes (\").
/// Takes into account double-backslash as escape of the escape character,
/// so that \\" still counts as the closing double-quote.
/// See YAML 1.2 spec, 7.3.1. Single-Quoted Style
pub fn scan_double_quoted<R: Read>(reader: &mut R) -> usize {
    let mut scanned = if let Some(b'"') = reader.get_byte_at_start() {
        1
    } else {
        return 0;
    };

    loop {
        match reader.get_byte_at(scanned) {
            None => break,
            Some(b'"') => {
                scanned += 1;
                break;
            }
            Some(b'\\') => match reader.get_byte_at(scanned + 1) {
                Some(b'"') | Some(b'\\') => {
                    scanned += 2;
                }
                None => {
                    scanned += 1;
                    break;
                }
                _ => {
                    scanned += 1;
                }
            },
            _ => scanned += 1,
        };
    }

    scanned
}

/// Scan the following bytes until the closing single-quote character (').
/// Takes into account double-single-quote as the escape sequence, thus it will skip
/// the escaped single-quotes ('').
/// See YAML 1.2 spec, 7.3.2. Single-Quoted Style
pub fn scan_single_quoted<R: Read>(reader: &mut R) -> usize {
    let mut scanned = if let Some(b'\'') = reader.get_byte_at_start() {
        1
    } else {
        return 0;
    };

    loop {
        match reader.get_byte_at(scanned) {
            None => break,
            Some(b'\'') => match reader.get_byte_at(scanned + 1) {
                Some(b'\'') => scanned += 2,
                _ => {
                    scanned += 1;
                    break;
                }
            },
            _ => scanned += 1,
        }
    }

    scanned
}

/// Scan the following bytes only while they are white spaces (' ') and horizontal tabs ("\t")
pub fn scan_while_spaces_and_tabs<R: Read>(reader: &mut R, at: usize) -> usize {
    let mut scanned = at;

    loop {
        match reader.get_byte_at(scanned) {
            Some(b' ') | Some(b'\t') => {
                scanned += 1;
                continue;
            }
            _ => break,
        }
    }

    scanned - at
}

/// Scan the following bytes until the first colon (:) or a newline
pub fn scan_until_colon_and_line_breakers<R: Read>(reader: &mut R, at: usize) -> usize {
    let mut scanned = at;
    loop {
        match reader.get_byte_at(scanned) {
            None | Some(b':') | Some(b'\n') | Some(b'\r') => break,
            _ => scanned += 1,
        };
    }
    scanned - at
}

/// Scan the following bytes until the first question (?) or a newline
pub fn scan_until_question_and_line_breakers<R: Read>(reader: &mut R, at: usize) -> usize {
    let mut scanned = at;
    loop {
        match reader.get_byte_at(scanned) {
            None | Some(b'?') | Some(b'\n') | Some(b'\r') => break,
            _ => scanned += 1,
        };
    }
    scanned - at
}

/// Scan one or two following bytes only if they are a space (' '), horizontal tab ("\t") or a newline
pub fn scan_one_spaces_and_line_breakers<R: Read>(reader: &mut R, at: usize) -> usize {
    match reader.get_byte_at(at) {
        Some(b' ') => 1,
        Some(b'\n') => 1,
        Some(b'\t') => 1,
        Some(b'\r') => {
            if let Some(b'\n') = reader.get_byte_at(at + 1) {
                2
            } else {
                1
            }
        }
        _ => 0,
    }
}

/// Scan the following bytes only while they are a space (' '), horizontal tab ("\t") or a newline
pub fn scan_while_spaces_and_line_breakers<R: Read>(reader: &mut R, at: usize) -> usize {
    let mut scanned = at;

    loop {
        match reader.get_byte_at(scanned) {
            Some(b' ') => scanned += 1,
            Some(b'\n') => scanned += 1,
            Some(b'\t') => scanned += 1,
            Some(b'\r') => {
                scanned += if let Some(b'\n') = reader.get_byte_at(scanned + 1) {
                    2
                } else {
                    1
                }
            }
            _ => break,
        };
    }

    scanned - at
}

/// Scan one newline sequence ("\n", "\r", "\r\n")
pub fn scan_one_line_breaker<R: Read>(reader: &mut R, at: usize) -> usize {
    match reader.get_byte_at(at) {
        Some(b'\n') => 1,
        Some(b'\r') => {
            if let Some(b'\n') = reader.get_byte_at(at + 1) {
                2
            } else {
                1
            }
        }
        _ => 0,
    }
}

/// Scan the following bytes until meeting an alias stop sequence which are:
/// ' ', "\t", "\n", "\r", "{", "}", "[", "]",
/// ": ", ":\t", ":\n", ":\r", ":,", ", ",
/// ",\t", ",\n", ",\r", ",:"
fn alias_stops<R: Read>(reader: &mut R) -> usize {
    let mut scanned = 0;

    loop {
        match reader.get_byte_at(scanned) {
            None => break,

            Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') | Some(b'{') | Some(b'}')
            | Some(b'[') | Some(b']') => break,

            Some(b':') => match reader.get_byte_at(scanned + 1) {
                Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') | Some(b',') => break,
                _ => (),
            },

            Some(b',') => match reader.get_byte_at(scanned + 1) {
                Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') | Some(b':') => break,
                _ => (),
            },

            _ => (),
        };

        scanned += 1
    }

    scanned
}

/// Scan the following bytes until meeting an anchor stop sequence which are:
/// ' ', "\t", "\n", "\r", '{', '}', '[', ']'
pub fn anchor_stops<R: Read>(reader: &mut R, at: usize) -> usize {
    let mut scanned = at;

    loop {
        match reader.get_byte_at(scanned) {
            None | Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') | Some(b'{')
            | Some(b'}') | Some(b'[') | Some(b']') => break,
            _ => scanned += 1,
        };
    }

    scanned - at
}

/// Scan the following bytes until meeting a raw stop sequence which are:
/// "\n", "\r", '{', '}', '[', ']', '#', ':', ','
pub fn raw_stops<R: Read>(reader: &mut R) -> usize {
    let mut scanned = 0;

    loop {
        match reader.get_byte_at(scanned) {
            None | Some(b'\n') | Some(b'\r') | Some(b'{') | Some(b'}') | Some(b'[')
            | Some(b']') | Some(b'#') | Some(b':') | Some(b',') => break,
            _ => scanned += 1,
        };
    }

    scanned
}

/// Scan the following bytes until meeting a tag flow stop sequence which are:
/// ' ', "\t", "\n", "\r", '{', '}', '[', ']', ':', ','
fn tag_flow_stops<R: Read>(reader: &mut R, at: usize) -> usize {
    let mut scanned = at;

    loop {
        match reader.get_byte_at(scanned) {
            None | Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') | Some(b'{')
            | Some(b'}') | Some(b'[') | Some(b']') | Some(b':') | Some(b',') => break,
            _ => scanned += 1,
        };
    }

    scanned - at
}

/// Reads until the following newline (exclusive)
pub fn line<R: Read>(reader: &mut R) -> usize {
    let mut scanned = 0;
    loop {
        match reader.get_byte_at(scanned) {
            None | Some(b'\n') | Some(b'\r') => break,
            _ => scanned += 1,
        };
    }
    scanned
}

/// Reads until the following newline (exclusive), starting at the position given via "at" parameter
pub fn line_at<R: Read>(reader: &mut R, at: usize) -> usize {
    let mut scanned = at;
    loop {
        match reader.get_byte_at(scanned) {
            None | Some(b'\n') | Some(b'\r') => break,
            _ => scanned += 1,
        };
    }
    scanned - at
}

pub fn get_token<R: Read>(reader: &mut R) -> Option<Token> {
    match reader.get_byte_at_start() {
        None => return None,
        Some(b',') => return Some(Token::new(TokenLabel::Comma, 1)),
        Some(b':') => return Some(Token::new(TokenLabel::Colon, 1)),
        Some(b'{') => return Some(Token::new(TokenLabel::DictionaryStart, 1)),
        Some(b'}') => return Some(Token::new(TokenLabel::DictionaryEnd, 1)),
        Some(b'[') => return Some(Token::new(TokenLabel::SequenceStart, 1)),
        Some(b']') => return Some(Token::new(TokenLabel::SequenceEnd, 1)),
        Some(b'>') => return Some(Token::new(TokenLabel::GT, 1)),
        Some(b'|') => return Some(Token::new(TokenLabel::Pipe, 1)),
        Some(b'"') => {
            return Some(Token::new(
                TokenLabel::StringDouble,
                scan_double_quoted(reader),
            ))
        }
        Some(b'\'') => {
            return Some(Token::new(
                TokenLabel::StringSingle,
                scan_single_quoted(reader),
            ))
        }
        Some(b'#') => return Some(Token::new(TokenLabel::Comment, line_at(reader, 1) + 1)),
        Some(b'*') => return Some(Token::new(TokenLabel::Alias, alias_stops(reader))),
        Some(b'&') => return Some(Token::new(TokenLabel::Anchor, anchor_stops(reader, 1) + 1)),

        Some(b'.') => {
            if let Some((b'.', b'.')) = reader.get_bytes_2_at(1) {
                return Some(Token::new(TokenLabel::DocumentEnd, 3));
            };
        }

        Some(b' ') => {
            let mut scanned = 1;
            loop {
                match reader.get_byte_at(scanned) {
                    Some(b' ') => {
                        scanned += 1;
                    }
                    _ => break,
                }
            }
            return Some(Token::new(TokenLabel::Indent, scanned));
        }

        Some(b'\n') | Some(b'\r') => {
            let mut scanned = 1;

            loop {
                match reader.get_byte_at(scanned) {
                    Some(b'\n') | Some(b'\r') => {
                        scanned += 1;
                    }
                    _ => break,
                }
            }

            return Some(Token::new(TokenLabel::Newline, scanned));
        }

        Some(b'!') => {
            let mut scanned = 0;
            if let Some(b'<') = reader.get_byte_at(1) {
                scanned += 2;
                loop {
                    match reader.get_byte_at(scanned) {
                        Some(b'>') => {
                            scanned += 1;
                            break;
                        }
                        _ => scanned += 1,
                    };
                }
            } else {
                scanned = tag_flow_stops(reader, 1) + 1;
            };

            return Some(Token::new(TokenLabel::TagHandle, scanned));
        }

        Some(b'%') => match reader.get_bytes_4_at(1) {
            None => match reader.get_bytes_3_at(1) {
                Some((b'T', b'A', b'G')) => return Some(Token::new(TokenLabel::DirectiveTag, 4)),
                _ => {
                    return Some(Token::new(
                        TokenLabel::DirectiveUnknown,
                        line_at(reader, 1) + 1,
                    ))
                }
            },
            Some((b'Y', b'A', b'M', b'L')) => {
                return Some(Token::new(TokenLabel::DirectiveYaml, 5))
            }
            Some((b'T', b'A', b'G', _)) => return Some(Token::new(TokenLabel::DirectiveTag, 4)),
            _ => {
                return Some(Token::new(
                    TokenLabel::DirectiveUnknown,
                    line_at(reader, 1) + 1,
                ))
            }
        },

        Some(b'-') => match reader.get_byte_at(1) {
            None => return Some(Token::new(TokenLabel::Dash, 1)),
            Some(b'-') => {
                if let Some(b'-') = reader.get_byte_at(2) {
                    return Some(Token::new(TokenLabel::DocumentStart, 3));
                };
            }
            _ => {
                if scan_one_spaces_and_line_breakers(reader, 1) > 0 {
                    return Some(Token::new(TokenLabel::Dash, 1));
                };
            }
        },

        Some(b'?') => match reader.get_byte_at(1) {
            None => return Some(Token::new(TokenLabel::Question, 1)),
            _ => {
                if scan_one_spaces_and_line_breakers(reader, 1) > 0 {
                    return Some(Token::new(TokenLabel::Question, 1));
                };
            }
        },

        Some(b'@') => return Some(Token::new(TokenLabel::ReservedCommercialAt, 1)),
        Some(b'`') => return Some(Token::new(TokenLabel::ReservedGraveAccent, 1)),

        Some(b'\t') => {
            let mut scanned = 1;
            loop {
                match reader.get_byte_at(scanned) {
                    Some(b'\t') => {
                        scanned += 1;
                    }
                    _ => break,
                }
            }
            return Some(Token::new(TokenLabel::Tab, scanned));
        }

        Some(0xEF) => match reader.get_bytes_2_at(1) {
            Some((0xBB, 0xBF)) => return Some(Token::new(TokenLabel::BOM8, 3)),
            _ => (),
        },

        _ => (),
    };

    let scanned = raw_stops(reader);
    if scanned > 0 {
        Some(Token::new(TokenLabel::Raw, scanned))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::borrow::Cow;
    use skimmer::reader::SliceReader;

    macro_rules! init_reader {
        ( $yaml_string:expr ) => {
            SliceReader::new($yaml_string.as_bytes())
        };
    }

    macro_rules! assert_token {
        ( $value:tt, $label:pat, $reader:ident ) => {
            let value_length = $value.len();

            match get_token(&mut $reader) {
                Some(
                    Token {
                        label: $label,
                        length: token_length,
                    },
                ) if token_length == value_length => {
                    assert_eq!($value, String::from_utf8_lossy($reader.slice_at(0, token_length).unwrap()));
                    assert_eq!(value_length, $reader.skip_long(token_length));
                },
                token @ _ => assert!(
                    false,
                    "Unexpected token: {:?} => {:?} , expected token: {:?} => {{ {:?}, {} }}",
                    match token {
                        Some(Token { length: length, .. }) => String::from_utf8_lossy($reader.slice_at(0, length).unwrap()),
                        _ => Cow::from(String::new())
                    },
                    token,
                    $value,
                    stringify!($label),
                    value_length
                ),
            };
        };
    }

    macro_rules! assert_end {
        ( $reader:ident ) => {
            if let Some(token) = get_token(&mut $reader) {
                let value = String::from_utf8_lossy($reader.slice_at(0, token.length).unwrap());
                assert!(false, "Unexpected token at the end: {:?} => {:?}", value, token)
            }
        };
    }

    #[test]
    fn test_tokenizer_general() {
        let src = "%YAML 1.2\n%TAG ! tag://example.com,2015:yamlette/\n---\n\"double string\"\n    \r'single string'\n\r[\"list\", 'of', tokens]\r\n{key: val, key: val} ...";

        let mut reader = init_reader!(src);

        assert_token!("%YAML", TokenLabel::DirectiveYaml, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("1.2", TokenLabel::Raw, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("%TAG", TokenLabel::DirectiveTag, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("!", TokenLabel::TagHandle, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("tag", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!("//example.com", TokenLabel::Raw, reader);
        assert_token!(",", TokenLabel::Comma, reader);
        assert_token!("2015", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!("yamlette/", TokenLabel::Raw, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("---", TokenLabel::DocumentStart, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("\"double string\"", TokenLabel::StringDouble, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("    ", TokenLabel::Indent, reader);
        assert_token!("\r", TokenLabel::Newline, reader);
        assert_token!("'single string'", TokenLabel::StringSingle, reader);
        assert_token!("\n\r", TokenLabel::Newline, reader);
        assert_token!("[", TokenLabel::SequenceStart, reader);
        assert_token!("\"list\"", TokenLabel::StringDouble, reader);
        assert_token!(",", TokenLabel::Comma, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("'of'", TokenLabel::StringSingle, reader);
        assert_token!(",", TokenLabel::Comma, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("tokens", TokenLabel::Raw, reader);
        assert_token!("]", TokenLabel::SequenceEnd, reader);
        assert_token!("\r\n", TokenLabel::Newline, reader);
        assert_token!("{", TokenLabel::DictionaryStart, reader);
        assert_token!("key", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("val", TokenLabel::Raw, reader);
        assert_token!(",", TokenLabel::Comma, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("key", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("val", TokenLabel::Raw, reader);
        assert_token!("}", TokenLabel::DictionaryEnd, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("...", TokenLabel::DocumentEnd, reader);

        assert_end!(reader);
    }

    #[test]
    fn test_tokenizer_anchor() {
        let src = "- &anchor string";

        let mut reader = init_reader!(src);

        assert_token!("-", TokenLabel::Dash, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("&anchor", TokenLabel::Anchor, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("string", TokenLabel::Raw, reader);

        assert_end!(reader);
    }

    #[test]
    fn test_tokenizer_alias() {
        let src = "- *anchor string";

        let mut reader = init_reader!(src);

        assert_token!("-", TokenLabel::Dash, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("*anchor", TokenLabel::Alias, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("string", TokenLabel::Raw, reader);

        assert_end!(reader);
    }

    #[test]
    fn test_tokenizer_e_2_1() {
        let src = r"- Mark McGwire
- Sammy Sosa
- Ken Griffey";

        let mut reader = init_reader!(src);

        assert_token!("-", TokenLabel::Dash, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("Mark McGwire", TokenLabel::Raw, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("-", TokenLabel::Dash, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("Sammy Sosa", TokenLabel::Raw, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("-", TokenLabel::Dash, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("Ken Griffey", TokenLabel::Raw, reader);

        assert_end!(reader);
    }

    #[test]
    fn test_tokenizer_e_2_2() {
        let src = r"hr:  65    # Home runs
avg: 0.278 # Batting average
rbi: 147   # Runs Batted In";

        let mut reader = init_reader!(src);

        assert_token!("hr", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("65    ", TokenLabel::Raw, reader);
        assert_token!("# Home runs", TokenLabel::Comment, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("avg", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("0.278 ", TokenLabel::Raw, reader);
        assert_token!("# Batting average", TokenLabel::Comment, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("rbi", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("147   ", TokenLabel::Raw, reader);
        assert_token!("# Runs Batted In", TokenLabel::Comment, reader);

        assert_end!(reader);
    }


    #[test]
    fn test_tokenizer_e_2_3() {
        let src = r"american:
  - Boston Red Sox
  - Detroit Tigers
  - New York Yankees
national:
  - New York Mets
  - Chicago Cubs
  - Atlanta Braves";

        let mut reader = init_reader!(src);

        assert_token!("american", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("-", TokenLabel::Dash, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("Boston Red Sox", TokenLabel::Raw, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("-", TokenLabel::Dash, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("Detroit Tigers", TokenLabel::Raw, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("-", TokenLabel::Dash, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("New York Yankees", TokenLabel::Raw, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("national", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("-", TokenLabel::Dash, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("New York Mets", TokenLabel::Raw, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("-", TokenLabel::Dash, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("Chicago Cubs", TokenLabel::Raw, reader);
        assert_token!("\n", TokenLabel::Newline, reader);
        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("-", TokenLabel::Dash, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("Atlanta Braves", TokenLabel::Raw, reader);
        

        assert_end!(reader);
    }

    #[test]
    fn test_tokenizer_e_2_4() {
        let src = r"-
  name: Mark McGwire
  hr:   65
  avg:  0.278
-
  name: Sammy Sosa
  hr:   63
  avg:  0.288";

        let mut reader = init_reader!(src);

        assert_token!("-", TokenLabel::Dash, reader);
        assert_token!("\n", TokenLabel::Newline, reader);

        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("name", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("Mark McGwire", TokenLabel::Raw, reader);
        assert_token!("\n", TokenLabel::Newline, reader);

        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("hr", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!("   ", TokenLabel::Indent, reader);
        assert_token!("65", TokenLabel::Raw, reader);
        assert_token!("\n", TokenLabel::Newline, reader);

        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("avg", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("0.278", TokenLabel::Raw, reader);
        assert_token!("\n", TokenLabel::Newline, reader);

        assert_token!("-", TokenLabel::Dash, reader);
        assert_token!("\n", TokenLabel::Newline, reader);

        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("name", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!(" ", TokenLabel::Indent, reader);
        assert_token!("Sammy Sosa", TokenLabel::Raw, reader);
        assert_token!("\n", TokenLabel::Newline, reader);

        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("hr", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!("   ", TokenLabel::Indent, reader);
        assert_token!("63", TokenLabel::Raw, reader);
        assert_token!("\n", TokenLabel::Newline, reader);

        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("avg", TokenLabel::Raw, reader);
        assert_token!(":", TokenLabel::Colon, reader);
        assert_token!("  ", TokenLabel::Indent, reader);
        assert_token!("0.288", TokenLabel::Raw, reader);

        assert_end!(reader);
    }
}