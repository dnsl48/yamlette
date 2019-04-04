extern crate skimmer;

// use self::skimmer::symbol::CopySymbol;
// use txt::CharSet;

use std::ptr;

#[derive(Debug, Clone)]
pub enum EncodedString {
    Static(&'static [u8]),
    String(Vec<u8>),
}

impl EncodedString {
    pub fn len(&self) -> usize {
        match *self {
            EncodedString::Static(s) => s.len(),
            EncodedString::String(ref v) => v.len(),
        }
    }

    pub fn as_ptr(&self) -> *const u8 {
        match *self {
            EncodedString::Static(s) => s.as_ptr(),
            EncodedString::String(ref v) => v.as_ptr(),
        }
    }
}

impl From<&'static [u8]> for EncodedString {
    fn from(val: &'static [u8]) -> EncodedString {
        EncodedString::Static(val)
    }
}

impl From<Vec<u8>> for EncodedString {
    fn from(val: Vec<u8>) -> EncodedString {
        EncodedString::String(val)
    }
}

#[derive(Debug)]
pub enum Node {
    Empty,

    StringSpecificTag(EncodedString),

    String(EncodedString),
    SingleQuotedString(EncodedString),
    DoubleQuotedString(EncodedString),

    StringConcat(EncodedString, EncodedString),

    StringNewline(EncodedString),

    AmpersandString(EncodedString),
    AsteriskString(EncodedString),

    Indent(usize),
    NewlineIndent(usize),
    CommaNewlineIndent(usize),
    IndentHyphenSpace(usize),
    NewlineIndentHyphenSpace(usize),

    Comma,
    Colon,
    Question,
    Hyphen,
    Dot,

    QuestionNewline,
    QuestionNewlineIndent(usize),
    IndentQuestionSpace(usize),
    NewlineIndentQuestionSpace(usize),

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
    ColonNewlineIndent(usize),

    TripleHyphenNewline,
    TripleDotNewline,
}

impl Node {
    pub fn indent(&mut self, len: usize) {
        match *self {
            Node::Indent(ref mut size) => {
                *size += len;
            }
            Node::NewlineIndent(ref mut size) => {
                *size += len;
            }
            Node::CommaNewlineIndent(ref mut size) => {
                *size += len;
            }
            Node::IndentHyphenSpace(ref mut size) => {
                *size += len;
            }
            Node::NewlineIndentHyphenSpace(ref mut size) => {
                *size += len;
            }
            Node::IndentQuestionSpace(ref mut size) => {
                *size += len;
            }
            Node::NewlineIndentQuestionSpace(ref mut size) => {
                *size += len;
            }
            Node::QuestionNewlineIndent(ref mut size) => {
                *size += len;
            }
            Node::ColonNewlineIndent(ref mut size) => {
                *size += len;
            }
            _ => (),
        }
    }

    pub fn is_newline(&self) -> bool {
        match *self {
            Node::Newline
            | Node::ColonNewline
            | Node::ColonNewlineIndent(_)
            | Node::QuestionNewline
            | Node::QuestionNewlineIndent(_)
            | Node::TripleDotNewline
            | Node::StringNewline(_)
            | Node::TripleHyphenNewline
            | Node::NewlineIndent(_)
            | Node::CommaNewlineIndent(_)
            | Node::NewlineIndentHyphenSpace(_) => true,
            Node::NewlineIndentQuestionSpace(_) => true,
            _ => false,
        }
    }

    pub fn is_flow_opening(&self) -> bool {
        match *self {
            Node::CurlyBrackets
            | Node::CurlyBracketOpen
            | Node::SquareBrackets
            | Node::SquareBracketOpen
            | Node::SingleQuotedString(_)
            | Node::DoubleQuotedString(_) => true,
            _ => false,
        }
    }

    pub fn is_flow_dict_opening(&self) -> bool {
        match *self {
            Node::CurlyBrackets | Node::CurlyBracketOpen => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Renderer;

impl Renderer {
    pub fn render_into_vec(&self, vec: &mut Vec<u8>, node: Node) {
        let node_len = self.node_len(&node);
        let vec_len = vec.len();
        vec.reserve(node_len);

        unsafe {
            vec.set_len(vec_len + node_len);
            let ptr = vec.as_mut_ptr().offset(vec_len as isize);
            self.render_onto_ptr(ptr, &node);
        }
    }

    pub fn node_len(&self, node: &Node) -> usize {
        match *node {
            Node::Empty => 0,

            Node::Indent(size) => size,
            Node::NewlineIndent(size) => 1 + size,
            Node::CommaNewlineIndent(size) => 2 + size,
            Node::IndentHyphenSpace(size) => 2 + size,
            Node::NewlineIndentHyphenSpace(size) => 3 + size,

            Node::IndentQuestionSpace(size) => 2 + size,
            Node::NewlineIndentQuestionSpace(size) => 3 + size,

            Node::StringSpecificTag(ref s) => s.len() + 3,

            Node::StringConcat(ref former, ref latter) => former.len() + latter.len(),

            Node::String(ref s) => s.len(),
            Node::StringNewline(ref s) => s.len() + 1,
            Node::SingleQuotedString(ref s) => s.len() + 2,
            Node::DoubleQuotedString(ref s) => s.len() + 2,

            Node::AmpersandString(ref s) => 1 + s.len(),
            Node::AsteriskString(ref s) => 1 + s.len(),

            Node::Comma
            | Node::Colon
            | Node::Space
            | Node::Question
            | Node::Hyphen
            | Node::Newline
            | Node::SquareBracketOpen
            | Node::SquareBracketClose
            | Node::CurlyBracketOpen
            | Node::CurlyBracketClose
            | Node::Dot => 1,

            Node::QuestionSpace
            | Node::QuestionNewline
            | Node::SquareBrackets
            | Node::CommaSpace
            | Node::ColonSpace
            | Node::HyphenSpace
            | Node::ColonNewline
            | Node::CurlyBrackets => 2,

            Node::QuestionNewlineIndent(size) => 2 + size,
            Node::ColonNewlineIndent(size) => 2 + size,

            Node::TripleHyphenNewline => 3 + 1,
            Node::TripleDotNewline => 3 + 1,
        }
    }

    pub unsafe fn render_onto_ptr(&self, mut dst_ptr: *mut u8, node: &Node) -> *mut u8 {
        match *node {
            Node::Empty => (),

            // Node::Indent (size) => { dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size); }
            Node::Indent(size) => {
                dst_ptr = copy_to_ptr_times(b' ', dst_ptr, size);
            }

            Node::NewlineIndent(size) => {
                // dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'\n', dst_ptr);
                // dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
                dst_ptr = copy_to_ptr_times(b' ', dst_ptr, size);
            }

            Node::IndentHyphenSpace(size) => {
                // dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
                dst_ptr = copy_to_ptr_times(b' ', dst_ptr, size);
                // dst_ptr = self.hyphen.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'-', dst_ptr);
                // dst_ptr = self.space.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b' ', dst_ptr);
            }

            Node::NewlineIndentHyphenSpace(size) => {
                // dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'\n', dst_ptr);
                // dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
                dst_ptr = copy_to_ptr_times(b' ', dst_ptr, size);
                // dst_ptr = self.hyphen.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'-', dst_ptr);
                // dst_ptr = self.space.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b' ', dst_ptr);
            }

            Node::IndentQuestionSpace(size) => {
                // dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
                dst_ptr = copy_to_ptr_times(b' ', dst_ptr, size);
                // dst_ptr = self.question.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'?', dst_ptr);
                // dst_ptr = self.space.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b' ', dst_ptr);
            }

            Node::NewlineIndentQuestionSpace(size) => {
                // dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'\n', dst_ptr);
                // dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
                dst_ptr = copy_to_ptr_times(b' ', dst_ptr, size);
                // dst_ptr = self.question.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'?', dst_ptr);
                // dst_ptr = self.space.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b' ', dst_ptr);
            }

            Node::CommaNewlineIndent(size) => {
                // dst_ptr = self.comma.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b',', dst_ptr);
                // dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'\n', dst_ptr);
                // dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
                dst_ptr = copy_to_ptr_times(b' ', dst_ptr, size);
            }

            Node::CommaSpace => {
                // dst_ptr = self.comma.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b',', dst_ptr);
                // dst_ptr = self.space.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b' ', dst_ptr);
            }

            Node::ColonSpace => {
                // dst_ptr = self.colon.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b':', dst_ptr);
                // dst_ptr = self.space.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b' ', dst_ptr);
            }

            Node::QuestionNewline => {
                // dst_ptr = self.question.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'?', dst_ptr);
                // dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'\n', dst_ptr);
            }

            Node::QuestionNewlineIndent(size) => {
                // dst_ptr = self.question.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'?', dst_ptr);
                // dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'\n', dst_ptr);
                // dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
                dst_ptr = copy_to_ptr_times(b' ', dst_ptr, size);
            }

            Node::QuestionSpace => {
                // dst_ptr = self.question.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'?', dst_ptr);
                // dst_ptr = self.space.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b' ', dst_ptr);
            }

            Node::ColonNewline => {
                // dst_ptr = self.colon.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b':', dst_ptr);
                // dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'\n', dst_ptr);
            }

            Node::ColonNewlineIndent(size) => {
                // dst_ptr = self.colon.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b':', dst_ptr);
                // dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'\n', dst_ptr);
                // dst_ptr = self.space.copy_to_ptr_times (dst_ptr, size);
                dst_ptr = copy_to_ptr_times(b' ', dst_ptr, size);
            }

            Node::HyphenSpace => {
                // dst_ptr = self.hyphen.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'-', dst_ptr);
                // dst_ptr = self.space.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b' ', dst_ptr);
            }

            Node::SquareBrackets => {
                // dst_ptr = self.square_bracket_open.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'[', dst_ptr);
                // dst_ptr = self.square_bracket_close.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b']', dst_ptr);
            }
            // Node::SquareBracketOpen => { dst_ptr = self.square_bracket_open.copy_to_ptr (dst_ptr); }
            Node::SquareBracketOpen => {
                dst_ptr = copy_to_ptr(b'[', dst_ptr);
            }
            // Node::SquareBracketClose => { dst_ptr = self.square_bracket_close.copy_to_ptr (dst_ptr); }
            Node::SquareBracketClose => {
                dst_ptr = copy_to_ptr(b']', dst_ptr);
            }

            Node::CurlyBrackets => {
                // dst_ptr = self.curly_bracket_open.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'{', dst_ptr);
                // dst_ptr = self.curly_bracket_close.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'}', dst_ptr);
            }
            // Node::CurlyBracketOpen => { dst_ptr = self.curly_bracket_open.copy_to_ptr (dst_ptr); }
            Node::CurlyBracketOpen => {
                dst_ptr = copy_to_ptr(b'{', dst_ptr);
            }
            // Node::CurlyBracketClose => { dst_ptr = self.curly_bracket_close.copy_to_ptr (dst_ptr); }
            Node::CurlyBracketClose => {
                dst_ptr = copy_to_ptr(b'}', dst_ptr);
            }

            // Node::Hyphen => { dst_ptr = self.hyphen.copy_to_ptr (dst_ptr); }
            Node::Hyphen => {
                dst_ptr = copy_to_ptr(b'-', dst_ptr);
            }
            // Node::Dot => { dst_ptr = self.dot.copy_to_ptr (dst_ptr); }
            Node::Dot => {
                dst_ptr = copy_to_ptr(b'.', dst_ptr);
            }
            // Node::Question => { dst_ptr = self.question.copy_to_ptr (dst_ptr); }
            Node::Question => {
                dst_ptr = copy_to_ptr(b'?', dst_ptr);
            }
            // Node::Comma => { dst_ptr = self.comma.copy_to_ptr (dst_ptr); }
            Node::Comma => {
                dst_ptr = copy_to_ptr(b',', dst_ptr);
            }
            // Node::Colon => { dst_ptr = self.colon.copy_to_ptr (dst_ptr); }
            Node::Colon => {
                dst_ptr = copy_to_ptr(b':', dst_ptr);
            }
            // Node::Space => { dst_ptr = self.space.copy_to_ptr (dst_ptr); }
            Node::Space => {
                dst_ptr = copy_to_ptr(b' ', dst_ptr);
            }
            // Node::Newline => { dst_ptr = self.newline.copy_to_ptr (dst_ptr); }
            Node::Newline => {
                dst_ptr = copy_to_ptr(b'\n', dst_ptr);
            }

            Node::TripleHyphenNewline => {
                // dst_ptr = self.hyphen.copy_to_ptr_times (dst_ptr, 3);
                dst_ptr = copy_to_ptr_times(b'-', dst_ptr, 3);
                // dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'\n', dst_ptr);
            }

            Node::TripleDotNewline => {
                // dst_ptr = self.dot.copy_to_ptr_times (dst_ptr, 3);
                dst_ptr = copy_to_ptr_times(b'.', dst_ptr, 3);
                // dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'\n', dst_ptr);
            }

            Node::StringSpecificTag(ref vec) => {
                // dst_ptr = self.exclamation.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'!', dst_ptr);
                // dst_ptr = self.lt.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'<', dst_ptr);
                let len = vec.len();
                ptr::copy_nonoverlapping(vec.as_ptr(), dst_ptr, len);
                dst_ptr = dst_ptr.offset(len as isize);
                // dst_ptr = self.gt.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'>', dst_ptr);
            }

            Node::StringConcat(ref former, ref latter) => {
                let len = former.len();
                ptr::copy_nonoverlapping(former.as_ptr(), dst_ptr, len);
                dst_ptr = dst_ptr.offset(len as isize);

                let len = latter.len();
                ptr::copy_nonoverlapping(latter.as_ptr(), dst_ptr, len);
                dst_ptr = dst_ptr.offset(len as isize);
            }

            Node::String(ref s) => {
                let len = s.len();
                ptr::copy_nonoverlapping(s.as_ptr(), dst_ptr, len);
                dst_ptr = dst_ptr.offset(len as isize);
            }
            Node::StringNewline(ref s) => {
                let len = s.len();
                ptr::copy_nonoverlapping(s.as_ptr(), dst_ptr, len);
                dst_ptr = dst_ptr.offset(len as isize);
                // dst_ptr = self.newline.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'\n', dst_ptr);
            }
            Node::SingleQuotedString(ref s) => {
                // dst_ptr = self.apostrophe.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'\'', dst_ptr);
                let len = s.len();
                ptr::copy_nonoverlapping(s.as_ptr(), dst_ptr, len);
                dst_ptr = dst_ptr.offset(len as isize);
                // dst_ptr = self.apostrophe.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'\'', dst_ptr);
            }
            Node::DoubleQuotedString(ref s) => {
                // dst_ptr = self.quotation.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'"', dst_ptr);
                let len = s.len();
                ptr::copy_nonoverlapping(s.as_ptr(), dst_ptr, len);
                dst_ptr = dst_ptr.offset(len as isize);
                // dst_ptr = self.quotation.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'"', dst_ptr);
            }

            Node::AmpersandString(ref s) => {
                // dst_ptr = self.ampersand.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'&', dst_ptr);
                let len = s.len();
                ptr::copy_nonoverlapping(s.as_ptr(), dst_ptr, len);
                dst_ptr = dst_ptr.offset(len as isize);
            }
            Node::AsteriskString(ref s) => {
                // dst_ptr = self.asterisk.copy_to_ptr (dst_ptr);
                dst_ptr = copy_to_ptr(b'*', dst_ptr);
                let len = s.len();
                ptr::copy_nonoverlapping(s.as_ptr(), dst_ptr, len);
                dst_ptr = dst_ptr.offset(len as isize);
            }
        };

        dst_ptr
    }
}

#[inline(always)]
unsafe fn copy_to_ptr(byte: u8, dst: *mut u8) -> *mut u8 {
    *dst = byte;
    dst.offset(1)
}

#[inline(always)]
unsafe fn copy_to_ptr_times(byte: u8, mut dst: *mut u8, times: usize) -> *mut u8 {
    for _ in 0..times {
        *dst = byte;
        dst = dst.offset(1);
    }
    dst
}
