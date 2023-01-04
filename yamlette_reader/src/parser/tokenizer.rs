use crate::error::Error;
use crate::io::Input;

use super::scanner;
use super::{Token, TokenKind};

// use nom::branch::alt;
// use nom::bytes::streaming::{tag, take};

pub type TokenResult<'a> = nom::IResult<Input<'a>, Token<'a>, Error<Input<'a>>>;

/// Scan the following bytes until the closing double-quote character (").
/// Takes into account backslash as the escape character, thus it will skip
/// the escaped double-quotes (\").
/// Takes into account double-backslash as escape of the escape character,
/// so that \\" still counts as the closing double-quote.
/// See YAML 1.2 spec, 7.3.1. Single-Quoted Style
fn double_quoted<'a>(input: Input<'a>) -> TokenResult<'a> {
    use nom::bytes::streaming::take;

    let len = scanner::scan_double_quoted(&*input);

    if len == 0 {
        Err(nom::Err::Incomplete(nom::Needed::Unknown))
    } else {
        let (input, o) = take(len)(input)?;
        Ok((input, Token::new(TokenKind::StringDouble, o)))
    }
}

/// Scan the following bytes until the closing single-quote character (').
/// Takes into account double-single-quote as the escape sequence, thus it will skip
/// the escaped single-quotes ('').
/// See YAML 1.2 spec, 7.3.2. Single-Quoted Style
fn single_quoted<'a>(input: Input<'a>) -> TokenResult<'a> {
    use nom::bytes::streaming::take;

    let len = scanner::scan_single_quoted(&*input);

    if len == 0 {
        Err(nom::Err::Incomplete(nom::Needed::Unknown))
    } else {
        let (input, o) = take(len)(input)?;
        Ok((input, Token::new(TokenKind::StringSingle, o)))
    }
}

fn single_directive_tag<'a>(input: Input<'a>) -> TokenResult<'a> {
    use nom::bytes::streaming::tag;

    tag("%TAG")(input).map(|(i, o)| (i, Token::new(TokenKind::DirectiveTag, o)))
}

fn single_directive_yaml<'a>(input: Input<'a>) -> TokenResult<'a> {
    use nom::bytes::streaming::tag;

    tag("%YAML")(input).map(|(i, o)| (i, Token::new(TokenKind::DirectiveYaml, o)))
}

fn single_directive_unknown<'a>(input: Input<'a>) -> TokenResult<'a> {
    use nom::bytes::streaming::take;

    take(scanner::scan_line(&*input))(input)
        .map(|(i, o)| (i, Token::new(TokenKind::DirectiveUnknown, o)))
}

fn single_directive<'a>(input: Input<'a>) -> TokenResult<'a> {
    use nom::branch::alt;

    alt((
        single_directive_yaml,
        single_directive_tag,
        single_directive_unknown,
    ))(input)
}

fn document_start<'a>(input: Input<'a>) -> TokenResult<'a> {
    use nom::bytes::streaming::tag;

    tag("---")(input).map(|(i, o)| (i, Token::new(TokenKind::DocumentStart, o)))
}

fn dash<'a>(input: Input<'a>) -> TokenResult<'a> {
    use nom::branch::alt;
    use nom::combinator::{peek, recognize, eof};
    use nom::character::complete::{char, line_ending, one_of};
    use nom::sequence::terminated;

    terminated(
        recognize(char('-')),
        peek(alt((
            eof,
            recognize(one_of(" \t")),
            recognize(line_ending),
            recognize(char('\r'))
        )))
    )(input).map(|(i, o)| {
        (i, Token::new(TokenKind::Dash, o)) })
}

fn dash_or_docstart<'a>(input: Input<'a>) -> TokenResult<'a> {
    use nom::branch::alt;

    alt((document_start, dash))(input)
}

fn question_mark<'a>(input: Input<'a>) -> TokenResult<'a> {
    use nom::branch::alt;
    use nom::combinator::{peek, recognize, eof};
    use nom::character::complete::{char, line_ending, one_of};
    use nom::sequence::terminated;

    terminated(
        recognize(char('?')),
        peek(alt((
            eof,
            recognize(one_of(" \t")),
            recognize(line_ending),
            recognize(char('\r'))
        )))
    )(input).map(|(i, o)| (i, Token::new(TokenKind::Question, o)))
}

fn raw<'a>(input: Input<'a>) -> TokenResult<'a> {
    use nom::bytes::streaming::take;

    let len = scanner::scan_until_raw_stops(&*input);

    if len == 0 {
        Err(nom::Err::Incomplete(nom::Needed::Unknown))
    } else {
        let (input, o) = take(len)(input)?;
        Ok((input, Token::new(TokenKind::Raw, o)))
    }
}

pub fn get_token<'a>(input: Input<'a>) -> TokenResult<'a> {
    use nom::bytes::streaming::take;

    match scanner::get_byte_at(&*input, 0) {
        None => return Err(nom::Err::Incomplete(nom::Needed::Unknown)),
        Some(b',') => { 
            return take(1usize)(input).map(|(i, o)| (i, Token::new(TokenKind::Comma, o)))
        }
        Some(b':') => {
            return take(1usize)(input).map(|(i, o)| (i, Token::new(TokenKind::Colon, o)))
        }
        Some(b'{') => {
            return take(1usize)(input).map(|(i, o)| (i, Token::new(TokenKind::DictionaryStart, o)))
        }
        Some(b'}') => {
            return take(1usize)(input).map(|(i, o)| (i, Token::new(TokenKind::DictionaryEnd, o)))
        }
        Some(b'[') => {
            return take(1usize)(input).map(|(i, o)| (i, Token::new(TokenKind::SequenceStart, o)))
        }
        Some(b']') => {
            return take(1usize)(input).map(|(i, o)| (i, Token::new(TokenKind::SequenceEnd, o)))
        }
        Some(b'>') => return take(1usize)(input).map(|(i, o)| (i, Token::new(TokenKind::GT, o))),
        Some(b'|') => return take(1usize)(input).map(|(i, o)| (i, Token::new(TokenKind::Pipe, o))),
        Some(b'"') => return double_quoted(input),
        Some(b'\'') => return single_quoted(input),
        Some(b'#') => {
            return take(scanner::scan_line(&*input))(input)
                .map(|(i, o)| (i, Token::new(TokenKind::Comment, o)))
        }
        Some(b'*') => {
            return take(scanner::scan_until_alias_stops(&*input))(input)
                .map(|(i, o)| (i, Token::new(TokenKind::Alias, o)))
        }
        Some(b'&') => {
            return take(scanner::scan_until_anchor_stops(&*input, 0))(input)
                .map(|(i, o)| (i, Token::new(TokenKind::Anchor, o)))
        }
        Some(b'.')
            if scanner::get_byte_at(&*input, 1) == Some(b'.')
                && scanner::get_byte_at(&*input, 2) == Some(b'.') =>
        {
            return take(3usize)(input).map(|(i, o)| (i, Token::new(TokenKind::DocumentEnd, o)))
        }

        Some(b' ') => {
            return take(scanner::scan_while_spaces(&*input, 0))(input)
                .map(|(i, o)| (i, Token::new(TokenKind::Indent, o)))
        }

        Some(b'\n') | Some(b'\r') => {
            return take(scanner::scan_while_newline(&*input, 0))(input)
                .map(|(i, o)| (i, Token::new(TokenKind::Newline, o)))
        }

        Some(b'!') => {
            return take(scanner::scan_single_tag_handle(&*input))(input)
                .map(|(i, o)| (i, Token::new(TokenKind::TagHandle, o)))
        }

        Some(b'%') => return single_directive(input),

        Some(b'-') => {
            if let Ok((i, t)) = dash_or_docstart(input) {
                return Ok((i, t))
            }
        }

        Some(b'?') => {
            if let Ok((i, t)) = question_mark(input) {
                return Ok((i, t))
            }
        }

        Some(b'@') => return take(1usize)(input).map(|(i, o)| (i, Token::new(TokenKind::ReservedCommercialAt, o))),
        Some(b'`') => return take(1usize)(input).map(|(i, o)| (i, Token::new(TokenKind::ReservedGraveAccent, o))),

        Some(b'\t') => return take(scanner::scan_while_tabs(&*input, 0))(input)
            .map(|(i, o)| (i, Token::new(TokenKind::Indent, o))),

        _ => (),
    };

    raw(input)
}


#[cfg(test)]
mod tests {
    use super::get_token;
    use super::super::{Token, TokenKind};

    macro_rules! init_input {
        ( $yaml_string:expr ) => {
            super::Input::new($yaml_string)
        };
    }

    macro_rules! assert_token {
        ( $value:tt, $kind:pat, $input:ident ) => {
            let value_length = $value.len();

            match get_token($input) {
                Ok((
                    i,
                    Token {
                        kind: $kind,
                        input: input,
                    },
                )) if input.fragment().len() == value_length => {
                    assert_eq!(&$value, input.fragment());
                    $input = i;
                },
                token @ _ => assert!(
                    false,
                    "Unexpected token: {:?} => {:?} , expected token: {:?} => {{ {:?}, {} }}",
                    match token {
                        Ok((_i, Token { input: input, .. })) => input.fragment(),
                        _ => ""
                    },
                    token,
                    $value,
                    stringify!($kind),
                    value_length
                ),
            };
        };
    }

    macro_rules! assert_end {
        ( $input:ident ) => {
            match get_token($input) {
                Ok((_input, token)) => {
                    assert!(false, "Unexpected token at the end: {:?}", token);
                }
                Err(nom::Err::Incomplete(nom::Needed::Unknown)) => {
                    assert!(true, "No input left");
                }
                Err(e) => {
                    assert!(false, "Expected incomplete error, got {:?}", e)
                }
            }
        };
    }


    #[test]
    fn test_tokenizer_general() {
        let src = "%YAML 1.2\n%TAG ! tag://example.com,2015:yamlette/\n---\n\"double string\"\n    \r'single string'\n\r[\"list\", 'of', tokens]\r\n{key: val, key: val} ...";

            let mut input = init_input!(src);

        assert_token!("%YAML", TokenKind::DirectiveYaml, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("1.2", TokenKind::Raw, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("%TAG", TokenKind::DirectiveTag, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("!", TokenKind::TagHandle, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("tag", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!("//example.com", TokenKind::Raw, input);
        assert_token!(",", TokenKind::Comma, input);
        assert_token!("2015", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!("yamlette/", TokenKind::Raw, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("---", TokenKind::DocumentStart, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("\"double string\"", TokenKind::StringDouble, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("    ", TokenKind::Indent, input);
        assert_token!("\r", TokenKind::Newline, input);
        assert_token!("'single string'", TokenKind::StringSingle, input);
        assert_token!("\n\r", TokenKind::Newline, input);
        assert_token!("[", TokenKind::SequenceStart, input);
        assert_token!("\"list\"", TokenKind::StringDouble, input);
        assert_token!(",", TokenKind::Comma, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("'of'", TokenKind::StringSingle, input);
        assert_token!(",", TokenKind::Comma, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("tokens", TokenKind::Raw, input);
        assert_token!("]", TokenKind::SequenceEnd, input);
        assert_token!("\r\n", TokenKind::Newline, input);
        assert_token!("{", TokenKind::DictionaryStart, input);
        assert_token!("key", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("val", TokenKind::Raw, input);
        assert_token!(",", TokenKind::Comma, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("key", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("val", TokenKind::Raw, input);
        assert_token!("}", TokenKind::DictionaryEnd, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("...", TokenKind::DocumentEnd, input);

        assert_end!(input);
    }

    #[test]
    fn test_tokenizer_anchor() {
        let src = "- &anchor string";

        let mut input = init_input!(src);

        assert_token!("-", TokenKind::Dash, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("&anchor", TokenKind::Anchor, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("string", TokenKind::Raw, input);

        assert_end!(input);
    }

    #[test]
    fn test_tokenizer_alias() {
        let src = "- *anchor string";

        let mut input = init_input!(src);

        assert_token!("-", TokenKind::Dash, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("*anchor", TokenKind::Alias, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("string", TokenKind::Raw, input);

        assert_end!(input);
    }

    #[test]
    fn test_tokenizer_e_2_1() {
        let src = r"- Mark McGwire
- Sammy Sosa
- Ken Griffey";

        let mut input = init_input!(src);

        assert_token!("-", TokenKind::Dash, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("Mark McGwire", TokenKind::Raw, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("-", TokenKind::Dash, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("Sammy Sosa", TokenKind::Raw, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("-", TokenKind::Dash, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("Ken Griffey", TokenKind::Raw, input);

        assert_end!(input);
    }

    #[test]
    fn test_tokenizer_e_2_2() {
        let src = r"hr:  65    # Home runs
avg: 0.278 # Batting average
rbi: 147   # Runs Batted In";

        let mut input = init_input!(src);

        assert_token!("hr", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("65    ", TokenKind::Raw, input);
        assert_token!("# Home runs", TokenKind::Comment, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("avg", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("0.278 ", TokenKind::Raw, input);
        assert_token!("# Batting average", TokenKind::Comment, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("rbi", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("147   ", TokenKind::Raw, input);
        assert_token!("# Runs Batted In", TokenKind::Comment, input);

        assert_end!(input);
    }


    #[test]
    fn test_tokenizer_e_2_3() {
        let src = r"american:
  - Boston Red Sox
  - Detroit Tigers
  - New York Yankees
national:
  - New York Mets
  - Chicago Cubs
  - Atlanta Braves";

        let mut input = init_input!(src);

        assert_token!("american", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("-", TokenKind::Dash, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("Boston Red Sox", TokenKind::Raw, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("-", TokenKind::Dash, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("Detroit Tigers", TokenKind::Raw, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("-", TokenKind::Dash, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("New York Yankees", TokenKind::Raw, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("national", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("-", TokenKind::Dash, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("New York Mets", TokenKind::Raw, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("-", TokenKind::Dash, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("Chicago Cubs", TokenKind::Raw, input);
        assert_token!("\n", TokenKind::Newline, input);
        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("-", TokenKind::Dash, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("Atlanta Braves", TokenKind::Raw, input);
        

        assert_end!(input);
    }

    #[test]
    fn test_tokenizer_e_2_4() {
        let src = r"-
  name: Mark McGwire
  hr:   65
  avg:  0.278
-
  name: Sammy Sosa
  hr:   63
  avg:  0.288";

        let mut input = init_input!(src);

        assert_token!("-", TokenKind::Dash, input);
        assert_token!("\n", TokenKind::Newline, input);

        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("name", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("Mark McGwire", TokenKind::Raw, input);
        assert_token!("\n", TokenKind::Newline, input);

        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("hr", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!("   ", TokenKind::Indent, input);
        assert_token!("65", TokenKind::Raw, input);
        assert_token!("\n", TokenKind::Newline, input);

        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("avg", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("0.278", TokenKind::Raw, input);
        assert_token!("\n", TokenKind::Newline, input);

        assert_token!("-", TokenKind::Dash, input);
        assert_token!("\n", TokenKind::Newline, input);

        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("name", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!(" ", TokenKind::Indent, input);
        assert_token!("Sammy Sosa", TokenKind::Raw, input);
        assert_token!("\n", TokenKind::Newline, input);

        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("hr", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!("   ", TokenKind::Indent, input);
        assert_token!("63", TokenKind::Raw, input);
        assert_token!("\n", TokenKind::Newline, input);

        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("avg", TokenKind::Raw, input);
        assert_token!(":", TokenKind::Colon, input);
        assert_token!("  ", TokenKind::Indent, input);
        assert_token!("0.288", TokenKind::Raw, input);

        assert_end!(input);
    }

}