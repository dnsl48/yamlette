// use txt::encoding::utf8::UTF8;


pub trait Unicode {
    fn char_max_bytes_len (self) -> u8;

    fn check_bom (self, bom: &[u8]) -> bool;

    unsafe fn to_unicode_ptr (self, ptr: *const u8, len: usize) -> (u32, u8);

    fn to_unicode (self, stream: &[u8]) -> (u32, u8);

    fn from_unicode (self, point: u32) -> [u8; 5];

    fn check_is_dec_num (self, stream: &[u8]) -> bool;

    fn check_is_flo_num (self, stream: &[u8]) -> bool;

    fn extract_bin_digit (self, &[u8]) -> Option<(u8, u8)>;

    fn extract_dec_digit (self, &[u8]) -> Option<(u8, u8)>;

    fn extract_oct_digit (self, &[u8]) -> Option<(u8, u8)>;

    fn extract_hex_digit (self, &[u8]) -> Option<(u8, u8)>;


    fn str_to_bytes<'a> (self, string: &'a str) -> Result<&'a [u8], Vec<u8>>; /* {
        let utf8 = UTF8;

        let bytes = string.as_bytes ();
        let capacity = bytes.len () * self.char_max_bytes_len () as usize;
        let mut result: Vec<u8> = Vec::with_capacity (capacity);

        let mut slen: usize = 0;
        let mut sptr = bytes.as_ptr ();

        let mut rlen: usize = 0;
        let mut rptr = result.as_mut_ptr ();

        loop {
            if slen >= bytes.len () { break; }

            let (code, len) = unsafe { utf8.to_unicode_ptr (sptr, bytes.len () - slen) };

            if len == 0 { break }

            slen += len as usize;
            sptr = unsafe { sptr.offset (len as isize) };

            let bts = self.from_unicode (code);

            if bts[4] == 0 { continue; }

            rlen += bts[4] as usize;
            if rlen >= capacity { unreachable! () /* overflow */ }

            unsafe { result.set_len (rlen) };
            for i in 0 .. bts[4] as usize {
                unsafe {
                    *rptr = bts[i];
                    rptr = rptr.offset (1);
                }
            }
        }

        Err (result)
    } */


    fn string_to_bytes (self, string: String) -> Vec<u8>; /* {
        match self.str_to_bytes (string.as_ref ()) {
            Ok (_) => string.into_bytes (),
            Err (vec) => vec
        }
    } */


    fn bytes_to_string (self, bytes: &[u8]) -> Result<String, ()>; /* { self.bytes_to_string_times (bytes, 1) } */

    fn bytes_to_string_times (self, bytes: &[u8], times: usize) -> Result<String, ()>; /* {
        let utf8 = UTF8;

        let capacity = bytes.len () * self.char_max_bytes_len () as usize;
        let mut result: Vec<u8> = Vec::with_capacity (capacity * times);

        let mut rlen: usize = 0;
        let mut rptr = result.as_mut_ptr ();

        for _ in 0 .. times {
            let mut sptr = bytes.as_ptr ();
            let mut slen: usize = 0;

            loop {
                if slen >= bytes.len () { break; }

                let (code, len) = unsafe { self.to_unicode_ptr (sptr, bytes.len () - slen) };

                if len == 0 { return Err ( () ) }

                slen += len as usize;
                sptr = unsafe { sptr.offset (len as isize) };

                let bts = utf8.from_unicode (code);

                if bts[4] == 0 { continue; }

                rlen += bts[4] as usize;
                if rlen >= capacity { unreachable! () /* overflow */ }

                unsafe { result.set_len (rlen) };
                for i in 0 .. bts[4] as usize {
                    unsafe {
                        *rptr = bts[i];
                        rptr = rptr.offset (1);
                    }
                }
            }
        }

        let string = unsafe { String::from_utf8_unchecked (result) };

        Ok (string)
    } */
}
