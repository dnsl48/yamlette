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
    fn char_max_bytes_len (&self) -> u8 {
        match *self {
            Encoding::UTF8 (ref e) => e.char_max_bytes_len ()
        }
    }


    fn check_bom (&self, bom: &[u8]) -> bool {
        match *self {
            Encoding::UTF8 (ref e) => e.check_bom (bom)
        }
    }


    unsafe fn to_unicode_ptr (&self, ptr: *const u8, len: usize) -> (u32, u8) {
        match *self {
            Encoding::UTF8 (ref e) => e.to_unicode_ptr (ptr, len)
        }
    }


    fn to_unicode (&self, stream: &[u8]) -> (u32, u8) {
        match *self {
            Encoding::UTF8 (ref e) => e.to_unicode (stream)
        }
    }


    fn from_unicode (&self, point: u32) -> [u8; 5] {
        match *self {
            Encoding::UTF8 (ref e) => e.from_unicode (point)
        }
    }


    fn str_to_bytes<'a, 'b> (&'a self, string: &'b str) -> Result<&'b [u8], Vec<u8>> {
        match *self {
            Encoding::UTF8 (ref e) => e.str_to_bytes (string)
        }
    }


    fn string_to_bytes (&self, string: String) -> Vec<u8> {
        match *self {
            Encoding::UTF8 (ref e) => e.string_to_bytes (string)
        }
    }
}



