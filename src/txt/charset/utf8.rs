extern crate skimmer;

pub use self::skimmer::symbol::{ Char1, Char2 };

use txt::charset::CharSet;
use txt::encoding::{ Encoding, UTF8 };



pub fn get_charset_utf8 () -> CharSet<Char1, Char2> {
    CharSet {
        encoding: Encoding::UTF8 (UTF8),

        crlf: Char2::new (&[0xD,0xA]),

        commercial_at: Char1::new (b'@'), // @

        backslash: Char1::new (b'\\'), // \

        greater_than: Char1::new (b'>'),
        less_than: Char1::new (b'<'),

        bracket_curly_right: Char1::new (b'}'),
        bracket_curly_left: Char1::new (b'{'),

        paren_right: Char1::new (b')'),
        paren_left: Char1::new (b'('),

        bracket_square_right: Char1::new (b']'),
        bracket_square_left: Char1::new (b'['),

        ampersand: Char1::new (b'&'),
        asterisk: Char1::new (b'*'),
        colon: Char1::new (b':'),
        comma: Char1::new (b','),
        hyphen_minus: Char1::new (b'-'),
        full_stop: Char1::new (b'.'),
        equal: Char1::new (b'='),
        exclamation: Char1::new (b'!'),
        grave_accent: Char1::new (b'`'),
        hashtag: Char1::new (b'#'),
        line_feed: Char1::new (0xA), // \n
        percent: Char1::new (b'%'),
        plus: Char1::new (b'+'),
        vertical_bar: Char1::new (b'|'),
        question: Char1::new (b'?'),
        tilde: Char1::new (b'~'),
        low_line: Char1::new (b'_'),

        quotation: Char1::new (b'"'),
        apostrophe: Char1::new (b'\''),

        carriage_return: Char1::new (0xD), // \r
        semicolon: Char1::new (b';'),
        slash: Char1::new (b'/'),
        space: Char1::new (b' '),
        tab_h: Char1::new (0x9), // \t
        tab_v: Char1::new (0xB), // \v
        form_feed: Char1::new (0xC), // \f
        escape: Char1::new (0x1B), // \e
        nbspace: Char1::new (0xA0),

        digit_0: Char1::new (b'0'),
        digit_1: Char1::new (b'1'),
        digit_2: Char1::new (b'2'),
        digit_3: Char1::new (b'3'),
        digit_4: Char1::new (b'4'),
        digit_5: Char1::new (b'5'),
        digit_6: Char1::new (b'6'),
        digit_7: Char1::new (b'7'),
        digit_8: Char1::new (b'8'),
        digit_9: Char1::new (b'9'),

        letter_a: Char1::new (b'a'),
        letter_b: Char1::new (b'b'),
        letter_c: Char1::new (b'c'),
        letter_d: Char1::new (b'd'),
        letter_e: Char1::new (b'e'),
        letter_f: Char1::new (b'f'),
        letter_g: Char1::new (b'g'),
        letter_h: Char1::new (b'h'),
        letter_i: Char1::new (b'i'),
        letter_j: Char1::new (b'j'),
        letter_k: Char1::new (b'k'),
        letter_l: Char1::new (b'l'),
        letter_m: Char1::new (b'm'),
        letter_n: Char1::new (b'n'),
        letter_o: Char1::new (b'o'),
        letter_p: Char1::new (b'p'),
        letter_q: Char1::new (b'q'),
        letter_r: Char1::new (b'r'),
        letter_s: Char1::new (b's'),
        letter_t: Char1::new (b't'),
        letter_u: Char1::new (b'u'),
        letter_v: Char1::new (b'v'),
        letter_w: Char1::new (b'w'),
        letter_x: Char1::new (b'x'),
        letter_y: Char1::new (b'y'),
        letter_z: Char1::new (b'z'),

        letter_t_a: Char1::new (b'A'),
        letter_t_b: Char1::new (b'B'),
        letter_t_c: Char1::new (b'C'),
        letter_t_d: Char1::new (b'D'),
        letter_t_e: Char1::new (b'E'),
        letter_t_f: Char1::new (b'F'),
        letter_t_g: Char1::new (b'G'),
        letter_t_h: Char1::new (b'H'),
        letter_t_i: Char1::new (b'I'),
        letter_t_j: Char1::new (b'J'),
        letter_t_k: Char1::new (b'K'),
        letter_t_l: Char1::new (b'L'),
        letter_t_m: Char1::new (b'M'),
        letter_t_n: Char1::new (b'N'),
        letter_t_o: Char1::new (b'O'),
        letter_t_p: Char1::new (b'P'),
        letter_t_q: Char1::new (b'Q'),
        letter_t_r: Char1::new (b'R'),
        letter_t_s: Char1::new (b'S'),
        letter_t_t: Char1::new (b'T'),
        letter_t_u: Char1::new (b'U'),
        letter_t_v: Char1::new (b'V'),
        letter_t_w: Char1::new (b'W'),
        letter_t_x: Char1::new (b'X'),
        letter_t_y: Char1::new (b'Y'),
        letter_t_z: Char1::new (b'Z'),

        null: Char1::new (0x0),
        bell: Char1::new (0x7),
        backspace: Char1::new (0x8),
        nextline: Char1::new (0x85), // \N

        longest_char: 1
    }
}
