use crate::io::Input;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TokenKind {
    DirectiveTag,
    DirectiveYaml,
    DirectiveUnknown,

    DocumentStart, // ---
    DocumentEnd,   // ...

    Comment, // #

    Newline, // \n+
    Indent,  // \s+
    Tab,     // \t+

    GT,       // >
    Dash,     // -
    Colon,    // :
    Comma,    // ,
    Pipe,     // |
    Question, // ?

    Anchor,    // &
    Alias,     // *
    TagHandle, // !

    DictionaryStart, // {
    DictionaryEnd,   // }

    SequenceStart, // [
    SequenceEnd,   // ]

    StringDouble, // "
    StringSingle, // '

    Raw,

    ReservedCommercialAt, // @
    ReservedGraveAccent,  // `
}

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub input: Input<'a>
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, input: Input<'a>) -> Self {
        Self { kind, input }
    }
}