extern crate skimmer;

use self::skimmer::symbol::{ CopySymbol };

use txt::encoding::Encoding;

pub mod utf8;

pub use self::utf8::get_charset_utf8;



#[derive (Clone)]
pub struct CharSet<Char, DoubleChar>
  where
    Char: CopySymbol,
    DoubleChar: CopySymbol
{
    pub line_feed: Char, // \n
    pub carriage_return: Char,  // \r

    pub commercial_at: Char, // @
    pub backslash: Char, // \

    pub greater_than: Char, // >
    pub less_than: Char, // <

    pub bracket_curly_left: Char, // {
    pub bracket_curly_right: Char, // }

    pub paren_left: Char, // (
    pub paren_right: Char, // )

    pub bracket_square_left: Char, // [
    pub bracket_square_right: Char, // ]

    pub ampersand: Char, // &
    pub asterisk: Char, // *

    pub colon: Char, // :
    pub comma: Char, // ,
    pub hyphen_minus: Char, // -
    pub full_stop: Char, // .
    pub equal: Char, // =
    pub exclamation: Char, // !
    pub grave_accent: Char, // `
    pub hashtag: Char, // #
    pub percent: Char, // %
    pub plus: Char, // +
    pub vertical_bar: Char, // |
    pub question: Char, // ?
    pub tilde: Char, // ~
    pub low_line: Char, // _

    pub quotation: Char, // "
    pub apostrophe: Char, // '

    pub semicolon: Char, // ;
    pub slash: Char, // /
    pub space: Char, //  
    pub tab_h: Char, // \t
    pub tab_v: Char, // \v
    pub form_feed: Char, // \f
    pub escape: Char, // \e

    pub nbspace: Char, // 0xA0
    // pub line_separator: Char, // 0x2028
    // pub paragraph_separator: Char, // 0x2029

    pub digit_0: Char, // 0
    pub digit_1: Char, // 1
    pub digit_2: Char, // 2
    pub digit_3: Char, // 3
    pub digit_4: Char, // 4
    pub digit_5: Char, // 5
    pub digit_6: Char, // 6
    pub digit_7: Char, // 7
    pub digit_8: Char, // 8
    pub digit_9: Char, // 9

    pub letter_a: Char, // a
    pub letter_b: Char, // b
    pub letter_c: Char, // c
    pub letter_d: Char, // d
    pub letter_e: Char, // e
    pub letter_f: Char, // f
    pub letter_g: Char, // g
    pub letter_h: Char, // h
    pub letter_i: Char, // i
    pub letter_j: Char, // j
    pub letter_k: Char, // k
    pub letter_l: Char, // l
    pub letter_m: Char, // m
    pub letter_n: Char, // n
    pub letter_o: Char, // o
    pub letter_p: Char, // p
    pub letter_q: Char, // q
    pub letter_r: Char, // r
    pub letter_s: Char, // s
    pub letter_t: Char, // t
    pub letter_u: Char, // u
    pub letter_v: Char, // v
    pub letter_w: Char, // w
    pub letter_x: Char, // x
    pub letter_y: Char, // y
    pub letter_z: Char, // z

    pub letter_t_a: Char, // A
    pub letter_t_b: Char, // B
    pub letter_t_c: Char, // C
    pub letter_t_d: Char, // D
    pub letter_t_e: Char, // E
    pub letter_t_f: Char, // F
    pub letter_t_g: Char, // G
    pub letter_t_h: Char, // H
    pub letter_t_i: Char, // I
    pub letter_t_j: Char, // J
    pub letter_t_k: Char, // K
    pub letter_t_l: Char, // L
    pub letter_t_m: Char, // M
    pub letter_t_n: Char, // N
    pub letter_t_o: Char, // O
    pub letter_t_p: Char, // P
    pub letter_t_q: Char, // Q
    pub letter_t_r: Char, // R
    pub letter_t_s: Char, // S
    pub letter_t_t: Char, // T
    pub letter_t_u: Char, // U
    pub letter_t_v: Char, // V
    pub letter_t_w: Char, // W
    pub letter_t_x: Char, // X
    pub letter_t_y: Char, // Y
    pub letter_t_z: Char, // Z

    pub null: Char, // 0x0
    pub bell: Char, // 0x7
    pub backspace: Char, // 0x8
    pub nextline: Char, // 0x85 == \N

    pub crlf: DoubleChar, // \r\n

    pub encoding: Encoding,

    pub longest_char: usize
}



impl<Char, DoubleChar> CharSet<Char, DoubleChar>
  where
    Char: CopySymbol,
    DoubleChar: CopySymbol
{
    pub fn extract_hex (&self, src: &[u8]) -> Option<(u8, usize)> { self.extract_hex_at (src, 0) }


    pub fn extract_hex_at (&self, src: &[u8], at: usize) -> Option<(u8, usize)> {
        // TODO: effective specialisations for UTF-8, UTF-16 and UTF-32

        self.extract_dec_at (src, at).or_else (|| {
                 if self.letter_a.contained_at (&src, at) { Some ( (10, self.letter_a.len ()) ) }
            else if self.letter_b.contained_at (&src, at) { Some ( (11, self.letter_b.len ()) ) }
            else if self.letter_c.contained_at (&src, at) { Some ( (12, self.letter_c.len ()) ) }
            else if self.letter_d.contained_at (&src, at) { Some ( (13, self.letter_d.len ()) ) }
            else if self.letter_e.contained_at (&src, at) { Some ( (14, self.letter_e.len ()) ) }
            else if self.letter_f.contained_at (&src, at) { Some ( (15, self.letter_f.len ()) ) }

            else if self.letter_t_a.contained_at (&src, at) { Some ( (10, self.letter_t_a.len ()) ) }
            else if self.letter_t_b.contained_at (&src, at) { Some ( (11, self.letter_t_b.len ()) ) }
            else if self.letter_t_c.contained_at (&src, at) { Some ( (12, self.letter_t_c.len ()) ) }
            else if self.letter_t_d.contained_at (&src, at) { Some ( (13, self.letter_t_d.len ()) ) }
            else if self.letter_t_e.contained_at (&src, at) { Some ( (14, self.letter_t_e.len ()) ) }
            else if self.letter_t_f.contained_at (&src, at) { Some ( (15, self.letter_t_f.len ()) ) }

            else { None }
        })
    }


    pub fn extract_dec (&self, src: &[u8]) -> Option<(u8, usize)> { self.extract_dec_at (src, 0) }


    pub fn extract_dec_at (&self, src: &[u8], at: usize) -> Option<(u8, usize)> {
        // TODO: effective specialisations for UTF-8, UTF-16 and UTF-32
             if self.digit_0.contained_at (&src, at) { Some ( (0, self.digit_0.len ()) ) }
        else if self.digit_1.contained_at (&src, at) { Some ( (1, self.digit_1.len ()) ) }
        else if self.digit_2.contained_at (&src, at) { Some ( (2, self.digit_2.len ()) ) }
        else if self.digit_3.contained_at (&src, at) { Some ( (3, self.digit_3.len ()) ) }
        else if self.digit_4.contained_at (&src, at) { Some ( (4, self.digit_4.len ()) ) }
        else if self.digit_5.contained_at (&src, at) { Some ( (5, self.digit_5.len ()) ) }
        else if self.digit_6.contained_at (&src, at) { Some ( (6, self.digit_6.len ()) ) }
        else if self.digit_7.contained_at (&src, at) { Some ( (7, self.digit_7.len ()) ) }
        else if self.digit_8.contained_at (&src, at) { Some ( (8, self.digit_8.len ()) ) }
        else if self.digit_9.contained_at (&src, at) { Some ( (9, self.digit_9.len ()) ) }

        else { None }
    }
}





#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    #[test]
    fn test_charset_extract_hex () {
        let cset = get_charset_utf8 ();

        assert_eq! (Some ( (0, 1) ), cset.extract_hex (&[b'0']));
        assert_eq! (Some ( (1, 1) ), cset.extract_hex (&[b'1']));
        assert_eq! (Some ( (2, 1) ), cset.extract_hex (&[b'2']));
        assert_eq! (Some ( (3, 1) ), cset.extract_hex (&[b'3']));
        assert_eq! (Some ( (4, 1) ), cset.extract_hex (&[b'4']));
        assert_eq! (Some ( (5, 1) ), cset.extract_hex (&[b'5']));
        assert_eq! (Some ( (6, 1) ), cset.extract_hex (&[b'6']));
        assert_eq! (Some ( (7, 1) ), cset.extract_hex (&[b'7']));
        assert_eq! (Some ( (8, 1) ), cset.extract_hex (&[b'8']));
        assert_eq! (Some ( (9, 1) ), cset.extract_hex (&[b'9']));
        assert_eq! (Some ( (10, 1) ), cset.extract_hex (&[b'a']));
        assert_eq! (Some ( (11, 1) ), cset.extract_hex (&[b'b']));
        assert_eq! (Some ( (12, 1) ), cset.extract_hex (&[b'c']));
        assert_eq! (Some ( (13, 1) ), cset.extract_hex (&[b'd']));
        assert_eq! (Some ( (14, 1) ), cset.extract_hex (&[b'e']));
        assert_eq! (Some ( (15, 1) ), cset.extract_hex (&[b'f']));
        assert_eq! (Some ( (10, 1) ), cset.extract_hex (&[b'A']));
        assert_eq! (Some ( (11, 1) ), cset.extract_hex (&[b'B']));
        assert_eq! (Some ( (12, 1) ), cset.extract_hex (&[b'C']));
        assert_eq! (Some ( (13, 1) ), cset.extract_hex (&[b'D']));
        assert_eq! (Some ( (14, 1) ), cset.extract_hex (&[b'E']));
        assert_eq! (Some ( (15, 1) ), cset.extract_hex (&[b'F']));
        assert_eq! (None, cset.extract_hex (&[b'z']));
    }


    #[test]
    fn test_charset_extract_dec () {
        let cset = get_charset_utf8 ();

        assert_eq! (Some ( (0, 1) ), cset.extract_dec (&[b'0']));
        assert_eq! (Some ( (1, 1) ), cset.extract_dec (&[b'1']));
        assert_eq! (Some ( (2, 1) ), cset.extract_dec (&[b'2']));
        assert_eq! (Some ( (3, 1) ), cset.extract_dec (&[b'3']));
        assert_eq! (Some ( (4, 1) ), cset.extract_dec (&[b'4']));
        assert_eq! (Some ( (5, 1) ), cset.extract_dec (&[b'5']));
        assert_eq! (Some ( (6, 1) ), cset.extract_dec (&[b'6']));
        assert_eq! (Some ( (7, 1) ), cset.extract_dec (&[b'7']));
        assert_eq! (Some ( (8, 1) ), cset.extract_dec (&[b'8']));
        assert_eq! (Some ( (9, 1) ), cset.extract_dec (&[b'9']));
        assert_eq! (None, cset.extract_dec (&[b'a']));
    }
}
