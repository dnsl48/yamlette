extern crate skimmer;

use self::skimmer::symbol::{ Char, Rune, Symbol };

use txt::CharSet;

use std::ptr;



#[derive (Debug, Clone)]
pub enum EncodedString {
    Static (&'static [u8]),
    String (Vec<u8>)
}


impl EncodedString {
    pub fn len (&self) -> usize { match *self {
        EncodedString::Static (s) => s.len (),
        EncodedString::String (ref v) => v.len ()
    } }

    pub fn as_ptr (&self) -> *const u8 { match *self {
        EncodedString::Static (s) => s.as_ptr (),
        EncodedString::String (ref v) => v.as_ptr ()
    } }
}


impl From<&'static [u8]> for EncodedString {
    fn from (val: &'static [u8]) -> EncodedString { EncodedString::Static (val) }
}


impl From<Vec<u8>> for EncodedString {
    fn from (val: Vec<u8>) -> EncodedString { EncodedString::String (val) }
}



#[derive (Debug)]
pub enum Node {
    Empty,

    StringSpecificTag (EncodedString),

    String (EncodedString),
    SingleQuotedString (EncodedString),
    DoubleQuotedString (EncodedString),

    StringConcat (EncodedString, EncodedString),

    StringNewline (EncodedString),

    AmpersandString (EncodedString),
    AsteriskString (EncodedString),

    Indent (usize),
    NewlineIndent (usize),
    CommaNewlineIndent (usize),
    IndentHyphenSpace (usize),
    NewlineIndentHyphenSpace (usize),

    Comma,
    Colon,
    Question,
    Hyphen,
    Dot,

    QuestionNewline,
    QuestionNewlineIndent (usize),
    IndentQuestionSpace (usize),
    NewlineIndentQuestionSpace (usize),

    Newline,

    SquareBrackets,
    SquareBracketOpen,
    SquareBracketClose,

    CurlyBrackets,
    CurlyBracketOpen,
    CurlyBracketClose,

    QuestionSpace,
    CommaSpace,
    ColonSpace,
    HyphenSpace,
    Space,

    ColonNewline,
    ColonNewlineIndent (usize),

    TripleHyphenNewline,
    TripleDotNewline
}



impl Node {
    pub fn indent (&mut self, len: usize) {
        match *self {
            Node::Indent (ref mut size) => { *size += len; }
            Node::NewlineIndent (ref mut size) => { *size += len; }
            Node::CommaNewlineIndent (ref mut size) => { *size += len; }
            Node::IndentHyphenSpace (ref mut size) => { *size += len; }
            Node::NewlineIndentHyphenSpace (ref mut size) => { *size += len; }
            Node::IndentQuestionSpace (ref mut size) => { *size += len; }
            Node::NewlineIndentQuestionSpace (ref mut size) => { *size += len; }
            Node::QuestionNewlineIndent (ref mut size) => { *size += len; }
            Node::ColonNewlineIndent (ref mut size) => { *size += len; }
            _ => ()
        }
    }

    pub fn is_newline (&self) -> bool {
        match *self {
            Node::Newline |
            Node::ColonNewline |
            Node::ColonNewlineIndent (_) |
            Node::QuestionNewline |
            Node::QuestionNewlineIndent (_) |
            Node::TripleDotNewline |
            Node::StringNewline (_) |
            Node::TripleHyphenNewline |
            Node::NewlineIndent (_) |
            Node::CommaNewlineIndent (_) |
            Node::NewlineIndentHyphenSpace (_) => true,
            Node::NewlineIndentQuestionSpace (_) => true,
            _ => false
        }
    }

    pub fn is_flow_opening (&self) -> bool {
        match *self {
            Node::CurlyBrackets |
            Node::CurlyBracketOpen |
            Node::SquareBrackets |
            Node::SquareBracketOpen |
            Node::SingleQuotedString (_) |
            Node::DoubleQuotedString (_) => true,
            _ => false
        }
    }

    pub fn is_flow_dict_opening (&self) -> bool {
        match *self {
            Node::CurlyBrackets |
            Node::CurlyBracketOpen => true,
            _ => false
        }
    }
}



#[derive(Clone)]
pub struct Renderer {
    newline: Rune,

    ampersand: Char,
    asterisk: Char,

    space: Char,
    comma: Char,
    colon: Char,

    gt: Char,
    lt: Char,

    exclamation: Char,
    hyphen: Char,
    dot: Char,

    apostrophe: Char,
    quotation: Char,

    question: Char,

    square_bracket_open: Char,
    square_bracket_close: Char,

    curly_bracket_open: Char,
    curly_bracket_close: Char
}


impl Renderer {
    pub fn new (cset: &CharSet) -> Renderer {
        Renderer {
            newline: Rune::from (cset.line_feed.clone ()),
            ampersand: cset.ampersand.clone (),
            asterisk: cset.asterisk.clone (),
            gt: cset.greater_than.clone (),
            lt: cset.less_than.clone (),
            exclamation: cset.exclamation.clone (),
            hyphen: cset.hyphen_minus.clone (),
            dot: cset.full_stop.clone (),
            apostrophe: cset.apostrophe.clone (),
            quotation: cset.quotation.clone (),
            question: cset.question.clone (),
            space: cset.space.clone (),
            comma: cset.comma.clone (),
            colon: cset.colon.clone (),
            square_bracket_open: cset.bracket_square_left.clone (),
            square_bracket_close: cset.bracket_square_right.clone (),
            curly_bracket_open: cset.bracket_curly_left.clone (),
            curly_bracket_close: cset.bracket_curly_right.clone ()
        }
    }


    pub fn new_crlf (cset: &CharSet) -> Renderer {
        let mut renderer = Renderer::new (cset);
        renderer.newline = Rune::from (cset.crlf.clone ());
        renderer
    }


    fn _indent_space_len (&self, size: usize) -> usize { size * self.space.len () }


    pub fn render_into_vec (&self, vec: &mut Vec<u8>, node: Node) {
        let node_len = self.node_len (&node);
        let vec_len = vec.len ();
        vec.reserve (node_len);

        unsafe {
            vec.set_len (vec_len + node_len);
            let ptr = vec.as_mut_ptr ().offset (vec_len as isize);
            self.render_onto_ptr (ptr, &node);
        }
    }


    pub fn node_len (&self, node: &Node) -> usize {
        match *node {
            Node::Empty => 0,

            Node::Indent (size) => self._indent_space_len (size),
            Node::NewlineIndent (size) => self.newline.len () + self._indent_space_len (size),
            Node::CommaNewlineIndent (size) => self.comma.len () + self.newline.len () + self._indent_space_len (size),
            Node::IndentHyphenSpace (size) => self.hyphen.len () + self.space.len () + self._indent_space_len (size),
            Node::NewlineIndentHyphenSpace (size) => self.newline.len () + self.hyphen.len () + self.space.len () + self._indent_space_len (size),

            Node::IndentQuestionSpace (size) => self.question.len () + self.space.len () + self._indent_space_len (size),
            Node::NewlineIndentQuestionSpace (size) => self.newline.len () + self.question.len () + self.space.len () + self._indent_space_len (size),

            Node::StringSpecificTag (ref s) => s.len () + self.exclamation.len () + self.lt.len () + self.gt.len (),

            Node::StringConcat (ref former, ref latter) => former.len () + latter.len (),

            Node::String (ref s) => s.len (),
            Node::StringNewline (ref s) => s.len () + self.newline.len (),
            Node::SingleQuotedString (ref s) => s.len () + self.apostrophe.len () * 2,
            Node::DoubleQuotedString (ref s) => s.len () + self.quotation.len () * 2,

            Node::AmpersandString (ref s) => self.ampersand.len () + s.len (),
            Node::AsteriskString (ref s) => self.asterisk.len () + s.len (),

            Node::Comma => self.comma.len (),
            Node::Colon => self.colon.len (),
            Node::Space => self.space.len (),
            Node::Question => self.question.len (),
            Node::Hyphen => self.hyphen.len (),
            Node::Dot => self.dot.len (),

            Node::QuestionNewline => self.question.len () + self.newline.len (),
            Node::QuestionNewlineIndent (size) => self.question.len () + self.newline.len () + self._indent_space_len (size),
            Node::QuestionSpace => self.question.len () + self.space.len (),

            Node::Newline => self.newline.len (),

            Node::SquareBrackets => self.square_bracket_open.len () + self.square_bracket_close.len (),
            Node::SquareBracketOpen => self.square_bracket_open.len (),
            Node::SquareBracketClose => self.square_bracket_close.len (),

            Node::CurlyBrackets => self.curly_bracket_open.len () + self.curly_bracket_close.len (),
            Node::CurlyBracketOpen => self.curly_bracket_open.len (),
            Node::CurlyBracketClose => self.curly_bracket_close.len (),

            Node::CommaSpace => self.comma.len () + self.space.len (),
            Node::ColonSpace => self.colon.len () + self.space.len (),
            Node::HyphenSpace => self.hyphen.len () + self.space.len (),

            Node::ColonNewline => self.colon.len () + self.newline.len (),
            Node::ColonNewlineIndent (size) => self.colon.len () + self.newline.len () + self._indent_space_len (size),

            Node::TripleHyphenNewline => self.hyphen.len () * 3 + self.newline.len (),
            Node::TripleDotNewline => self.dot.len () * 3 + self.newline.len ()
        }
    }


    pub unsafe fn render_onto_ptr (&self, mut dst_ptr: *mut u8, node: &Node) -> *mut u8 {
        match *node {
            Node::Empty => (),

            Node::Indent (size) => { dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size); }

            Node::NewlineIndent (size) => {
                dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
            }

            Node::IndentHyphenSpace (size) => {
                dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
                dst_ptr = self.hyphen.copy_to_ptr (dst_ptr);
                dst_ptr = self.space.copy_to_ptr (dst_ptr);
            }

            Node::NewlineIndentHyphenSpace (size) => {
                dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
                dst_ptr = self.hyphen.copy_to_ptr (dst_ptr);
                dst_ptr = self.space.copy_to_ptr (dst_ptr);
            }

            Node::IndentQuestionSpace (size) => {
                dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
                dst_ptr = self.question.copy_to_ptr (dst_ptr);
                dst_ptr = self.space.copy_to_ptr (dst_ptr);
            }

            Node::NewlineIndentQuestionSpace (size) => {
                dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
                dst_ptr = self.question.copy_to_ptr (dst_ptr);
                dst_ptr = self.space.copy_to_ptr (dst_ptr);
            }

            Node::CommaNewlineIndent (size) => {
                dst_ptr = self.comma.copy_to_ptr (dst_ptr);
                dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
            }

            Node::CommaSpace => {
                dst_ptr = self.comma.copy_to_ptr (dst_ptr);
                dst_ptr = self.space.copy_to_ptr (dst_ptr);
            }

            Node::ColonSpace => {
                dst_ptr = self.colon.copy_to_ptr (dst_ptr);
                dst_ptr = self.space.copy_to_ptr (dst_ptr);
            }

            Node::QuestionNewline => {
                dst_ptr = self.question.copy_to_ptr (dst_ptr);
                dst_ptr = self.newline.copy_to_ptr (dst_ptr);
            }

            Node::QuestionNewlineIndent (size) => {
                dst_ptr = self.question.copy_to_ptr (dst_ptr);
                dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
            }

            Node::QuestionSpace => {
                dst_ptr = self.question.copy_to_ptr (dst_ptr);
                dst_ptr = self.space.copy_to_ptr (dst_ptr);
            }

            Node::ColonNewline => {
                dst_ptr = self.colon.copy_to_ptr (dst_ptr);
                dst_ptr = self.newline.copy_to_ptr (dst_ptr);
            }

            Node::ColonNewlineIndent (size) => {
                dst_ptr = self.colon.copy_to_ptr (dst_ptr);
                dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
            }

            Node::HyphenSpace => {
                dst_ptr = self.hyphen.copy_to_ptr (dst_ptr);
                dst_ptr = self.space.copy_to_ptr (dst_ptr);
            }

            Node::SquareBrackets => {
                dst_ptr = self.square_bracket_open.copy_to_ptr (dst_ptr);
                dst_ptr = self.square_bracket_close.copy_to_ptr (dst_ptr);
            }
            Node::SquareBracketOpen => { dst_ptr = self.square_bracket_open.copy_to_ptr (dst_ptr); }
            Node::SquareBracketClose => { dst_ptr = self.square_bracket_close.copy_to_ptr (dst_ptr); }


            Node::CurlyBrackets => {
                dst_ptr = self.curly_bracket_open.copy_to_ptr (dst_ptr);
                dst_ptr = self.curly_bracket_close.copy_to_ptr (dst_ptr);
            }
            Node::CurlyBracketOpen => { dst_ptr = self.curly_bracket_open.copy_to_ptr (dst_ptr); }
            Node::CurlyBracketClose => { dst_ptr = self.curly_bracket_close.copy_to_ptr (dst_ptr); }

            Node::Hyphen => { dst_ptr = self.hyphen.copy_to_ptr (dst_ptr); }
            Node::Dot => { dst_ptr = self.dot.copy_to_ptr (dst_ptr); }
            Node::Question => { dst_ptr = self.question.copy_to_ptr (dst_ptr); }
            Node::Comma => { dst_ptr = self.comma.copy_to_ptr (dst_ptr); }
            Node::Colon => { dst_ptr = self.colon.copy_to_ptr (dst_ptr); }
            Node::Space => { dst_ptr = self.space.copy_to_ptr (dst_ptr); }
            Node::Newline => { dst_ptr = self.newline.copy_to_ptr (dst_ptr); }


            Node::TripleHyphenNewline => {
                dst_ptr = self.hyphen.copy_to_ptr_times (dst_ptr, 3);
                dst_ptr = self.newline.copy_to_ptr (dst_ptr);
            }

            Node::TripleDotNewline => {
                dst_ptr = self.dot.copy_to_ptr_times (dst_ptr, 3);
                dst_ptr = self.newline.copy_to_ptr (dst_ptr);
            }


            Node::StringSpecificTag (ref vec) => {
                dst_ptr = self.exclamation.copy_to_ptr (dst_ptr);
                dst_ptr = self.lt.copy_to_ptr (dst_ptr);
                let len = vec.len ();
                ptr::copy_nonoverlapping (vec.as_ptr (), dst_ptr, len);
                dst_ptr = dst_ptr.offset (len as isize);
                dst_ptr = self.gt.copy_to_ptr (dst_ptr);
            }


            Node::StringConcat (ref former, ref latter) => {
                let len = former.len ();
                ptr::copy_nonoverlapping (former.as_ptr (), dst_ptr, len);
                dst_ptr = dst_ptr.offset (len as isize);

                let len = latter.len ();
                ptr::copy_nonoverlapping (latter.as_ptr (), dst_ptr, len);
                dst_ptr = dst_ptr.offset (len as isize);
            }


            Node::String (ref s) => {
                let len = s.len ();
                ptr::copy_nonoverlapping (s.as_ptr (), dst_ptr, len);
                dst_ptr = dst_ptr.offset (len as isize);
            }
            Node::StringNewline (ref s) => {
                let len = s.len ();
                ptr::copy_nonoverlapping (s.as_ptr (), dst_ptr, len);
                dst_ptr = dst_ptr.offset (len as isize);
                dst_ptr = self.newline.copy_to_ptr (dst_ptr);
            }
            Node::SingleQuotedString (ref s) => {
                dst_ptr = self.apostrophe.copy_to_ptr (dst_ptr);
                let len = s.len ();
                ptr::copy_nonoverlapping (s.as_ptr (), dst_ptr, len);
                dst_ptr = dst_ptr.offset (len as isize);
                dst_ptr = self.apostrophe.copy_to_ptr (dst_ptr);
            }
            Node::DoubleQuotedString (ref s) => {
                dst_ptr = self.quotation.copy_to_ptr (dst_ptr);
                let len = s.len ();
                ptr::copy_nonoverlapping (s.as_ptr (), dst_ptr, len);
                dst_ptr = dst_ptr.offset (len as isize);
                dst_ptr = self.quotation.copy_to_ptr (dst_ptr);
            }

            Node::AmpersandString (ref s) => {
                dst_ptr = self.ampersand.copy_to_ptr (dst_ptr);
                let len = s.len ();
                ptr::copy_nonoverlapping (s.as_ptr (), dst_ptr, len);
                dst_ptr = dst_ptr.offset (len as isize);
            }
            Node::AsteriskString (ref s) => {
                dst_ptr = self.asterisk.copy_to_ptr (dst_ptr);
                let len = s.len ();
                ptr::copy_nonoverlapping (s.as_ptr (), dst_ptr, len);
                dst_ptr = dst_ptr.offset (len as isize);
            }
        };

        dst_ptr
    }
}
