extern crate skimmer;

use self::skimmer::reader::Read;

#[derive(Debug)]
pub enum Token {
    /*
    BOM32BE,
    BOM32LE,
    BOM16BE,
    BOM16LE,
    */
    BOM8,

    DirectiveTag,  // %TAG
    DirectiveYaml, // %YAML
    Directive,     // % // TODO: directives

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

pub fn scan_while_spaces_and_tabs<Reader: Read>(reader: &mut Reader, at: usize) -> usize {
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

pub fn scan_one_colon_and_line_breakers<Reader: Read>(reader: &mut Reader, at: usize) -> usize {
    match reader.get_byte_at(at) {
        Some(b':') => 1,
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

pub fn scan_until_colon_and_line_breakers<Reader: Read>(reader: &mut Reader, at: usize) -> usize {
    let mut scanned = at;
    loop {
        match reader.get_byte_at(scanned) {
            None | Some(b':') | Some(b'\n') | Some(b'\r') => break,
            _ => scanned += 1,
        };
    }
    scanned - at
}

pub fn scan_until_question_and_line_breakers<Reader: Read>(
    reader: &mut Reader,
    at: usize,
) -> usize {
    let mut scanned = at;
    loop {
        match reader.get_byte_at(scanned) {
            None | Some(b'?') | Some(b'\n') | Some(b'\r') => break,
            _ => scanned += 1,
        };
    }
    scanned - at
}

pub fn scan_one_spaces_and_line_breakers<Reader: Read>(reader: &mut Reader, at: usize) -> usize {
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

pub fn scan_while_spaces_and_line_breakers<Reader: Read>(reader: &mut Reader, at: usize) -> usize {
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

pub fn scan_one_line_breaker<Reader: Read>(reader: &mut Reader, at: usize) -> usize {
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

fn alias_stops<Reader: Read>(reader: &mut Reader) -> usize {
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

pub fn anchor_stops<Reader: Read>(reader: &mut Reader, at: usize) -> usize {
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

pub fn raw_stops<Reader: Read>(reader: &mut Reader) -> usize {
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

fn tag_flow_stops<Reader: Read>(reader: &mut Reader, at: usize) -> usize {
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

pub fn line<Reader: Read>(reader: &mut Reader) -> usize {
    let mut scanned = 0;
    loop {
        match reader.get_byte_at(scanned) {
            None | Some(b'\n') | Some(b'\r') => break,
            _ => scanned += 1,
        };
    }
    scanned
}

pub fn line_at<Reader: Read>(reader: &mut Reader, at: usize) -> usize {
    let mut scanned = at;
    loop {
        match reader.get_byte_at(scanned) {
            None | Some(b'\n') | Some(b'\r') => break,
            _ => scanned += 1,
        };
    }
    scanned - at
}

pub fn get_token<Reader: Read>(reader: &mut Reader) -> Option<(Token, usize, usize)> {
    match reader.get_byte_at_start() {
        None => return None,
        Some(b',') => return Some((Token::Comma, 1, 1)),
        Some(b':') => return Some((Token::Colon, 1, 1)),
        Some(b'{') => return Some((Token::DictionaryStart, 1, 1)),
        Some(b'}') => return Some((Token::DictionaryEnd, 1, 1)),
        Some(b'[') => return Some((Token::SequenceStart, 1, 1)),
        Some(b']') => return Some((Token::SequenceEnd, 1, 1)),
        Some(b'>') => return Some((Token::GT, 1, 1)),
        Some(b'|') => return Some((Token::Pipe, 1, 1)),
        Some(b'"') => return Some((Token::StringDouble, scan_double_quoted(reader), 1)),
        Some(b'\'') => return Some((Token::StringSingle, scan_single_quoted(reader), 1)),
        Some(b'#') => return Some((Token::Comment, line_at(reader, 1) + 1, 1)),
        Some(b'*') => return Some((Token::Alias, alias_stops(reader), 1)),
        Some(b'&') => return Some((Token::Anchor, anchor_stops(reader, 1) + 1, 1)),
        Some(b'.') => {
            if let Some((b'.', b'.')) = reader.get_bytes_2_at(1) {
                return Some((Token::DocumentEnd, 3, 0));
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
            return Some((Token::Indent, scanned, scanned));
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

            return Some((Token::Newline, scanned, scanned));
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

            return Some((Token::TagHandle, scanned, 0));
        }

        Some(b'%') => match reader.get_bytes_4_at(1) {
            None => match reader.get_bytes_3_at(1) {
                Some((b'T', b'A', b'G')) => return Some((Token::DirectiveTag, 4, 0)),
                // _ => return Some ((Token::DirectiveYaml, line_at (reader, 1) + 1, 0))
                _ => return Some((Token::Directive, line_at(reader, 1) + 1, 0)),
            },
            Some((b'Y', b'A', b'M', b'L')) => return Some((Token::DirectiveYaml, 5, 0)),
            Some((b'T', b'A', b'G', _)) => return Some((Token::DirectiveTag, 4, 0)),
            // _ => return Some ((Token::DirectiveYaml, line_at (reader, 1) + 1, 0))
            _ => return Some((Token::Directive, line_at(reader, 1) + 1, 0)),
        },

        Some(b'-') => match reader.get_byte_at(1) {
            None => return Some((Token::Dash, 1, 1)),
            Some(b'-') => {
                if let Some(b'-') = reader.get_byte_at(2) {
                    return Some((Token::DocumentStart, 3, 0));
                };
            }
            _ => {
                if scan_one_spaces_and_line_breakers(reader, 1) > 0 {
                    return Some((Token::Dash, 1, 1));
                };
            }
        },

        Some(b'?') => match reader.get_byte_at(1) {
            None => return Some((Token::Question, 1, 1)),
            _ => {
                if scan_one_spaces_and_line_breakers(reader, 1) > 0 {
                    return Some((Token::Question, 1, 1));
                };
            }
        },

        Some(b'@') => return Some((Token::ReservedCommercialAt, 1, 1)),
        Some(b'`') => return Some((Token::ReservedGraveAccent, 1, 1)),

        Some(b'\t') => {
            let mut scanned = 1;
            loop {
                match reader.get_byte_at(scanned) {
                    Some(b't') => {
                        scanned += 1;
                    }
                    _ => break,
                }
            }
            return Some((Token::Tab, scanned, scanned));
        }

        Some(0xEF) => match reader.get_bytes_2_at(1) {
            Some((0xBB, 0xBF)) => return Some((Token::BOM8, 3, 1)),
            _ => (),
        },

        _ => (),
    };

    let scanned = raw_stops(reader);
    if scanned > 0 {
        Some((Token::Raw, scanned, 1))
    } else {
        None
    }
}

#[cfg(all(test, not(feature = "dev")))]
mod tests {
    use super::*;

    use super::skimmer::reader::Read;
    use super::skimmer::reader::SliceReader;

    // use txt::get_charset_utf8;

    #[test]
    fn test_tokenizer_general() {
        let src = "%YAML 1.2\n%TAG ! tag://example.com,2015:yamlette/\n---\n\"double string\"\n    \r'single string'\n\r[\"list\", 'of', tokens]\r\n{key: val, key: val} ...";

        // %TAG ! tag://example.com,2015:yamlette/\n

        let mut reader = SliceReader::new(src.as_bytes());
        // let tokenizer = Tokenizer; // ::new (get_charset_utf8 ());

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(5, len);
            assert_eq!(0, clen);

            if let Token::DirectiveYaml = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(3, len);
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(4, len);
            assert_eq!(0, clen);

            if let Token::DirectiveTag = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(0, clen);

            if let Token::TagHandle = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "tag".as_bytes().len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, ":".as_bytes().len());
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "//example.com".as_bytes().len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, ",".as_bytes().len());
            assert_eq!(1, clen);

            if let Token::Comma = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "2015".as_bytes().len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, ":".as_bytes().len());
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "yamlette/".as_bytes().len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(3, len);
            assert_eq!(0, clen);

            if let Token::DocumentStart = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "\"double string\"".len());
            assert_eq!(1, clen);

            if let Token::StringDouble = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "\n".len());
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "    ".len());
            assert_eq!(4, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "\r".len());
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "'single string'".len());
            assert_eq!(1, clen);

            if let Token::StringSingle = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "\n\r".len());
            assert_eq!(2, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(clen));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "[".len());
            assert_eq!(1, clen);

            if let Token::SequenceStart = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "\"list\"".len());
            assert_eq!(1, clen);

            if let Token::StringDouble = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, 1);
            assert_eq!(1, clen);

            if let Token::Comma = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, 1);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "'of'".len());
            assert_eq!(1, clen);

            if let Token::StringSingle = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, 1);
            assert_eq!(1, clen);

            if let Token::Comma = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, 1);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "tokens".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "]".len());
            assert_eq!(1, clen);

            if let Token::SequenceEnd = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "\r\n".len());
            assert_eq!(2, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "{".len());
            assert_eq!(1, clen);

            if let Token::DictionaryStart = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "key".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, 1);
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, 1);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "key".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, 1);
            assert_eq!(1, clen);

            if let Token::Comma = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, 1);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "key".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, 1);
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, 1);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "key".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "{".len());
            assert_eq!(1, clen);

            if let Token::DictionaryEnd = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, 1);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(3, len);
            assert_eq!(0, clen);

            if let Token::DocumentEnd = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some(_) = get_token(&mut reader) {
            assert!(false, "Unexpected token")
        }
    }

    #[test]
    fn test_tokenizer_anchor() {
        let src = "- &anchor string";

        let mut reader = SliceReader::new(src.as_bytes());
        // let tokenizer = Tokenizer; // ::new (get_charset_utf8 ());

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Dash = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(7, len);
            assert_eq!(1, clen);

            if let Token::Anchor = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(6, len);
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }
    }

    #[test]
    fn test_tokenizer_alias() {
        let src = "- *anchor string";

        let mut reader = SliceReader::new(src.as_bytes());
        // let tokenizer = Tokenizer; // ::new (get_charset_utf8 ());

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Dash = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(7, len);
            assert_eq!(1, clen);

            if let Token::Alias = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(6, len);
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        } else {
            assert!(false, "Unexpected result!")
        }
    }

    #[test]
    fn test_tokenizer_e_2_1() {
        let src = r"- Mark McGwire
- Sammy Sosa
- Ken Griffey";

        let mut reader = SliceReader::new(src.as_bytes());
        // let tokenizer = Tokenizer; // ::new (get_charset_utf8 ());

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Dash = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "Mark McGwire".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Dash = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(clen, reader.skip_long(clen));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "Sammy Sosa".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Dash = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "Ken Griffey".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }
    }

    #[test]
    fn test_tokenizer_e_2_2() {
        let src = r"hr:  65    # Home runs
avg: 0.278 # Batting average
rbi: 147   # Runs Batted In";

        let mut reader = SliceReader::new(src.as_bytes());
        // let tokenizer = Tokenizer; // ::new (get_charset_utf8 ());

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "hr".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "65    ".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "# Home runs".len());
            assert_eq!(1, clen);

            if let Token::Comment = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "avg".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "0.278 ".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "# Batting average".len());
            assert_eq!(1, clen);

            if let Token::Comment = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "rbi".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "147   ".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "# Runs Batted In".len());
            assert_eq!(1, clen);

            if let Token::Comment = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }
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

        let mut reader = SliceReader::new(src.as_bytes());
        // let tokenizer = Tokenizer; // ::new (get_charset_utf8 ());

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "american".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Dash = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "Boston Red Sox".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Dash = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "Detroit Tigers".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Dash = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "New York Yankees".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "national".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Dash = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "New York Mets".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Dash = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "Chicago Cubs".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Dash = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "Atlanta Braves".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }
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

        let mut reader = SliceReader::new(src.as_bytes());
        // let tokenizer = Tokenizer; // ::new (get_charset_utf8 ());

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(len, clen);

            if let Token::Dash = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "name".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "Mark McGwire".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "hr".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(3, len);
            assert_eq!(3, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "65".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "avg".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "0.278".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Dash = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "name".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "Sammy Sosa".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "hr".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(3, len);
            assert_eq!(3, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "63".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Newline = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "avg".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(1, len);
            assert_eq!(1, clen);

            if let Token::Colon = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(2, len);
            assert_eq!(2, clen);

            if let Token::Indent = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }

        if let Some((token, len, clen)) = get_token(&mut reader) {
            assert_eq!(len, "0.288".len());
            assert_eq!(1, clen);

            if let Token::Raw = token {
            } else {
                assert!(false, "Unexpected token!")
            }

            assert_eq!(len, reader.skip_long(len));
        }
    }
}
