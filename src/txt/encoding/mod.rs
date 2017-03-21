pub mod unicode;
pub mod utf8;



pub use self::unicode::Unicode;
pub use self::utf8::UTF8;


use std::default::Default;



#[derive (Copy, Clone)]
pub enum Encoding {
    UTF8 (self::utf8::UTF8)
}



impl Default for Encoding {
    fn default () -> Encoding { Encoding::UTF8 (UTF8) }
}



impl Unicode for Encoding {
    fn char_max_bytes_len (self) -> u8 {
        match self {
            Encoding::UTF8 (e) => e.char_max_bytes_len ()
        }
    }


    fn check_is_flo_num (self, stream: &[u8]) -> bool {
        match self {
            Encoding::UTF8 (e) => e.check_is_flo_num (stream)
        }
    }


    fn check_is_dec_num (self, stream: &[u8]) -> bool {
        match self {
            Encoding::UTF8 (e) => e.check_is_dec_num (stream)
        }
    }


    fn extract_bin_digit (self, stream: &[u8]) -> Option<(u8, u8)> {
        match self {
            Encoding::UTF8 (e) => e.extract_bin_digit (stream)
        }
    }


    fn extract_dec_digit (self, stream: &[u8]) -> Option<(u8, u8)> {
        match self {
            Encoding::UTF8 (e) => e.extract_dec_digit (stream)
        }
    }


    fn extract_oct_digit (self, stream: &[u8]) -> Option<(u8, u8)> {
        match self {
            Encoding::UTF8 (e) => e.extract_oct_digit (stream)
        }
    }


    fn extract_hex_digit (self, stream: &[u8]) -> Option<(u8, u8)> {
        match self {
            Encoding::UTF8 (e) => e.extract_hex_digit (stream)
        }
    }


    fn check_bom (self, bom: &[u8]) -> bool {
        match self {
            Encoding::UTF8 (e) => e.check_bom (bom)
        }
    }


    unsafe fn to_unicode_ptr (self, ptr: *const u8, len: usize) -> (u32, u8) {
        match self {
            Encoding::UTF8 (e) => e.to_unicode_ptr (ptr, len)
        }
    }


    fn to_unicode (self, stream: &[u8]) -> (u32, u8) {
        match self {
            Encoding::UTF8 (e) => e.to_unicode (stream)
        }
    }


    fn from_unicode (self, point: u32) -> [u8; 5] {
        match self {
            Encoding::UTF8 (e) => e.from_unicode (point)
        }
    }


    fn str_to_bytes<'a> (self, string: &'a str) -> Result<&'a [u8], Vec<u8>> {
        match self {
            Encoding::UTF8 (e) => e.str_to_bytes (string)
        }
    }


    fn string_to_bytes (self, string: String) -> Vec<u8> {
        match self {
            Encoding::UTF8 (e) => e.string_to_bytes (string)
        }
    }


    fn bytes_to_string (self, bytes: &[u8]) -> Result<String, ()> {
        match self {
            Encoding::UTF8 (e) => e.bytes_to_string (bytes)
        }
    }


    fn bytes_to_string_times (self, bytes: &[u8], times: usize) -> Result<String, ()> {
        match self {
            Encoding::UTF8 (e) => e.bytes_to_string_times (bytes, times)
        }
    }
}
