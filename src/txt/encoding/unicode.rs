pub trait Unicode {
    fn char_max_bytes_len (self) -> u8;

    fn check_bom (self, bom: &[u8]) -> bool;

    unsafe fn to_unicode_ptr (self, ptr: *const u8, len: usize) -> (u32, u8);

    fn to_unicode (self, stream: &[u8]) -> (u32, u8);

    fn from_unicode (self, point: u32) -> [u8; 5];

    fn check_is_dec_num (self, stream: &[u8]) -> bool;

    fn check_is_flo_num (self, stream: &[u8]) -> bool;

    fn extract_bin_digit (self, stream: &[u8]) -> Option<(u8, u8)>;

    fn extract_dec_digit (self, stream: &[u8]) -> Option<(u8, u8)>;

    fn extract_oct_digit (self, stream: &[u8]) -> Option<(u8, u8)>;

    fn extract_hex_digit (self, stream: &[u8]) -> Option<(u8, u8)>;

    fn str_to_bytes<'a> (self, string: &'a str) -> Result<&'a [u8], Vec<u8>>;

    fn string_to_bytes (self, string: String) -> Vec<u8>;

    fn bytes_to_string (self, bytes: &[u8]) -> Result<String, ()>;

    fn bytes_to_string_times (self, bytes: &[u8], times: usize) -> Result<String, ()>;
}
