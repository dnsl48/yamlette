extern crate skimmer;


use self::skimmer::symbol::{ Char, Word, Symbol };


use txt::encoding::Encoding;
use txt::encoding::UTF8;




#[derive (Clone)]
pub struct CharSet {
    pub encoding: Encoding,

    pub crlf: Word, // \r\n

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
    pub line_separator: Char, // 0x2028
    pub paragraph_separator: Char, // 0x2029

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
}



impl CharSet {
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



pub fn get_charset_utf8 () -> CharSet {
    CharSet {
        encoding: Encoding::UTF8 (UTF8),

        crlf: Word::combine (&[&Char::new (&[0xD]), &Char::new (&[0xA])]),

        commercial_at: Char::new (&[b'@']), // @

        backslash: Char::new (&[b'\\']), // \

        greater_than: Char::new (&[b'>']),
        less_than: Char::new (&[b'<']),

        bracket_curly_right: Char::new (&[b'}']),
        bracket_curly_left: Char::new (&[b'{']),

        paren_right: Char::new (&[b')']),
        paren_left: Char::new (&[b'(']),

        bracket_square_right: Char::new (&[b']']),
        bracket_square_left: Char::new (&[b'[']),

        ampersand: Char::new (&[b'&']),
        asterisk: Char::new (&[b'*']),
        colon: Char::new (&[b':']),
        comma: Char::new (&[b',']),
        hyphen_minus: Char::new (&[b'-']),
        full_stop: Char::new (&[b'.']),
        equal: Char::new (&[b'=']),
        exclamation: Char::new (&[b'!']),
        grave_accent: Char::new (&[b'`']),
        hashtag: Char::new (&[b'#']),
        line_feed: Char::new (&[0xA]), // \n
        percent: Char::new (&[b'%']),
        plus: Char::new (&[b'+']),
        vertical_bar: Char::new (&[b'|']),
        question: Char::new (&[b'?']),
        tilde: Char::new (&[b'~']),
        low_line: Char::new (&[b'_']),

        quotation: Char::new (&[b'"']),
        apostrophe: Char::new (&[b'\'']),

        carriage_return: Char::new (&[0xD]), // \r
        semicolon: Char::new (&[b';']),
        slash: Char::new (&[b'/']),
        space: Char::new (&[b' ']),
        tab_h: Char::new (&[0x9]), // \t
        tab_v: Char::new (&[0xB]), // \v
        form_feed: Char::new (&[0xC]), // \f
        escape: Char::new (&[0x1B]), // \e
        nbspace: Char::new (&[0xA0]),
        line_separator: Char::new (&[0xE2, 0x80, 0xA8]), // \x2028
        paragraph_separator: Char::new (&[0xE2, 0x80, 0xA9]), // \x2029

        digit_0: Char::new (&[b'0']),
        digit_1: Char::new (&[b'1']),
        digit_2: Char::new (&[b'2']),
        digit_3: Char::new (&[b'3']),
        digit_4: Char::new (&[b'4']),
        digit_5: Char::new (&[b'5']),
        digit_6: Char::new (&[b'6']),
        digit_7: Char::new (&[b'7']),
        digit_8: Char::new (&[b'8']),
        digit_9: Char::new (&[b'9']),

        letter_a: Char::new (&[b'a']),
        letter_b: Char::new (&[b'b']),
        letter_c: Char::new (&[b'c']),
        letter_d: Char::new (&[b'd']),
        letter_e: Char::new (&[b'e']),
        letter_f: Char::new (&[b'f']),
        letter_g: Char::new (&[b'g']),
        letter_h: Char::new (&[b'h']),
        letter_i: Char::new (&[b'i']),
        letter_j: Char::new (&[b'j']),
        letter_k: Char::new (&[b'k']),
        letter_l: Char::new (&[b'l']),
        letter_m: Char::new (&[b'm']),
        letter_n: Char::new (&[b'n']),
        letter_o: Char::new (&[b'o']),
        letter_p: Char::new (&[b'p']),
        letter_q: Char::new (&[b'q']),
        letter_r: Char::new (&[b'r']),
        letter_s: Char::new (&[b's']),
        letter_t: Char::new (&[b't']),
        letter_u: Char::new (&[b'u']),
        letter_v: Char::new (&[b'v']),
        letter_w: Char::new (&[b'w']),
        letter_x: Char::new (&[b'x']),
        letter_y: Char::new (&[b'y']),
        letter_z: Char::new (&[b'z']),

        letter_t_a: Char::new (&[b'A']),
        letter_t_b: Char::new (&[b'B']),
        letter_t_c: Char::new (&[b'C']),
        letter_t_d: Char::new (&[b'D']),
        letter_t_e: Char::new (&[b'E']),
        letter_t_f: Char::new (&[b'F']),
        letter_t_g: Char::new (&[b'G']),
        letter_t_h: Char::new (&[b'H']),
        letter_t_i: Char::new (&[b'I']),
        letter_t_j: Char::new (&[b'J']),
        letter_t_k: Char::new (&[b'K']),
        letter_t_l: Char::new (&[b'L']),
        letter_t_m: Char::new (&[b'M']),
        letter_t_n: Char::new (&[b'N']),
        letter_t_o: Char::new (&[b'O']),
        letter_t_p: Char::new (&[b'P']),
        letter_t_q: Char::new (&[b'Q']),
        letter_t_r: Char::new (&[b'R']),
        letter_t_s: Char::new (&[b'S']),
        letter_t_t: Char::new (&[b'T']),
        letter_t_u: Char::new (&[b'U']),
        letter_t_v: Char::new (&[b'V']),
        letter_t_w: Char::new (&[b'W']),
        letter_t_x: Char::new (&[b'X']),
        letter_t_y: Char::new (&[b'Y']),
        letter_t_z: Char::new (&[b'Z']),

        null: Char::new (&[0x0]),
        bell: Char::new (&[0x7]),
        backspace: Char::new (&[0x8]),
        nextline: Char::new (&[0x85]), // \N
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
