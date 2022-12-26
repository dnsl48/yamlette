use crate::txt::encoding::Unicode;

#[derive(Copy, Clone)]
pub struct UTF8;

impl Unicode for UTF8 {
    #[inline(always)]
    fn char_max_bytes_len(self) -> u8 {
        4
    }

    fn check_bom(self, bom: &[u8]) -> bool {
        bom == &[0xEF, 0xBB, 0xBF]
    }

    fn check_is_flo_num(self, stream: &[u8]) -> bool {
        if stream.len() == 0 {
            return false;
        }
        let n = stream[0];
        if n > 47 && n < 58 {
            return true;
        }
        // [0-9]
        else if (n == 43 || n == 45 || n == 46) && stream.len() > 1 {
            // [+-.]
            true
        } else {
            false
        }
    }

    fn check_is_dec_num(self, stream: &[u8]) -> bool {
        if stream.len() == 0 {
            return false;
        }
        let n = stream[0];
        (n > 47 && n < 58) || ((n == 43 || n == 45) && stream.len() > 1) // [0-9] || [+-]
    }

    fn extract_bin_digit(self, stream: &[u8]) -> Option<(u8, u8)> {
        if stream.len() == 0 {
            None
        } else {
            let n = stream[0];
            if n > 47 && n < 50 {
                Some((n - 48, 1))
            } else {
                None
            }
        }
    }

    fn extract_dec_digit(self, stream: &[u8]) -> Option<(u8, u8)> {
        if stream.len() == 0 {
            None
        } else {
            let n = stream[0];
            if n > 47 && n < 58 {
                Some((n - 48, 1))
            } else {
                None
            }
        }
    }

    fn extract_oct_digit(self, stream: &[u8]) -> Option<(u8, u8)> {
        if stream.len() == 0 {
            None
        } else {
            let n = stream[0];
            if n > 47 && n < 56 {
                Some((n - 48, 1))
            } else {
                None
            }
        }
    }

    fn extract_hex_digit(self, stream: &[u8]) -> Option<(u8, u8)> {
        if stream.len() == 0 {
            None
        } else {
            let n = stream[0];
            if n > 47 && n < 58 {
                Some((n - 48, 1))
            } else if n > 64 && n < 71 {
                Some((n - 55, 1))
            } else if n > 96 && n < 103 {
                Some((n - 87, 1))
            } else {
                None
            }
        }
    }

    unsafe fn to_unicode_ptr(self, mut ptr: *const u8, len: usize) -> (u32, u8) {
        let (code, len) = if len > 0 {
            if *ptr & 0x80 == 0 {
                ((*ptr as u8) as u32, 1)
            } else if len > 1 {
                let _1 = *ptr as u8;
                ptr = ptr.offset(1);

                if _1 & 0xC0 == 0xC0 && _1 & 0x20 == 0 && *ptr & 0x80 == 0x80 && *ptr & 0x40 == 0 {
                    (((((_1 as u32) ^ 0xC0) << 6) | ((*ptr as u32) ^ 0x80)), 2)
                } else if len > 2 {
                    let _2 = *ptr as u8;
                    ptr = ptr.offset(1);

                    if _1 & 0xE0 == 0xE0
                        && _1 & 0x10 == 0
                        && _2 & 0x80 == 0x80
                        && _2 & 0x40 == 0
                        && *ptr & 0x80 == 0x80
                        && *ptr & 0x40 == 0
                    {
                        (
                            (((_1 as u32) ^ 0xE0) << 12)
                                | (((_2 as u32) ^ 0x80) << 6)
                                | ((*ptr as u32) ^ 0x80),
                            3,
                        )
                    } else if len > 3 {
                        let _3 = *ptr as u8;
                        ptr = ptr.offset(1);

                        if _1 & 0xF0 == 0xF0
                            && _1 & 0x08 == 0
                            && _2 & 0x80 == 0x80
                            && _2 & 0x40 == 0
                            && _3 & 0x80 == 0x80
                            && _3 & 0x40 == 0
                            && *ptr & 0x80 == 0x80
                            && *ptr & 0x40 == 0
                        {
                            (
                                (((_1 as u32) ^ 0xF0) << 18)
                                    | (((_2 as u32) ^ 0x80) << 12)
                                    | (((_3 as u32) ^ 0x80) << 6)
                                    | ((*ptr as u32) ^ 0x80),
                                4,
                            )
                        } else {
                            (0xFFFD, 1)
                        }
                    } else {
                        (0xFFFD, 1)
                    }
                } else {
                    (0xFFFD, 1)
                }
            } else {
                (0xFFFD, 1)
            }
        } else {
            (0xFFFD, 1)
        };

        if code > 0x10FFFF || (code >= 0xD800 && code <= 0xE000) {
            (0xFFFD, len)
        } else {
            (code, len)
        }
    }

    fn to_unicode(self, stream: &[u8]) -> (u32, u8) {
        let slen = stream.len();

        let (code, len) = if slen > 0 && stream[0] & 0x80 == 0 {
            return (stream[0] as u32, 1);
        } else if slen > 1
            && stream[0] & 0xC0 == 0xC0
            && stream[0] & 0x20 == 0
            && stream[1] & 0x80 == 0x80
            && stream[1] & 0x40 == 0
        {
            return (
                (((stream[0] as u32) ^ 0xC0) << 6) | ((stream[1] as u32) ^ 0x80),
                2,
            );
        } else if slen > 2
            && stream[0] & 0xE0 == 0xE0
            && stream[0] & 0x10 == 0
            && stream[1] & 0x80 == 0x80
            && stream[1] & 0x40 == 0
            && stream[2] & 0x80 == 0x80
            && stream[2] & 0x40 == 0
        {
            (
                (((stream[0] as u32) ^ 0xE0) << 12)
                    | (((stream[1] as u32) ^ 0x80) << 6)
                    | ((stream[2] as u32) ^ 0x80),
                3,
            )
        } else if slen > 3
            && stream[0] & 0xF0 == 0xF0
            && stream[0] & 0x08 == 0
            && stream[1] & 0x80 == 0x80
            && stream[1] & 0x40 == 0
            && stream[2] & 0x80 == 0x80
            && stream[2] & 0x40 == 0
            && stream[3] & 0x80 == 0x80
            && stream[3] & 0x40 == 0
        {
            (
                (((stream[0] as u32) ^ 0xF0) << 18)
                    | (((stream[1] as u32) ^ 0x80) << 12)
                    | (((stream[2] as u32) ^ 0x80) << 6)
                    | ((stream[3] as u32) ^ 0x80),
                4,
            )
        } else {
            return (0xFFFD, 1);
        };

        if code > 0x10FFFF || (code >= 0xD800 && code <= 0xE000) {
            (0xFFFD, len)
        } else {
            (code, len)
        }
    }

    fn from_unicode(self, code: u32) -> [u8; 5] {
        match code {
            0x0000..=0x007F => [code as u8, 0, 0, 0, 1],

            0x0080..=0x07FF => [
                0x00C0 | ((code >> 6) as u8),
                0x0080 | ((code ^ (code & 0x07C0)) as u8),
                0,
                0,
                2,
            ],

            0x0800..=0xD7FF | 0xE000..=0xFFFF => [
                0x00E0 | ((code >> 12) as u8),
                0x0080 | (((code ^ (code & 0xF000)) >> 6) as u8),
                0x0080 | ((code ^ (code & 0xFFC0)) as u8),
                0,
                3,
            ],

            0x10000..=0x10FFFF => [
                0x00F0 | ((code >> 18) as u8),
                0x0080 | (((code ^ (code & 0x1C0000)) >> 12) as u8),
                0x0080 | (((code ^ (code & 0x1FF000)) >> 6) as u8),
                0x0080 | ((code ^ (code & 0x1FFFC0)) as u8),
                4,
            ],

            _ => [0, 0, 0, 0, 0],
        }
    }

    fn str_to_bytes<'a>(self, string: &'a str) -> Result<&'a [u8], Vec<u8>> {
        Ok(string.as_bytes())
    }

    #[inline(always)]
    fn string_to_bytes(self, string: String) -> Vec<u8> {
        string.into_bytes()
    }

    fn bytes_to_string(self, bytes: &[u8]) -> Result<String, ()> {
        Ok(String::from(String::from_utf8_lossy(bytes)))
    }

    fn bytes_to_string_times(self, bytes: &[u8], times: usize) -> Result<String, ()> {
        let mut result = Vec::with_capacity(bytes.len() * times);
        for _ in 0..times {
            result.extend(bytes);
        }

        if let Ok(string) = String::from_utf8(result) {
            Ok(string)
        } else {
            Err(())
        }
    }
}

#[cfg(all(test, not(feature = "dev")))]
mod tests {
    use super::UTF8;
    use crate::txt::encoding::unicode::Unicode;

    #[test]
    fn to_unicode() {
        let enc = UTF8;

        let src: [u8; 23] = [
            0xd1, 0x82, 0xd0, 0xb5, 0xd1, 0x81, 0xd1, 0x82, 0x20, 0xd1, 0x8e, 0xd0, 0xbd, 0xd0,
            0xb8, 0xd0, 0xba, 0xd0, 0xbe, 0xd0, 0xb4, 0xd0, 0xb0,
        ];

        assert_eq!((0x442, 2), enc.to_unicode(&src));
        assert_eq!((0x435, 2), enc.to_unicode(&src[2..]));
        assert_eq!((0x441, 2), enc.to_unicode(&src[4..]));
        assert_eq!((0x442, 2), enc.to_unicode(&src[6..]));
        assert_eq!((0x20, 1), enc.to_unicode(&src[8..]));
        assert_eq!((0x44e, 2), enc.to_unicode(&src[9..]));
        assert_eq!((0x43d, 2), enc.to_unicode(&src[11..]));
        assert_eq!((0x438, 2), enc.to_unicode(&src[13..]));
        assert_eq!((0x43a, 2), enc.to_unicode(&src[15..]));
        assert_eq!((0x43e, 2), enc.to_unicode(&src[17..]));
        assert_eq!((0x434, 2), enc.to_unicode(&src[19..]));
        assert_eq!((0x430, 2), enc.to_unicode(&src[21..]));

        let src = [
            0xe7, 0xa7, 0x81, 0xe3, 0x81, 0xaf, 0xe7, 0xa7, 0x81, 0xe3, 0x81, 0x8c, 0xe6, 0x9b,
            0xb8, 0xe3, 0x81, 0x84, 0xe3, 0x81, 0xa6, 0xe3, 0x81, 0x84, 0xe3, 0x82, 0x8b, 0xe3,
            0x81, 0xae, 0xe3, 0x81, 0x8b, 0xe5, 0x88, 0x86, 0xe3, 0x81, 0x8b, 0xe3, 0x82, 0x8a,
            0xe3, 0x81, 0xbe, 0xe3, 0x81, 0x9b, 0xe3, 0x82, 0x93,
        ];

        assert_eq!((0x79c1, 3), enc.to_unicode(&src));
        assert_eq!((0x306f, 3), enc.to_unicode(&src[3..]));
        assert_eq!((0x79c1, 3), enc.to_unicode(&src[6..]));
        assert_eq!((0x304c, 3), enc.to_unicode(&src[9..]));
        assert_eq!((0x66f8, 3), enc.to_unicode(&src[12..]));
        assert_eq!((0x3044, 3), enc.to_unicode(&src[15..]));
        assert_eq!((0x3066, 3), enc.to_unicode(&src[18..]));
        assert_eq!((0x3044, 3), enc.to_unicode(&src[21..]));
        assert_eq!((0x308b, 3), enc.to_unicode(&src[24..]));
        assert_eq!((0x306e, 3), enc.to_unicode(&src[27..]));
        assert_eq!((0x304b, 3), enc.to_unicode(&src[30..]));
        assert_eq!((0x5206, 3), enc.to_unicode(&src[33..]));
        assert_eq!((0x304b, 3), enc.to_unicode(&src[36..]));
        assert_eq!((0x308a, 3), enc.to_unicode(&src[39..]));
        assert_eq!((0x307e, 3), enc.to_unicode(&src[42..]));
        assert_eq!((0x305b, 3), enc.to_unicode(&src[45..]));
        assert_eq!((0x3093, 3), enc.to_unicode(&src[48..]));

        let src = [
            0xef, 0xa3, 0x96, 0xef, 0xa3, 0x94, 0xef, 0xa3, 0x95, 0xef, 0xa3, 0x99, 0xef, 0xa3,
            0xa5, 0xef, 0xa3, 0xa9, 0xef, 0xa3, 0x9a, 0xef, 0xa3, 0x94, 0xef, 0xa3, 0x96, 0x20,
            0xef, 0xa3, 0xa0, 0xef, 0xa3, 0x90, 0xef, 0xa3, 0xa0, 0x20, 0xef, 0xa3, 0x98, 0xef,
            0xa3, 0x90, 0xef, 0xa3, 0x98, 0xef, 0xa3, 0xa6, 0xef, 0xa3, 0x90, 0xef, 0xa3, 0x9a,
            0xef, 0xa3, 0xbe,
        ];

        assert_eq!((0xf8d6, 3), enc.to_unicode(&src));
        assert_eq!((0xf8d4, 3), enc.to_unicode(&src[3..]));
        assert_eq!((0xf8d5, 3), enc.to_unicode(&src[6..]));
        assert_eq!((0xf8d9, 3), enc.to_unicode(&src[9..]));
        assert_eq!((0xf8e5, 3), enc.to_unicode(&src[12..]));
        assert_eq!((0xf8e9, 3), enc.to_unicode(&src[15..]));
        assert_eq!((0xf8da, 3), enc.to_unicode(&src[18..]));
        assert_eq!((0xf8d4, 3), enc.to_unicode(&src[21..]));
        assert_eq!((0xf8d6, 3), enc.to_unicode(&src[24..]));
        assert_eq!((0x20, 1), enc.to_unicode(&src[27..]));
        assert_eq!((0xf8e0, 3), enc.to_unicode(&src[28..]));
        assert_eq!((0xf8d0, 3), enc.to_unicode(&src[31..]));
        assert_eq!((0xf8e0, 3), enc.to_unicode(&src[34..]));
        assert_eq!((0x20, 1), enc.to_unicode(&src[37..]));
        assert_eq!((0xf8d8, 3), enc.to_unicode(&src[38..]));
        assert_eq!((0xf8d0, 3), enc.to_unicode(&src[41..]));
        assert_eq!((0xf8d8, 3), enc.to_unicode(&src[44..]));
        assert_eq!((0xf8e6, 3), enc.to_unicode(&src[47..]));
        assert_eq!((0xf8d0, 3), enc.to_unicode(&src[50..]));
        assert_eq!((0xf8da, 3), enc.to_unicode(&src[53..]));
        assert_eq!((0xf8fe, 3), enc.to_unicode(&src[56..]));

        let src = [
            0xf0, 0x9f, 0x98, 0x81, 0xf0, 0x9f, 0x98, 0x82, 0xf0, 0x9f, 0x98, 0x83, 0xf0, 0x9f,
            0x98, 0x84,
        ];

        assert_eq!((0x1f601, 4), enc.to_unicode(&src));
        assert_eq!((0x1f602, 4), enc.to_unicode(&src[4..]));
        assert_eq!((0x1f603, 4), enc.to_unicode(&src[8..]));
        assert_eq!((0x1f604, 4), enc.to_unicode(&src[12..]));
    }

    #[test]
    fn from_unicode() {
        let enc = UTF8;

        assert_eq!([0xF0, 0x9F, 0x98, 0x93, 4], enc.from_unicode(0x1f613));

        assert_eq!([0xEF, 0xA3, 0x90, 0, 3], enc.from_unicode(0xf8d0));

        assert_eq!([0xD1, 0x91, 0, 0, 2], enc.from_unicode(0x451));

        assert_eq!([0x20, 0, 0, 0, 1], enc.from_unicode(0x20));
    }
}
