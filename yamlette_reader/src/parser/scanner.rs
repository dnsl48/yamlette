pub fn get_byte_at<R: AsRef<[u8]>>(source: &R, at: usize) -> Option<u8> {
    <R as Readable>::get_byte_at(source, at)
}

trait Readable {
    fn get_byte_at_start(&self) -> Option<u8>;

    fn get_byte_at(&self, at: usize) -> Option<u8>;
}

impl<T> Readable for T
where
    T: AsRef<[u8]>,
{
    fn get_byte_at(&self, at: usize) -> Option<u8> {
        self.as_ref().get(at).map(|b| *b)
    }

    fn get_byte_at_start(&self) -> Option<u8> {
        self.as_ref().get(0).map(|b| *b)
    }
}

/// Scan the following bytes until the closing double-quote character (").
/// Takes into account backslash as the escape character, thus it will skip
/// the escaped double-quotes (\").
/// Takes into account double-backslash as escape of the escape character,
/// so that \\" still counts as the closing double-quote.
/// See YAML 1.2 spec, 7.3.1. Single-Quoted Style
pub fn scan_double_quoted<R: AsRef<[u8]>>(readable: &R) -> usize {
    let mut scanned = if let Some(b'"') = readable.get_byte_at_start() {
        1
    } else {
        return 0;
    };

    loop {
        match readable.get_byte_at(scanned) {
            None => break,
            Some(b'"') => {
                scanned += 1;
                break;
            }
            Some(b'\\') => match readable.get_byte_at(scanned + 1) {
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
pub fn scan_single_quoted<R: AsRef<[u8]>>(readable: &R) -> usize {
    let mut scanned = if let Some(b'\'') = readable.get_byte_at_start() {
        1
    } else {
        return 0;
    };

    loop {
        match readable.get_byte_at(scanned) {
            None => break,
            Some(b'\'') => match readable.get_byte_at(scanned + 1) {
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

pub fn scan_single_tag_handle<R: AsRef<[u8]>>(readable: &R) -> usize {
    let mut scanned = 0;

    match readable.get_byte_at_start() {
        Some(b'!') => {
            scanned += 1;
        }
        _ => return 0,
    };

    match readable.get_byte_at(scanned) {
        Some(b'<') => {
            scanned += 1;

            loop {
                match readable.get_byte_at(scanned) {
                    Some(b'>') => {
                        scanned += 1;
                        break;
                    }
                    _ => scanned += 1,
                };
            }
        }
        _ => {
            scanned += scan_until_tag_flow_stops(readable, scanned);
        }
    };

    scanned
}

/// Scan the following bytes only while they are white spaces (' ')
pub fn scan_while_spaces<R: AsRef<[u8]>>(readable: &R, at: usize) -> usize {
    let mut scanned = at;

    loop {
        match readable.get_byte_at(scanned) {
            Some(b' ') => {
                scanned += 1;
                continue;
            }
            _ => break,
        }
    }

    scanned - at
}

/// Scan the following bytes only while they are tabs ('\t')
pub fn scan_while_tabs<R: AsRef<[u8]>>(readable: &R, at: usize) -> usize {
    let mut scanned = at;

    loop {
        match readable.get_byte_at(scanned) {
            Some(b'\t') => {
                scanned += 1;
                continue;
            }
            _ => break,
        }
    }

    scanned - at
}

// /// Scan the following bytes only while they are white spaces (' ') and horizontal tabs ("\t")
// pub fn scan_while_spaces_and_tabs<R: AsRef<[u8]>>(readable: &mut R, at: usize) -> usize {
//     let mut scanned = at;

//     loop {
//         match readable.get_byte_at(scanned) {
//             Some(b' ') | Some(b'\t') => {
//                 scanned += 1;
//                 continue;
//             }
//             _ => break,
//         }
//     }

//     scanned - at
// }

// /// Scan the following bytes until the first colon (:) or a newline
// pub fn scan_until_colon_and_line_breakers<R: AsRef<[u8]>>(readable: &mut R, at: usize) -> usize {
//     let mut scanned = at;
//     loop {
//         match readable.get_byte_at(scanned) {
//             None | Some(b':') | Some(b'\n') | Some(b'\r') => break,
//             _ => scanned += 1,
//         };
//     }
//     scanned - at
// }

// /// Scan the following bytes until the first question (?) or a newline
// pub fn scan_until_question_and_line_breakers<R: AsRef<[u8]>>(readable: &mut R, at: usize) -> usize {
//     let mut scanned = at;
//     loop {
//         match readable.get_byte_at(scanned) {
//             None | Some(b'?') | Some(b'\n') | Some(b'\r') => break,
//             _ => scanned += 1,
//         };
//     }
//     scanned - at
// }

/// Scan one or two following bytes only if they are a space (' '), horizontal tab ("\t") or a newline
pub fn scan_one_spaces_and_line_breakers<R: AsRef<[u8]>>(readable: &R, at: usize) -> usize {
    match readable.get_byte_at(at) {
        Some(b' ') => 1,
        Some(b'\n') => 1,
        Some(b'\t') => 1,
        Some(b'\r') => {
            if let Some(b'\n') = readable.get_byte_at(at + 1) {
                2
            } else {
                1
            }
        }
        _ => 0,
    }
}

// /// Scan the following bytes only while they are a space (' '), horizontal tab ("\t") or a newline
// pub fn scan_while_spaces_and_line_breakers<R: AsRef<[u8]>>(readable: &mut R, at: usize) -> usize {
//     let mut scanned = at;

//     loop {
//         match readable.get_byte_at(scanned) {
//             Some(b' ') => scanned += 1,
//             Some(b'\n') => scanned += 1,
//             Some(b'\t') => scanned += 1,
//             Some(b'\r') => {
//                 scanned += if let Some(b'\n') = readable.get_byte_at(scanned + 1) {
//                     2
//                 } else {
//                     1
//                 }
//             }
//             _ => break,
//         };
//     }

//     scanned - at
// }

// /// Scan one newline sequence ("\n", "\r", "\r\n")
// pub fn scan_one_line_breaker<R: AsRef<[u8]>>(readable: &mut R, at: usize) -> usize {
//     match readable.get_byte_at(at) {
//         Some(b'\n') => 1,
//         Some(b'\r') => {
//             if let Some(b'\n') = readable.get_byte_at(at + 1) {
//                 2
//             } else {
//                 1
//             }
//         }
//         _ => 0,
//     }
// }

/// Scan the following bytes until meeting an alias stop sequence which are:
/// ' ', "\t", "\n", "\r", "{", "}", "[", "]",
/// ": ", ":\t", ":\n", ":\r", ":,", ", ",
/// ",\t", ",\n", ",\r", ",:"
pub fn scan_until_alias_stops<R: AsRef<[u8]>>(readable: &R) -> usize {
    let mut scanned = 0;

    loop {
        match readable.get_byte_at(scanned) {
            None => break,

            Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') | Some(b'{') | Some(b'}')
            | Some(b'[') | Some(b']') => break,

            Some(b':') => match readable.get_byte_at(scanned + 1) {
                Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') | Some(b',') => break,
                _ => (),
            },

            Some(b',') => match readable.get_byte_at(scanned + 1) {
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
pub fn scan_until_anchor_stops<R: AsRef<[u8]>>(readable: &R, at: usize) -> usize {
    let mut scanned = at;

    loop {
        match readable.get_byte_at(scanned) {
            None | Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') | Some(b'{')
            | Some(b'}') | Some(b'[') | Some(b']') => break,
            _ => scanned += 1,
        };
    }

    scanned - at
}

/// Scan the following bytes until meeting a raw stop sequence which are:
/// "\n", "\r", '{', '}', '[', ']', '#', ':', ','
pub fn scan_until_raw_stops<R: AsRef<[u8]>>(readable: &R) -> usize {
    let mut scanned = 0;

    loop {
        match readable.get_byte_at(scanned) {
            None | Some(b'\n') | Some(b'\r') | Some(b'{') | Some(b'}') | Some(b'[')
            | Some(b']') | Some(b'#') | Some(b':') | Some(b',') => break,
            _ => scanned += 1,
        };
    }

    scanned
}

/// Scan the following bytes until meeting a tag flow stop sequence which are:
/// ' ', "\t", "\n", "\r", '{', '}', '[', ']', ':', ','
pub fn scan_until_tag_flow_stops<R: AsRef<[u8]>>(readable: &R, at: usize) -> usize {
    let mut scanned = at;

    loop {
        match readable.get_byte_at(scanned) {
            None | Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') | Some(b'{')
            | Some(b'}') | Some(b'[') | Some(b']') | Some(b':') | Some(b',') => break,
            _ => scanned += 1,
        };
    }

    scanned - at
}

/// Reads until the following newline (exclusive)
pub fn scan_line<R: AsRef<[u8]>>(readable: &R) -> usize {
    scan_line_at(readable, 0)
}

/// Reads until the following newline (exclusive), starting at the position given via "at" parameter
pub fn scan_line_at<R: AsRef<[u8]>>(readable: &R, at: usize) -> usize {
    let mut scanned = at;
    loop {
        match readable.get_byte_at(scanned) {
            None | Some(b'\n') | Some(b'\r') => break,
            _ => scanned += 1,
        };
    }
    scanned - at
}

pub fn scan_while_newline<R: AsRef<[u8]>>(readable: &R, at: usize) -> usize {
    let mut scanned = at;

    loop {
        match readable.get_byte_at(scanned) {
            Some(b'\n') | Some(b'\r') => {
                scanned += 1;
            }
            _ => break,
        }
    }

    scanned - at
}
