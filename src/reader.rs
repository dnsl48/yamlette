extern crate skimmer;

use self::skimmer::{ Data, Datum, Marker, Read, Rune, Symbol };
use self::skimmer::scanner::{ scan_one_at, scan_until_at, scan_while_at };


use tokenizer::{ Token, Tokenizer };

use txt::{ Twine, Unicode };


use std::error::Error;
use std::fmt;

use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::BitXor;

use std::sync::Arc;




#[inline]
fn is<T: BitAnd<Output=T> + Eq + Copy> (state: T, val: T) -> bool { val == state & val }


#[inline]
fn not<T: BitAnd<Output=T> + Eq + Copy> (state: T, val: T) -> bool { !is (state, val) }


#[inline]
fn on<T: BitOr<Output=T> + Copy> (state: &mut T, val: T) { *state = *state | val; }


#[inline]
fn off<T: BitAnd<Output=T> + Eq + BitXor<Output=T> + Copy> (state: &mut T, val: T) { *state = *state ^ (val & *state) }



#[derive (Debug)]
pub struct ReadError {
    pub position: usize,
    pub description: Twine
}



impl fmt::Display for ReadError {
    fn fmt (&self, fmtter: &mut fmt::Formatter) -> fmt::Result {
        write! (fmtter, "{}", self.description)
    }
}



impl Error for ReadError {
    fn description (&self) -> &str {
        self.description.as_ref ()
    }
}



impl ReadError {
    pub fn new<T> (description: T) -> ReadError where T: Into<Twine> { ReadError { description: description.into (), position: 0 } }

    pub fn pos (mut self, pos: usize) -> ReadError {
        self.position = pos;
        self
    }
}




#[derive (Clone, Copy, Debug, Hash)]
pub struct Id {
    pub level: usize,
    pub parent: usize,
    pub index: usize
}




#[derive (Debug)]
pub struct Block {
    pub id: Id,
    pub cargo: BlockType
}



impl Block {
    pub fn new (id: Id, cargo: BlockType) -> Block {
        let block = Block {
            id: id,
            cargo: cargo
        };

        block
    }
}




#[derive (Debug)]
pub enum BlockType {
    Alias (Marker),

    DirectiveTag ( (Marker, Marker) ),
    DirectiveYaml ( (u8, u8) ),

    DocStart,
    DocEnd,

    BlockMap (Id, Option<Marker>, Option<Marker>),
    Literal (Marker),
    Rune (Rune, usize),

    Node (Node),

    Error (Twine, usize),
    Warning (Twine, usize),

    StreamEnd,
    Datum (Arc<Datum>)
}



#[derive (Debug)]
pub struct Node {
    pub anchor: Option<Marker>,
    pub tag: Option<Marker>,
    pub content: NodeKind
}



#[derive (Debug)]
pub enum NodeKind {
    LiteralBlockOpen,
    LiteralBlockClose,

    Mapping,
    Null,
    Scalar (Marker),
    Sequence
}



#[derive (Debug)]
enum ContextKind {
    Zero,
    Layer,
    Node,
    MappingBlock,
    MappingFlow,
    ScalarBlock,
    SequenceBlock,
    SequenceFlow
}



struct Context<'a> {
    parent: Option<&'a Context<'a>>,
    layer: usize,

    kind: ContextKind,

    level: usize,
    indent: usize,
}



impl<'a> Context<'a> {
    pub fn zero () -> Context<'a> {
        Context {
            parent: None,
            kind: ContextKind::Zero,
            layer: 0,
            level: 0,
            indent: 0,
        }
    }


    pub fn new (parent: &'a Context<'a>, kind: ContextKind, indent: usize, level: usize) -> Context<'a> {
        Context {
            kind: kind,
            layer: match parent.kind { ContextKind::Zero => 0, _ => parent.layer + 1 },
            level: level,
            indent: indent,
            parent: Some(parent)
        }
    }
}




pub struct Reader {
    index: usize,
    line: usize,
    cursor: usize,
    position: usize,
    data: Data,
    tokenizer: Tokenizer
}



impl Reader {
    pub fn new (tokenizer: Tokenizer) -> Reader {
        Reader {
            index: 0,
            line: 0,
            cursor: 0,
            position: 0,
            data: Data::with_capacity (4),
            tokenizer: tokenizer
        }
    }


    fn yield_block (&mut self, block: Block, callback: &mut FnMut (Block) -> Result<(), Twine>) -> Result<(), ReadError> {
        if let Err (error) = callback (block) {
            Err (ReadError::new (error))
        } else {
            Ok ( () )
        }
    }


    pub fn read<R: Read> (&mut self, mut reader: R, callback: &mut FnMut (Block) -> Result<(), Twine>) -> Result<(), ReadError> {
        let ctx = Context::zero ();

        let mut cur_idx = self.index;
        let result = self.read_layer (&mut reader, callback, &ctx, 0, 0, &mut cur_idx, 0, &mut None, &mut None);
        self.yield_stream_end (callback).ok ();
        result
    }


    fn get_idx (&mut self) -> usize {
        self.index += 1;
        self.index
    }


    fn nl (&mut self) {
        self.line += 1;
        self.cursor = 0;
    }


    fn skip<R: Read> (&mut self, reader: &mut R, len: usize, chars: usize) {
        self.cursor += chars;
        self.position += len;
        reader.skip (len);
    }


    fn consume<R: Read> (&mut self, reader: &mut R, callback: &mut FnMut (Block) -> Result<(), Twine>, len: usize, chars: usize) -> Result<Marker, ReadError> {
        self.cursor += chars;
        self.position += len;
        let marker = reader.consume (len);

        if marker.pos2.0 > self.data.amount () {
            for i in self.data.amount () .. marker.pos2.0 + 1 {
                let datum = reader.get_datum (i).unwrap ();
                self.yield_block (Block::new (Id { level: 0, parent: 0, index: i }, BlockType::Datum (datum.clone ())), callback) ?;
                self.data.push (reader.get_datum (i).unwrap ());
            }
        } else if self.data.amount () == 0 {
            let datum = reader.get_datum (0).unwrap ();
            self.yield_block (Block::new (Id { level: 0, parent: 0, index: 0 }, BlockType::Datum (datum.clone ())), callback) ?;
            self.data.push (datum);
        }

        Ok (marker)
    }


    fn yield_stream_end (&mut self, callback: &mut FnMut (Block) -> Result<(), Twine>) -> Result<(), ReadError> {
        self.yield_block (Block::new (Id { level: 0, parent: 0, index: 0 }, BlockType::StreamEnd), callback)
    }


    fn yield_null (&mut self, callback: &mut FnMut (Block) -> Result<(), Twine>, level: usize, parent_idx: usize) -> Result<(), ReadError> {
        let idx = self.get_idx ();
        self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
            anchor: None,
            tag: None,
            content: NodeKind::Null
        })), callback)
    }


    fn yield_error (&mut self, callback: &mut FnMut (Block) -> Result<(), Twine>, id: Id, message: Twine) -> Result<(), ReadError> {
        let pos = self.position;
        self.yield_block (Block::new (id, BlockType::Error (message.clone (), pos)), callback) ?;

        Err (ReadError::new (message).pos (pos))
    }


    fn yield_warning (&mut self, callback: &mut FnMut (Block) -> Result<(), Twine>, id: Id, message: Twine) -> Result<(), ReadError> {
        let pos = self.position;
        self.yield_block (Block::new (id, BlockType::Warning (message.clone (), pos)), callback)
    }


    fn read_layer_propagated<R: Read> (&mut self, reader: &mut R, callback: &mut FnMut (Block) -> Result<(), Twine>, ctx: &Context, level: usize, parent_idx: usize, anchor: &mut Option<Marker>, tag: &mut Option<Marker>) -> Result<(), ReadError> {
        let mut cur_idx = self.index;
        self.read_layer (reader, callback, ctx, level, parent_idx, &mut cur_idx, 15, anchor, tag) // INDENT_PASSED + INDENT_DEFINED + DIRS_PASSED
    }


    fn read_layer_expected<R: Read> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block) -> Result<(), Twine>,
        ctx: &Context,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        anchor: &mut Option<Marker>,
        tag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        self.read_layer (reader, callback, ctx, level, parent_idx, cur_idx, 3, anchor, tag)
    }


    fn read_layer<R: Read> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block) -> Result<(), Twine>,
        ctx: &Context,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        mut state: u8,
        anchor: &mut Option<Marker>,
        tag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        let ctx = Context::new (ctx, ContextKind::Layer, self.cursor, level);

        const YAML_PASSED: u8 = 1; // %YAML directive has been passed
        const DIRS_PASSED: u8 = 3; // All directives have been passed

        const INDENT_PASSED: u8 = 4; // Indentation has been passed for the line
        const INDENT_DEFINED: u8 = 8; // Indentation has been defined for the block

        const NODE_PASSED: u8 = 16; // A node has just been passed

        let mut indent: usize = self.cursor;
        let mut prev_indent = indent;

        let propagated = state > 0;
        let mut document_issued = false;


        'top: loop {
            if let Some ( (token, len, chars) ) = self.tokenizer.get_token (reader) {
                loop {
                    match token {
                        Token::BOM32BE |
                        Token::BOM32LE |
                        Token::BOM16BE |
                        Token::BOM16LE |
                        Token::BOM8 => {
                            try! (self.check_bom (reader, callback, level, parent_idx, len));
                            self.skip (reader, len, chars);
                        }


                        Token::DirectiveYaml if is (state, YAML_PASSED) => {
                            if document_issued {
                                try! (self.emit_doc_border (callback, level, parent_idx, Token::DocumentEnd));
                                self.index = 0; // reset the counter!
                                off (&mut state, DIRS_PASSED | NODE_PASSED);
                                continue;
                            }

                            let idx = self.get_idx ();
                            return self.yield_error (
                                callback,
                                Id { level: level, parent: parent_idx, index: idx },
                                Twine::from ("The YAML directive must only be given at most once per document")
                            );
                        }


                        Token::DirectiveYaml if not (state, YAML_PASSED) => {
                            self.skip (reader, len, chars);
                            try! (self.read_directive_yaml (reader, callback, level, parent_idx));
                            on (&mut state, YAML_PASSED);
                        }


                        Token::DirectiveTag if not (state, DIRS_PASSED) => {
                            self.skip (reader, len, chars);
                            try! (self.read_directive_tag (reader, callback, level, parent_idx));
                        }


                        Token::Directive if not (state, DIRS_PASSED) => {
                            let idx = self.get_idx ();
                            let line = self.line;

                            try! (self.yield_warning (
                                callback,
                                Id { level: level, parent: parent_idx, index: idx },
                                Twine::from (format! ("Unknown directive at the line {}", line))
                            ));

                            self.skip (reader, len, chars);
                        }


                        Token::DocumentStart if not (state, DIRS_PASSED) => {
                            self.skip (reader, len, chars);
                            try! (self.emit_doc_border (callback, level, parent_idx, token));
                            document_issued = true;
                            on (&mut state, DIRS_PASSED | INDENT_PASSED);
                            prev_indent = self.cursor;
                            *cur_idx = self.index;
                        }


                        Token::DocumentStart if not (state, INDENT_PASSED) => {
                            if indent > 0 { break 'top; }
                            self.skip (reader, len, chars);
                            try! (self.emit_doc_border (callback, level, parent_idx, Token::DocumentEnd));
                            document_issued = true;

                            self.index = 0; // reset the counter!
                            try! (self.emit_doc_border (callback, level, parent_idx, Token::DocumentStart));
                            prev_indent = self.cursor;
                            *cur_idx = self.index;
                        }


                        Token::Comment |
                        Token::Newline if not (state, DIRS_PASSED) => {
                            self.skip (reader, len, chars);
                            self.nl ();
                            prev_indent = self.cursor;
                        }


                        Token::Indent if not (state, DIRS_PASSED) => {
                            self.skip (reader, len, chars);
                        }


                        _ if not (state, DIRS_PASSED) => {
                            try! (self.emit_doc_border (callback, level, parent_idx, Token::DocumentStart));
                            document_issued = true;
                            on (&mut state, DIRS_PASSED);
                            *cur_idx = self.index;
                        }


                        Token::DocumentEnd if is (state, DIRS_PASSED) => {
                            self.skip (reader, len, chars);
                            try! (self.emit_doc_border (callback, level, parent_idx, token));
                            self.index = 0; // reset the counter!
                            document_issued = true;
                            state = 0;
                            prev_indent = self.cursor;
                            *cur_idx = self.index;
                        }


                        Token::Comment |
                        Token::Newline => {
                            self.skip (reader, len, chars);
                            self.nl ();
                            prev_indent = self.cursor;
                            off (&mut state, INDENT_PASSED);
                        }


                        Token::Indent /*if not (state, INDENT_PASSED)*/ => {
                            if let Some ((idx, ilen)) = scan_one_at (len, reader, &self.tokenizer.line_breakers) {
                                let bchrs = self.tokenizer.line_breakers[idx].len_chars ();
                                self.skip (reader, len + ilen, chars + bchrs);
                                self.nl ();
                                break;
                            }

                            if let Some (_) = self.tokenizer.cset.hashtag.read_at (len, reader) {
                                self.skip (reader, len, chars);
                                break;
                            }

                            if not (state, INDENT_DEFINED) {
                                self.skip (reader, len, chars);
                                indent = chars;

                                on (&mut state, INDENT_DEFINED);
                                on (&mut state, INDENT_PASSED);

                            } else if chars < indent {
                                break 'top;

                            } else if chars > indent {
                                self.skip (reader, len, chars);
                                // TODO: assess this propagation (in what cases should it work)
                                // try! (self.read_layer (&mut id.child (), len, State::new_indent_propagation ())); // propagate
                            } else {
                                self.skip (reader, len, chars);
                                on (&mut state, INDENT_PASSED);
                            }

                            prev_indent = self.cursor;
                        }


                        _ if not (state, INDENT_DEFINED) => {
                            indent = self.cursor;
                            on (&mut state, INDENT_DEFINED);
                            continue;
                        }


                        _ if not (state, INDENT_PASSED) => {
                            if indent > 0 {
                                /* zero indentation in here */
                                break 'top;
                            } else {
                                on (&mut state, INDENT_PASSED);
                                prev_indent = self.cursor;
                                continue;
                            }
                        }


                        Token::Dash => {
                            let idx = self.get_idx ();
                            *cur_idx = idx;

                            try! (self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                anchor: anchor.take (),
                                tag: tag.take (),
                                content: NodeKind::Sequence
                            })), callback));

                            try! (self.read_seq_block (reader, callback, &ctx, level + 1, idx, Some ( (token, len, chars) )));

                            if self.cursor == 0 { off (&mut state, INDENT_PASSED); }
                        }


                        Token::Colon if is (state, NODE_PASSED) => {
                            self.skip (reader, len, chars);

                            let idx = self.get_idx ();

                            try! (self.yield_block (Block::new (
                                Id { level: level, parent: parent_idx, index: idx },
                                BlockType::BlockMap (
                                    Id { level: level, parent: parent_idx, index: *cur_idx },
                                    anchor.take (),
                                    tag.take ()
                                )
                            ), callback));

                            try! (self.read_map_block_implicit (reader, callback, &ctx, level + 1, idx, prev_indent, None));

                            *cur_idx = self.index;

                            off (&mut state, NODE_PASSED);
                            off (&mut state, INDENT_PASSED);
                        }


                        Token::Question if not (state, NODE_PASSED) => {
                            let idx = self.get_idx ();

                            try! (self.yield_block (Block::new (
                                Id { level: level, parent: parent_idx, index: idx },
                                BlockType::Node (Node {
                                    anchor: anchor.take (),
                                    tag: tag.take (),
                                    content: NodeKind::Mapping
                                })
                            ), callback));

                            try! (self.read_map_block_explicit (reader, callback, &ctx, level + 1, idx, Some ( (token, len, chars) )));

                            *cur_idx = self.index;

                            off (&mut state, INDENT_PASSED);
                        }


                        _ if not (state, NODE_PASSED) => {
                            let indent = if is (state, INDENT_DEFINED) { self.cursor } else { 0 };

                            try! (self.read_node (
                                reader,
                                callback,
                                &ctx,
                                indent,
                                level,
                                parent_idx,
                                cur_idx,
                                Some ( (token, len, chars) ),
                                anchor,
                                tag
                            ));

                            on (&mut state, NODE_PASSED);
                            if self.cursor == 0 { off (&mut state, INDENT_PASSED); }
                        }

                        _ => {
                            let idx = self.get_idx ();
                            return self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Twine::from ("Unexpected token / 0001"))
                        }
                    }
                    break;
                }
            } else {
                if level == 0 && parent_idx == 0 && self.index > 0 && is (state, DIRS_PASSED) {
                    try! (self.emit_doc_border (callback, level, parent_idx, Token::DocumentEnd));
                    self.index = 0;
                } else if !propagated && !document_issued {
                    try! (self.emit_doc_border (callback, level, parent_idx, Token::DocumentStart));
                    try! (self.emit_doc_border (callback, level, parent_idx, Token::DocumentEnd));
                    self.index = 0;
                }

                break;
            }
        }

        Ok ( () )
    }


    fn read_scalar_block_literal<R: Read> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block) -> Result<(), Twine>,
        _ctx: &Context,
        level: usize,
        parent_idx: usize,
        mut accel: Option<(Token, usize, usize)>,
        mut state: u8,
        mut indent: usize
    ) -> Result<(), ReadError> {
        const INDENT_DEFINED: u8 = 1;
        const INDENT_PASSED: u8 = 2;
        const CHOMP_STRIP: u8 = 4;
        const CHOMP_KEEP: u8 = 8;

        const HUNGRY: u8 = 32;
        const KEEPER: u8 = 64;
        const ALWAYS_KEEP: u8 = 128; // literal mode

        let mut lazy_indent: usize = 0;
        let mut lazy_nl: Option<Marker> = None;
        let mut lazy_tail: Option<(Marker, usize)> = None;

        'top: loop {
            if let Some ( (token, len, chars) ) = if accel.is_none () { self.tokenizer.get_token (reader) } else { accel.take () } {
                loop {
                    match token {
                        Token::BOM32BE |
                        Token::BOM32LE |
                        Token::BOM16BE |
                        Token::BOM16LE |
                        Token::BOM8 => {
                            try! (self.check_bom (reader, callback, level, parent_idx, len));
                            self.skip (reader, len, chars);
                        }


                        _ if is (state, ALWAYS_KEEP) && not (state, KEEPER) => {
                            on (&mut state, KEEPER);
                            continue;
                        }


                        Token::Newline if not (state, INDENT_DEFINED) => {
                            lazy_indent = 0;

                            if lazy_tail.is_some () {
                                let (chunk, _) = lazy_tail.take ().unwrap ();
                                let idx = self.get_idx ();
                                try! (self.yield_block (Block::new (
                                    Id { level: level, parent: parent_idx, index: idx },
                                    BlockType::Literal (chunk)
                                ), callback));
                            }

                            lazy_tail = Some ( (self.consume (reader, callback, len, chars) ?, chars) );

                            on (&mut state, KEEPER);
                            off (&mut state, INDENT_PASSED);
                        }


                        Token::Indent if not (state, INDENT_DEFINED) => {
                            self.skip (reader, len, chars);
                            lazy_indent = chars;
                            on (&mut state, INDENT_PASSED);
                        }


                        _ if not (state, INDENT_DEFINED) && lazy_indent >= indent && match token { Token::Newline => false, Token::Indent => false, _ => true } => {
                            indent = lazy_indent;
                            on (&mut state, INDENT_DEFINED | INDENT_PASSED);
                            continue;
                        }


                        Token::Indent if not (state, INDENT_PASSED) => {
                            if chars < indent { break 'top; }

                            /* Do not skip more than indent in here! */
                            let slen = self.tokenizer.spaces[0].len ();
                            self.skip (reader, slen * indent, indent);

                            on (&mut state, INDENT_PASSED);
                        }


                        _ if not (state, INDENT_PASSED) => {
                            if indent == 0 {
                                on (&mut state, INDENT_PASSED);
                                continue;
                            } else {
                                break 'top
                            }
                        }


                        Token::Tab    |
                        Token::Indent if not (state, KEEPER) => {
                            on (&mut state, KEEPER);
                            continue;
                        }


                        _ if lazy_nl.is_some () && is (state, KEEPER) => {
                            let chunk = lazy_nl.take ().unwrap ();

                            let idx = self.get_idx ();
                            try! (self.yield_block (Block::new (
                                Id { level: level, parent: parent_idx, index: idx },
                                BlockType::Literal (chunk)
                            ), callback));
                            off (&mut state, HUNGRY | KEEPER);

                            continue;
                        }


                        _ if lazy_tail.is_some () => {
                            let (_, nls) = lazy_tail.take ().unwrap ();
                            // let mut chunk = Chunk::with_capacity (self.tokenizer.cset.line_feed.len () * nls);
                            // for _ in 0 .. nls { chunk.push_slice (self.tokenizer.cset.line_feed.as_slice ()); }
                            
                            let idx = self.get_idx ();
                            let rune = Rune::from (self.tokenizer.cset.line_feed.clone ());
                            try! (self.yield_block (Block::new (
                                Id { level: level, parent: parent_idx, index: idx },
                                BlockType::Rune (rune, nls)
                                // BlockType::Literal (chunk)
                            ), callback));
                            off (&mut state, HUNGRY | KEEPER);
                            continue;
                        }


                        _ if is (state, HUNGRY) => {
                            // TODO: try to take a part of the indentation instead of making a new chunk

                            // let mut chunk: Chunk;
                            let rune: Rune;

                            if is (state, KEEPER) {
                                // chunk = Chunk::with_capacity (self.tokenizer.cset.line_feed.len ());
                                // chunk.push_slice (self.tokenizer.cset.line_feed.as_slice ());
                                rune = Rune::from (self.tokenizer.cset.line_feed.clone ());
                            } else {
                                // chunk = Chunk::with_capacity (self.tokenizer.spaces[0].len ());
                                // chunk.push_slice (self.tokenizer.spaces[0].as_slice ());
                                rune = Rune::from (self.tokenizer.spaces[0].clone ());
                            }

                            let idx = self.get_idx ();
                            try! (self.yield_block (Block::new (
                                Id { level: level, parent: parent_idx, index: idx },
                                BlockType::Rune (rune, 1)
                                // BlockType::Literal (chunk)
                            ), callback));

                            off (&mut state, HUNGRY | KEEPER);

                            continue;
                        }


                        _ => {
                            let len = self.tokenizer.line (reader);

                            let nl = if let Some ( (_, len) ) = scan_one_at (len, reader, &self.tokenizer.line_breakers) {
                                len
                            } else { 0 };

                            let mut tail: usize = 0;
                            let mut tail_nls: usize = 0;
                            lazy_nl = None;
                            lazy_tail = None;

                            if nl > 0 {
                                let mut spaces_add_bytes: usize = 0;
                                let mut spaces_add_chars: usize = 0;

                                loop {
                                    if let Some ( (idx, len) ) = scan_one_at (len + nl + tail + spaces_add_bytes, reader, &self.tokenizer.spaces) {
                                        spaces_add_bytes += len;
                                        spaces_add_chars += self.tokenizer.spaces[idx].len_chars ();
                                        continue;
                                    }


                                    if let Some ( (_, len) ) = scan_one_at (len + nl + tail + spaces_add_bytes, reader, &self.tokenizer.line_breakers) {
                                        if spaces_add_chars > indent {
                                            break;
                                        } else {
                                            tail += spaces_add_bytes + len;
                                            tail_nls += 1;
                                            spaces_add_bytes = 0;
                                            spaces_add_chars = 0;

                                            continue;
                                        }
                                    }

                                    break;
                                }
                            }


                            if len > 0 {
                                let marker = self.consume (reader, callback, len, 0) ?;
                                let idx = self.get_idx ();
                                try! (self.yield_block (Block::new (
                                    Id { level: level, parent: parent_idx, index: idx },
                                    BlockType::Literal (marker)
                                ), callback));
                            }


                            if nl > 0 {
                                self.nl ();

                                if is (state, INDENT_DEFINED) {
                                    lazy_nl = Some (self.consume (reader, callback, nl, 0) ?);

                                    if tail > 0 {
                                        lazy_tail = Some ( (self.consume (reader, callback, tail, 0) ?, tail_nls) );
                                        for _ in 0..tail_nls { self.nl (); }
                                    }
                                } else {
                                    self.skip (reader, nl + tail_nls, 0);
                                    if tail > 0 {
                                        for _ in 0..tail_nls { self.nl (); }
                                    }
                                }
                            }

                            on (&mut state, HUNGRY);
                            off (&mut state, INDENT_PASSED);
                        }
                    }

                    break;
                }
            } else { break; }
        }


        if not (state, CHOMP_STRIP) {
            if lazy_nl.is_some () {
                let idx = self.get_idx ();

                try! (self.yield_block (Block::new (
                    Id { level: level, parent: parent_idx, index: idx },
                    BlockType::Literal (lazy_nl.take ().unwrap ())
                ), callback));

                if is (state, CHOMP_KEEP) && lazy_tail.is_some () {
                    let idx = self.get_idx ();
                    try! (self.yield_block (Block::new (
                        Id { level: level, parent: parent_idx, index: idx },
                        BlockType::Literal (lazy_tail.take ().unwrap ().0)
                    ), callback));
                }
            } else if lazy_tail.is_some () {
                let (_, chars) = lazy_tail.take ().unwrap ();

                if is (state, CHOMP_KEEP) && chars > 0 {
                    // let mut chunk = Chunk::with_capacity (self.tokenizer.cset.line_feed.len () * chars);
                    // for _ in 0..chars { chunk.push_slice (self.tokenizer.cset.line_feed.as_slice ()); }

                    let idx = self.get_idx ();
                    let rune = Rune::from (self.tokenizer.cset.line_feed.clone ());
                    try! (self.yield_block (Block::new (
                        Id { level: level, parent: parent_idx, index: idx },
                        BlockType::Rune (rune, chars)
                        // BlockType::Literal (chunk)
                    ), callback));
                }
            }
        }

        Ok ( () )
    }


    fn read_scalar_block<R: Read> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block) -> Result<(), Twine>,
        ctx: &Context,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        mut accel: Option<(Token, usize, usize)>,
        anchor: Option<Marker>,
        tag: Option<Marker>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        let ctx = Context::new (ctx, ContextKind::ScalarBlock, self.cursor, level);

        const CHOMP_STRIP: u8 = 1;
        const CHOMP_KEEP: u8 = 2;
        const CHOMP_DEFINED: u8 = 4;

        const FOLDED: u8 = 8;
        const TYPE_DEFINED: u8 = 16;
        const HEAD_PASSED: u8 = 32;
        const INDENT_DEFINED: u8 = 64;

        let mut indent: usize = 0;
        let default_indent: usize = if self.cursor > 0 { 1 } else { 0 };
        let mut state: u8 = 0;

        let mut idx = 0;

        loop {
            if let Some ( (token, len, chars) ) = if accel.is_none () { self.tokenizer.get_token (reader) } else { accel.take () } {
                loop {
                    match token {
                        Token::BOM32BE |
                        Token::BOM32LE |
                        Token::BOM16BE |
                        Token::BOM16LE |
                        Token::BOM8 => {
                            try! (self.check_bom (reader, callback, level, parent_idx, len));
                            self.skip (reader, len, chars);
                        }

                        Token::GT |
                        Token::Pipe if not (state, HEAD_PASSED) && not (state, TYPE_DEFINED) => {
                            self.skip (reader, len, chars);
                            on (&mut state, match token { Token::GT => FOLDED, _ => 0 } | TYPE_DEFINED);
                        }

                        Token::Comment |
                        Token::Tab |
                        Token::Indent if not (state, HEAD_PASSED) => self.skip (reader, len, chars),

                        Token::Newline if not (state, HEAD_PASSED) => {
                            if let Some ((_, ilen)) = scan_one_at (0, reader, &self.tokenizer.line_breakers) {
                                self.skip (reader, ilen, 1);
                            }

                            self.nl ();

                            idx = self.get_idx ();

                            *cur_idx = idx;

                            try! (self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                anchor: None,
                                tag: None,
                                content: NodeKind::LiteralBlockOpen
                            })), callback));

                            on (&mut state, HEAD_PASSED);
                        }

                        Token::Dash if not (state, HEAD_PASSED) && not (state, CHOMP_DEFINED) => {
                            self.skip (reader, len, chars);
                            on (&mut state, CHOMP_DEFINED | CHOMP_STRIP);
                        }

                        Token::Raw if not (state, HEAD_PASSED) && not (state, CHOMP_DEFINED) && not (state, INDENT_DEFINED) => {
                            let marker = self.consume (reader, callback, len, chars) ?;
                            let chunk = self.data.chunk (&marker);
                            let chunk_slice = chunk.as_slice ();

                            let mut pos: usize = 0;

                            loop {
                                if not (state, CHOMP_DEFINED) {
                                    if self.tokenizer.cset.hyphen_minus.contained_at (chunk_slice, pos) {
                                        pos += self.tokenizer.cset.hyphen_minus.len ();
                                        on (&mut state, CHOMP_DEFINED | CHOMP_STRIP);
                                        if indent > 0 { on (&mut state, INDENT_DEFINED) }
                                    } else if self.tokenizer.cset.plus.contained_at (chunk_slice, pos) {
                                        pos += self.tokenizer.cset.plus.len ();
                                        on (&mut state, CHOMP_DEFINED | CHOMP_KEEP);
                                        if indent > 0 { on (&mut state, INDENT_DEFINED) }
                                    }
                                }

                                if not (state, INDENT_DEFINED) {
                                    if let Some ( (n, p) ) = self.tokenizer.cset.extract_dec_at (chunk_slice, pos) {
                                        pos += p;
                                        indent = indent * 10 + (n as usize);

                                        continue;
                                    } else if indent > 0 {
                                        on (&mut state, INDENT_DEFINED);
                                    }
                                }

                                if not (state, CHOMP_DEFINED) {
                                    if self.tokenizer.cset.hyphen_minus.contained_at (chunk_slice, pos) {
                                        on (&mut state, CHOMP_DEFINED | CHOMP_STRIP);
                                        if indent > 0 { on (&mut state, INDENT_DEFINED) }
                                    } else if self.tokenizer.cset.plus.contained_at (chunk_slice, pos) {
                                        on (&mut state, CHOMP_DEFINED | CHOMP_KEEP);
                                        if indent > 0 { on (&mut state, INDENT_DEFINED) }
                                    }
                                }

                                break;
                            }
                        }

                        _ if not (state, HEAD_PASSED) => {
                            let idx = self.get_idx ();
                            return self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Twine::from ("Unexpected token / 0002"))
                        }

                        _ if not (state, FOLDED) => return self.read_scalar_block_literal (
                            reader,
                            callback,
                            &ctx,
                            level + 1,
                            idx,
                            Some ( (token, len, chars) ),
                            if is (state, INDENT_DEFINED) { 1 } else { 0 } |
                            if is (state, CHOMP_DEFINED) {
                                if is (state, CHOMP_STRIP) { 4 }
                                else if is (state, CHOMP_KEEP) { 8 }
                                else { 0 }
                            } else { 0 } | 128,
                            if is (state, INDENT_DEFINED) { indent } else { default_indent }
                        ).and_then (| () | {
                            let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                                (anchor, tag)
                            } else {
                                (
                                    if anchor.is_none () { overanchor.take () } else { anchor },
                                    if tag.is_none () { overtag.take () } else { tag }
                                )
                            };

                            self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                anchor: anchor,
                                tag: tag,
                                content: NodeKind::LiteralBlockClose
                            })), callback)
                        }),

                        _ => return self.read_scalar_block_literal (
                            reader,
                            callback,
                            &ctx,
                            level + 1,
                            idx,
                            Some ( (token, len, chars) ),
                            if is (state, INDENT_DEFINED) { 1 } else { 0 } |
                            if is (state, CHOMP_DEFINED) {
                                if is (state, CHOMP_STRIP) { 4 }
                                else if is (state, CHOMP_KEEP) { 8 }
                                else { 0 }
                            } else { 0 },
                            if is (state, INDENT_DEFINED) { indent } else { default_indent }
                        ).and_then (| () | {
                            let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                                (anchor, tag)
                            } else {
                                (
                                    if anchor.is_none () { overanchor.take () } else { anchor },
                                    if tag.is_none () { overtag.take () } else { tag }
                                )
                            };

                            self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                anchor: anchor,
                                tag: tag,
                                content: NodeKind::LiteralBlockClose
                            })), callback)
                        })
                    }
                    break;
                }
            } else { break; }
        }

        Ok ( () )
    }


    fn read_seq_block<R: Read> (&mut self, reader: &mut R, callback: &mut FnMut (Block) -> Result<(), Twine>, ctx: &Context, level: usize, parent_idx: usize, mut accel: Option<(Token, usize, usize)>) -> Result<(), ReadError> {
        let ctx = Context::new (ctx, ContextKind::SequenceBlock, self.cursor, level);

        const INDENT_PASSED: u8 = 1; // Indentation has been passed for the line
        const NODE_READ: u8 = 2;

        let mut state: u8 = INDENT_PASSED;
        let indent = self.cursor;

        let mut prev_indent = indent;

        'top: loop {
            if let Some ( (token, len, chars) ) = if accel.is_none () { self.tokenizer.get_token (reader) } else { accel.take () } {
                loop {
                    match token {
                        Token::BOM32BE |
                        Token::BOM32LE |
                        Token::BOM16BE |
                        Token::BOM16LE |
                        Token::BOM8 => {
                            try! (self.check_bom (reader, callback, level, parent_idx, len));
                            self.skip (reader, len, chars);
                        }

                        Token::Comment |
                        Token::Newline => {
                            self.skip (reader, len, chars);
                            self.nl ();
                            prev_indent = self.cursor;
                            off (&mut state, INDENT_PASSED);
                            off (&mut state, NODE_READ);
                        }

                        Token::Indent if not (state, INDENT_PASSED) => {
                            // TODO: check for a newline right after the indent and continue in that case
                            // scan_one_at

                            if let Some ((idx, ilen)) = scan_one_at (len, reader, &self.tokenizer.line_breakers) {
                                let bchrs = self.tokenizer.line_breakers[idx].len_chars ();
                                self.skip (reader, len + ilen, chars + bchrs);
                                self.nl ();
                                break;
                            }

                            if let Some (_) = self.tokenizer.cset.hashtag.read_at (len, reader) {
                                self.skip (reader, len, chars);
                                break;
                            }


                            if chars < indent {
                                break 'top
                            } else if chars > indent {
                                self.skip (reader, len, chars);
                                // TODO: assess this propagation (in what cases should it work)
                                // try! (self.read_layer (&mut id.child (), len, State::new_indent_propagation ())); // propagate
                            } else {
                                self.skip (reader, len, chars);
                                on (&mut state, INDENT_PASSED);
                            }

                            prev_indent = self.cursor;
                        }

                        _ if not (state, INDENT_PASSED) => {
                            if indent > 0 {
                                break 'top;
                            } else {
                                prev_indent = self.cursor;
                                on (&mut state, INDENT_PASSED);
                                continue;
                            }
                        }

                        Token::Dash if is (state, INDENT_PASSED) => {
                            try! (self.read_seq_block_item (reader, callback, &ctx, level, parent_idx, &mut prev_indent, Some ( (token, len, chars) )));
                            if self.cursor == 0 { off (&mut state, INDENT_PASSED); }
                            on (&mut state, NODE_READ);
                        }

                        Token::Colon if is (state, NODE_READ) => {
                            self.skip (reader, len, chars);

                            let cur_idx = self.index;
                            let idx = self.get_idx ();

                            try! (self.yield_block (Block::new (
                                Id { level: level, parent: parent_idx, index: idx },
                                BlockType::BlockMap (Id { level: level, parent: parent_idx, index: cur_idx }, None, None)
                            ), callback));

                            try! (self.read_map_block_implicit (reader, callback, &ctx, level + 1, idx, prev_indent, None));

                            off (&mut state, NODE_READ);
                            if self.cursor == 0 { off (&mut state, INDENT_PASSED); }
                        }

                        _ => break 'top
                    }
                    break;
                }
            } else { break; }
        }

        Ok ( () )
    }



    fn read_seq_flow<R: Read> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block) -> Result<(), Twine>,
        ctx: &Context,
        idx: usize,
        level: usize,
        parent_idx: usize,
        indent: usize,
        anchor: Option<Marker>,
        tag: Option<Marker>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        let ctx = Context::new (ctx, ContextKind::SequenceFlow, self.cursor, level);

        const NODE_PASSED: u8 = 1; // Indentation has been passed for the line
        const MAP_KEY_PASSED: u8 = 2;
        const COLON_IS_RAW: u8 = 4;

        let mut state: u8 = 0;
        let mut cur_idx = idx;

        'top: loop {
            if let Some ( (token, len, chars) ) = self.tokenizer.get_token (reader) {
                loop {
                    match token {
                        Token::BOM32BE |
                        Token::BOM32LE |
                        Token::BOM16BE |
                        Token::BOM16LE |
                        Token::BOM8 => {
                            try! (self.check_bom (reader, callback, level + 1, idx, len));
                            self.skip (reader, len, chars);
                        }


                        Token::SequenceEnd => {
                            self.skip (reader, len, chars);

                            let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                                (anchor, tag)
                            } else {
                                (
                                    if anchor.is_none () { overanchor.take () } else { anchor },
                                    if tag.is_none () { overtag.take () } else { tag }
                                )
                            };

                            try! (self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                anchor: anchor,
                                tag: tag,
                                content: NodeKind::Sequence
                            })), callback));

                            break 'top;
                        }


                        Token::Comment |
                        Token::Newline => {
                            self.skip (reader, len, chars);
                            self.nl ();
                        }


                        Token::Tab    |
                        Token::Indent => self.skip (reader, len, chars),


                        Token::Comma if is (state, MAP_KEY_PASSED) => {
                            try! (self.yield_null (callback, level + 2, cur_idx));
                            off (&mut state, MAP_KEY_PASSED | NODE_PASSED);
                        }


                        Token::Comma if is (state, NODE_PASSED) => {
                            self.skip (reader, len, chars);
                            off (&mut state, NODE_PASSED);
                        }


                        Token::Question if not (state, NODE_PASSED)  => {
                            self.skip (reader, len, chars);

                            let new_idx = self.get_idx ();

                            try! (self.yield_block (Block::new (
                                Id { level: level + 1, parent: idx, index: new_idx },
                                BlockType::Node (Node {
                                    anchor: None,
                                    tag: None,
                                    content: NodeKind::Mapping
                                })
                            ), callback));

                            cur_idx = new_idx;
                            let mut cur_idx_tmp = cur_idx;
                            try! (self.read_node_flow (reader, callback, &ctx, indent, level + 2, new_idx, &mut cur_idx_tmp, None, &mut None, &mut None));

                            on (&mut state, MAP_KEY_PASSED);
                        }


                        Token::Colon if not (state, MAP_KEY_PASSED) && is (state, NODE_PASSED) && not (state, COLON_IS_RAW) => {
                            self.skip (reader, len, chars);

                            let new_idx = self.get_idx ();

                            try! (self.yield_block (Block::new (
                                Id { level: level + 1, parent: idx, index: new_idx },
                                BlockType::BlockMap (
                                    Id { level: level + 1, parent: idx, index: cur_idx },
                                    None,
                                    None
                                )
                            ), callback));

                            cur_idx = new_idx;
                            try! (self.read_node_flow (reader, callback, &ctx, indent, level + 2, new_idx, &mut cur_idx, None, &mut None, &mut None));
                        }


                        Token::Colon if is (state, MAP_KEY_PASSED) && not (state, COLON_IS_RAW) => {
                            self.skip (reader, len, chars);
                            try! (self.read_node_flow (reader, callback, &ctx, indent, level + 2, cur_idx, &mut cur_idx, None, &mut None, &mut None));
                            off (&mut state, MAP_KEY_PASSED | COLON_IS_RAW);
                            on (&mut state, NODE_PASSED);
                        }


                        Token::Colon if not (state, MAP_KEY_PASSED) && not (state, NODE_PASSED) && not (state, COLON_IS_RAW) => {
                            if let None = scan_one_at (len, reader, &self.tokenizer.spaces_and_line_breakers) {
                                on (&mut state, COLON_IS_RAW);
                                continue;
                            }

                            self.skip (reader, len, chars);

                            let new_idx = self.get_idx ();

                            try! (self.yield_block (Block::new (
                                Id { level: level + 1, parent: idx, index: new_idx },
                                BlockType::Node (Node {
                                    anchor: None,
                                    tag: None,
                                    content: NodeKind::Mapping
                                })
                            ), callback));

                            try! (self.yield_null (callback, level + 2, new_idx));
                            try! (self.read_node_flow (reader, callback, &ctx, indent, level + 2, new_idx, &mut cur_idx, None, &mut None, &mut None));

                            cur_idx = new_idx;

                            off (&mut state, MAP_KEY_PASSED | NODE_PASSED);
                        }


                        _ => {
                            let indent = self.cursor;
                            try! (self.read_node_flow (reader, callback, &ctx, indent, level + 1, idx, &mut cur_idx, Some ( (token, len, chars) ), &mut None, &mut None));
                            on (&mut state, NODE_PASSED);
                            off (&mut state, COLON_IS_RAW);
                        }
                    }
                    break;
                }
            } else { break; }
        }

        Ok ( () )
    }


    fn read_map_block_explicit<R: Read> (&mut self, reader: &mut R, callback: &mut FnMut (Block) -> Result<(), Twine>, ctx: &Context, level: usize, parent_idx: usize, accel: Option<(Token, usize, usize)>) -> Result<(), ReadError> {
        self.read_map_block (reader, callback, ctx, level, parent_idx, 1, accel, None)
    }

    fn read_map_block_implicit<R: Read> (&mut self, reader: &mut R, callback: &mut FnMut (Block) -> Result<(), Twine>, ctx: &Context, level: usize, parent_idx: usize, indent: usize, accel: Option<(Token, usize, usize)>) -> Result<(), ReadError> {
        self.read_map_block (reader, callback, ctx, level, parent_idx, 15, accel, Some (indent))
    }


    fn read_map_block<R: Read> (&mut self, reader: &mut R, callback: &mut FnMut (Block) -> Result<(), Twine>, ctx: &Context, level: usize, parent_idx: usize, mut state: u8, mut accel: Option<(Token, usize, usize)>, indent: Option<usize>) -> Result<(), ReadError> {
        let ctx = Context::new (ctx, ContextKind::MappingBlock, if indent.is_some () { *indent.as_ref ().unwrap () } else { self.cursor }, level);
        const INDENT_PASSED: u8 = 1;
        const QST_PASSED: u8 = 2;
        const KEY_PASSED: u8 = 6;
        const SEP_PASSED: u8 = 14;
        const VAL_PASSED: u8 = 30;
        const QST_EXPLICIT: u8 = 32;

        let mut qst_explicit_line: usize = 0;
        let mut last_key_idx: usize = 0;
        let mut last_val_idx: usize;

        let indent = if indent.is_some () { indent.unwrap () } else { self.cursor };

        'top: loop {
            if let Some ( (token, len, chars) ) = if accel.is_none () { self.tokenizer.get_token (reader) } else { accel.take () } {
                loop {
                    match token {
                        Token::BOM32BE |
                        Token::BOM32LE |
                        Token::BOM16BE |
                        Token::BOM16LE |
                        Token::BOM8 => {
                            try! (self.check_bom (reader, callback, level, parent_idx, len));
                            self.skip (reader, len, chars);
                        }

                        Token::Comment |
                        Token::Newline => {
                            self.skip (reader, len, chars);
                            self.nl ();

                            off (&mut state, INDENT_PASSED);
                            if is (state, VAL_PASSED) { off (&mut state, VAL_PASSED); }
                        }

                        Token::Indent if not (state, INDENT_PASSED) => {
                            // TODO: check for a newline right after the indent and continue in that case
                            if let Some ((idx, ilen)) = scan_one_at (len, reader, &self.tokenizer.line_breakers) {
                                let bchrs = self.tokenizer.line_breakers[idx].len_chars ();
                                self.skip (reader, len + ilen, chars + bchrs);
                                self.nl ();
                                break;
                            }

                            if let Some (_) = self.tokenizer.cset.hashtag.read_at (len, reader) {
                                self.skip (reader, len, chars);
                                break;
                            }

                            if chars < indent {
                                break 'top

                            } else if chars > indent {
                                self.skip (reader, len, chars);
                                try! (self.read_layer_propagated (reader, callback, &ctx, level, parent_idx, &mut None, &mut None));
                                if self.cursor == 0 { off (&mut state, INDENT_PASSED); }
                                if is (state, SEP_PASSED) { off (&mut state, SEP_PASSED); }

                            } else {
                                self.skip (reader, len, chars);
                                if is (state, SEP_PASSED) {
                                    try! (self.yield_null (callback, level, parent_idx));
                                    off (&mut state, VAL_PASSED);
                                }
                                on (&mut state, INDENT_PASSED);
                            }
                        }


                        Token::DocumentEnd |
                        Token::DocumentStart if not (state, INDENT_PASSED) && not (state, KEY_PASSED) => {
                            break 'top
                        }


                        Token::Colon if is (state, KEY_PASSED) && not (state, SEP_PASSED) => {
                            if is (state, QST_EXPLICIT) && qst_explicit_line == self.line {
                                let (len_, _) = scan_while_at (len, reader, &self.tokenizer.spaces_and_tabs);
                                let (len__, idx) = scan_until_at (len + len_, reader, &self.tokenizer.colon_and_line_breakers);

                                if let Some ( (idx, _) ) = idx {
                                    if idx > 0 {
                                        let (len___, _) = scan_while_at (len + len_ + len__, reader, &self.tokenizer.spaces_and_line_breakers);
                                        if let Some ( (0, _) ) = scan_one_at (len + len_ + len__ + len___, reader, &self.tokenizer.colon_and_line_breakers) {
                                            self.skip (reader, len + len_, chars);

                                            let idx = self.get_idx ();

                                            try! (self.yield_block (Block::new (
                                                Id { level: level, parent: parent_idx, index: idx },
                                                BlockType::BlockMap (
                                                    Id { level: level, parent: parent_idx, index: last_key_idx },
                                                    None,
                                                    None
                                                )
                                            ), callback));

                                            let mut cur_idx = self.index;

                                            try! (self.read_node_mblockval (reader, callback, &ctx, indent + 1, level + 1, idx, &mut cur_idx, None, &mut None, &mut None));
                                            off (&mut state, VAL_PASSED | QST_EXPLICIT);
                                            if self.cursor == 0 { off (&mut state, INDENT_PASSED); }
                                            break;
                                        }
                                    }
                                }
                            }

                            self.skip (reader, len, chars);
                            on (&mut state, SEP_PASSED);
                            on (&mut state, INDENT_PASSED);
                        }

                        Token::Dash if is (state, SEP_PASSED) && self.cursor >= indent => {
                            let indent = self.cursor;
                            let mut cur_idx = self.index;
                            try! (self.read_node_mblockval (reader, callback, &ctx, indent, level, parent_idx, &mut cur_idx, Some ( (token, len, chars) ), &mut None, &mut None));
                            off (&mut state, VAL_PASSED);
                            if self.cursor == 0 { off (&mut state, INDENT_PASSED); }
                        }

                        _ if not (state, INDENT_PASSED) => {
                            if indent > 0 {
                                break 'top

                            } else {
                                if is (state, SEP_PASSED)  || is (state, QST_PASSED) {
                                    try! (self.yield_null (callback, level, parent_idx));
                                    off (&mut state, VAL_PASSED);
                                }
                                on (&mut state, INDENT_PASSED);
                                continue;
                            }
                        }

                        Token::Tab    |
                        Token::Indent => self.skip (reader, len, chars),

                        Token::Question if not (state, QST_PASSED) => {
                            self.skip (reader, len, chars);
                            qst_explicit_line = self.line;
                            on (&mut state, QST_PASSED | QST_EXPLICIT);
                        }

                        Token::Question if not (state, SEP_PASSED) => {
                            try! (self.yield_null (callback, level, parent_idx));
                            self.skip (reader, len, chars);
                            qst_explicit_line = self.line;
                            off (&mut state, VAL_PASSED);
                            on (&mut state, QST_PASSED | QST_EXPLICIT);
                        }

                        _ if not (state, KEY_PASSED) => {
                            let indent = self.cursor;
                            let mut cur_idx = self.index;
                            last_key_idx = cur_idx + 1;
                            try! (self.read_node_mblockval (reader, callback, &ctx, indent, level, parent_idx, &mut cur_idx, Some ( (token, len, chars) ), &mut None, &mut None));
                            on (&mut state, KEY_PASSED);
                            if self.cursor == 0 { off (&mut state, INDENT_PASSED); }
                        }

                        _ if is (state, SEP_PASSED) && not (state, VAL_PASSED) => {
                            let indent = self.cursor;
                            let mut cur_idx = self.index;
                            let lid = self.line;
                            last_val_idx = cur_idx + 1;

                            try! (self.read_node_mblockval (reader, callback, &ctx, indent, level, parent_idx, &mut cur_idx, Some ( (token, len, chars) ), &mut None, &mut None));
                            off (&mut state, VAL_PASSED);
                            if self.tokenizer.cset.colon.read (reader).is_some () { on (&mut state, QST_PASSED); }
                            if self.cursor == 0 { off (&mut state, INDENT_PASSED); }

                            if lid < self.line { break; }

                            'inliner: loop {
                                if let Some (len1_colon) = self.tokenizer.cset.colon.read (reader) {
                                    let (len2_space, _) = scan_while_at (len1_colon, reader, &self.tokenizer.spaces_and_tabs);
                                    let (_, idx) = scan_until_at (len1_colon + len2_space, reader, &self.tokenizer.question_and_line_breakers);

                                    if let Some ( (idx, _) ) = idx {
                                        if idx > 0 {
                                            self.skip (reader, len1_colon + len2_space, 0);

                                            let idx = self.get_idx ();

                                            try! (self.yield_block (Block::new (
                                                Id { level: level, parent: parent_idx, index: idx },
                                                BlockType::BlockMap (
                                                    Id { level: level, parent: parent_idx, index: last_val_idx },
                                                    None,
                                                    None
                                                )
                                            ), callback));

                                            let mut cur_idx = self.index;

                                            try! (self.read_node_mblockval (reader, callback, &ctx, indent + 1, level + 1, idx, &mut cur_idx, None, &mut None, &mut None));
                                            off (&mut state, VAL_PASSED | QST_EXPLICIT);
                                            if self.cursor == 0 { off (&mut state, INDENT_PASSED); }
                                        }
                                    }
                                }
                                break;
                            }
                        }

                        _ => {
                            let idx = self.get_idx ();
                            return self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Twine::from ("Unexpected token / 0003"))
                        }
                    }

                    break;
                }
            } else { break; }
        }

        Ok ( () )
    }


    fn read_map_flow<R: Read> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block) -> Result<(), Twine>,
        ctx: &Context,
        idx: usize,
        level: usize,
        parent_idx: usize,
        indent: usize,
        anchor: Option<Marker>,
        tag: Option<Marker>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        let ctx = Context::new (ctx, ContextKind::MappingFlow, self.cursor, level);

        const QST_PASSED: u8 = 1;
        const KEY_PASSED: u8 = 3;
        const SEP_PASSED: u8 = 4;
        const VAL_PASSED: u8 = 15;

        let mut state: u8 = 0;

        loop {
            if let Some ( (token, len, chars) ) = self.tokenizer.get_token (reader) {
                match token {
                    Token::BOM32BE |
                    Token::BOM32LE |
                    Token::BOM16BE |
                    Token::BOM16LE |
                    Token::BOM8 => {
                        try! (self.check_bom (reader, callback, level + 1, idx, len));
                        self.skip (reader, len, chars);
                    }

                    Token::DictionaryEnd => {
                        self.skip (reader, len, chars);

                        if is (state, QST_PASSED) && not (state, KEY_PASSED) {
                            try! (self.yield_null (callback, level + 1, idx));
                        }

                        if is (state, QST_PASSED) && not (state, VAL_PASSED) {
                            try! (self.yield_null (callback, level + 1, idx));
                        }

                        let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                            (anchor, tag)
                        } else {
                            (
                                if anchor.is_none () { overanchor.take () } else { anchor },
                                if tag.is_none () { overtag.take () } else { tag }
                            )
                        };

                        try! (self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                            anchor: anchor,
                            tag: tag,
                            content: NodeKind::Mapping
                        })), callback));

                        break;
                    }

                    Token::Comment |
                    Token::Newline => {
                        self.skip (reader, len, chars);
                        self.nl ();
                    }

                    Token::Tab    |
                    Token::Indent => self.skip (reader, len, chars),

                    Token::Question if not (state, QST_PASSED) => {
                        self.skip (reader, len, chars);
                        on (&mut state, QST_PASSED);
                    }

                    Token::Comma if is (state, VAL_PASSED) => {
                        self.skip (reader, len, chars);
                        off (&mut state, VAL_PASSED);
                    }

                    Token::Colon if is (state, KEY_PASSED) && not (state, SEP_PASSED) => {
                        self.skip (reader, len, chars);
                        on (&mut state, SEP_PASSED);
                    }

                    Token::Comma if is (state, KEY_PASSED) /*&& not (state, SEP_PASSED)*/ => {
                        self.skip (reader, len, chars);
                        try! (self.yield_null (callback, level + 1, idx));
                        off (&mut state, VAL_PASSED);
                    }

                    Token::Colon if not (state, KEY_PASSED) && not (state, SEP_PASSED) => {
                        self.skip (reader, len, chars);
                        try! (self.yield_null (callback, level + 1, idx));
                        on (&mut state, KEY_PASSED);
                        on (&mut state, SEP_PASSED);
                    }

                    _ if not (state, KEY_PASSED) => {
                        let indent = self.cursor;
                        let mut cur_idx = self.index;
                        try! (self.read_node_flow (reader, callback, &ctx, indent, level + 1, idx, &mut cur_idx, Some ( (token, len, chars) ), &mut None, &mut None));
                        on (&mut state, KEY_PASSED);
                    }

                    _ if is (state, SEP_PASSED) && not (state, VAL_PASSED) => {
                        let indent = self.cursor;
                        let mut cur_idx = self.index;
                        try! (self.read_node_flow (reader, callback, &ctx, indent, level + 1, idx, &mut cur_idx, Some ( (token, len, chars) ), &mut None, &mut None));
                        on (&mut state, VAL_PASSED);
                    }

                    _ => { break }
                }
            } else { break; }
        }

        Ok ( () )
    }



    fn read_directive_yaml<R: Read> (&mut self, reader: &mut R, callback: &mut FnMut (Block) -> Result<(), Twine>, level: usize, parent_idx: usize) -> Result<(), ReadError> {
        self.tokenizer.get_token (reader)
            .ok_or_else (|| {
                let idx = self.get_idx ();
                self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Twine::from ("Unexpected end of the document while parse %YAML directive")).unwrap_err ()
            })
            .and_then (|(token, len, chars)| {
                match token {
                    Token::Indent => {
                        self.skip (reader, len, chars);
                        self.tokenizer.get_token (reader)
                            .ok_or_else (|| {
                                let idx = self.get_idx ();
                                self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Twine::from ("Cannot read the version part of the %YAML directive")).unwrap_err ()
                            })
                    }
                    _ => {
                        let idx = self.get_idx ();
                        Err (self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Twine::from ("Any %YAML directive should be followed by some space characters")).unwrap_err ())
                    }
                }
            }).and_then (|(_, len, chars)| {
                let marker = self.consume (reader, callback, len, chars) ?;

                self.check_yaml_version (callback, level, parent_idx, &marker)
                    .and_then (|ver| {
                        let idx = self.get_idx ();
                        self.yield_block (Block::new (
                            Id { level: level, parent: parent_idx, index: idx },
                            BlockType::DirectiveYaml (ver)
                        ), callback)
                    })
            })
    }


    fn check_bom<R: Read> (&mut self, reader: &mut R, callback: &mut FnMut (Block) -> Result<(), Twine>, level: usize, parent_idx: usize, len: usize) -> Result<(), ReadError> {
        let is_my_bom = {
            let bom = reader.slice (len).unwrap ();
            self.tokenizer.cset.encoding.check_bom (bom)
        };

        if !is_my_bom {
            let idx = self.get_idx ();
            return self.yield_error (
                callback,
                Id { level: level, parent: parent_idx, index: idx },
                Twine::from ("Found a BOM of another encoding")
            )
        }

        Ok ( () )
    }


    fn check_yaml_version (&mut self, callback: &mut FnMut (Block) -> Result<(), Twine>, level: usize, parent_idx: usize, marker: &Marker) -> Result<(u8, u8), ReadError> {
        enum R {
            Err (Twine),
            Warn (Twine, (u8, u8))
        };

        let result = {
            let chunk = self.data.chunk (&marker);
            let chunk_slice = chunk.as_slice ();

            if self.tokenizer.directive_yaml_version.same_as_slice (chunk_slice) { return Ok ( (1, 2) ) }

            if let Some ( (digit_first, digit_first_len) ) = self.tokenizer.cset.extract_dec (chunk_slice) {
                if digit_first != 1 || !self.tokenizer.cset.full_stop.contained_at (chunk_slice, digit_first_len) {
                    R::Err (Twine::from ("%YAML major version is not supported"))
                } else {
                    if let Some ( (digit_second, digit_second_len) ) = self.tokenizer.cset.extract_dec_at (chunk_slice, digit_first_len + self.tokenizer.cset.full_stop.len ()) {
                        if chunk_slice.len () > digit_first_len + digit_second_len + self.tokenizer.cset.full_stop.len () {
                            R::Warn (Twine::from ("%YAML minor version is not fully supported"), (digit_first, 3))
                        } else {
                            if digit_second == 1 {
                                R::Warn ( Twine::from (format! (
                                    "{}. {}.",
                                    "%YAML version 1.1 is supported accordingly to the YAML 1.2 specification, paragraph 5.4",
                                    "This means that non-ASCII line-breaks are considered to be non-break characters"
                                )), (digit_first, digit_second) )
                            } else {
                                R::Err (Twine::from ("%YAML minor version is not supported"))
                            }
                        }
                    } else {
                        R::Err ( Twine::from ("%YAML version is malformed") )
                    }
                }
            } else {
                R::Err ( Twine::from ("%YAML version is malformed") )
            }
        };

        match result {
            R::Err (msg) => {
                let idx = self.get_idx ();
                Err (self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, msg).unwrap_err ())
            },
            R::Warn (msg, res) => {
                let idx = self.get_idx ();
                self.yield_warning (callback, Id { level: level, parent: parent_idx, index: idx }, msg) ?;
                Ok (res)
            }
        }
    }


/*
    fn check_yaml_version (&mut self, callback: &mut FnMut (Block) -> Result<(), Twine>, level: usize, parent_idx: usize, marker: &Marker) -> Result<(u8, u8), ReadError> {
        // let chunk = self.data.chunk (&marker);
        // let chunk_slice = chunk.as_slice ();

        if self.tokenizer.directive_yaml_version.same_as_slice (self.data.chunk (&marker).as_slice ()) { return Ok ( (1, 2) ) }

        /*
        if let Some ( (digit_first, digit_first_len) ) = self.tokenizer.cset.extract_dec (chunk_slice) {
            if digit_first != 1 || !self.tokenizer.cset.full_stop.contained_at (chunk_slice, digit_first_len) {
                let idx = self.get_idx ();
                return Err (self.yield_error (
                    callback,
                    Id { level: level, parent: parent_idx, index: idx },
                    Twine::from ("%YAML major version is not supported")
                ).unwrap_err ())
            }

            if let Some ( (digit_second, digit_second_len) ) = self.tokenizer.cset.extract_dec_at (chunk_slice, digit_first_len + self.tokenizer.cset.full_stop.len ()) {
                if chunk_slice.len () > digit_first_len + digit_second_len + self.tokenizer.cset.full_stop.len () {
                    let idx = self.get_idx ();
                    try! (self.yield_warning (
                        callback,
                        Id { level: level, parent: parent_idx, index: idx },
                        Twine::from ("%YAML minor version is not fully supported")
                    ));

                    Ok ( (digit_first, 3) )
                } else {
                    if digit_second == 1 {
                        let idx = self.get_idx ();
                        try! (self.yield_warning (callback, Id { level: level, parent: parent_idx, index: idx }, Twine::from (format! (
                            "{}. {}.",
                            "%YAML version 1.1 is supported accordingly to the YAML 1.2 specification, paragraph 5.4",
                            "This means that non-ASCII line-breaks are considered to be non-break characters"
                        ))));
                    } else {
                        let idx = self.get_idx ();
                        return Err (self.yield_error (
                            callback,
                            Id { level: level, parent: parent_idx, index: idx },
                            Twine::from ("%YAML minor version is not supported")
                        ).unwrap_err ())
                    }

                    Ok ( (digit_first, digit_second) )
                }
            } else {
                let idx = self.get_idx ();
                Err (self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Twine::from ("%YAML version is malformed")).unwrap_err ())
            }
        } else {
            let idx = self.get_idx ();
            Err (self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Twine::from ("%YAML version is malformed")).unwrap_err ())
        }
        */


        self.tokenizer.cset.extract_dec (self.data.chunk (&marker).as_slice ())
            .ok_or_else (|| {
                let idx = self.get_idx ();
                self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Twine::from ("%YAML version is malformed")).unwrap_err ()
            })
            .and_then (move |(digit_first, digit_first_len)| {
                if digit_first != 1 || !self.tokenizer.cset.full_stop.contained_at (self.data.chunk (&marker).as_slice (), digit_first_len) {
                    let idx = self.get_idx ();
                    return Err (self.yield_error (
                        callback,
                        Id { level: level, parent: parent_idx, index: idx },
                        Twine::from ("%YAML major version is not supported")
                    ).unwrap_err ())
                }

                self.tokenizer.cset.extract_dec_at (self.data.chunk (&marker).as_slice (), digit_first_len + self.tokenizer.cset.full_stop.len ())
                    .ok_or_else (|| {
                        let idx = self.get_idx ();
                        self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Twine::from ("%YAML version is malformed")).unwrap_err ()
                    })
                    .and_then (|(digit_second, digit_second_len)| {
                        if self.data.chunk (&marker).as_slice ().len () > digit_first_len + digit_second_len + self.tokenizer.cset.full_stop.len () {
                            let idx = self.get_idx ();
                            try! (self.yield_warning (
                                callback,
                                Id { level: level, parent: parent_idx, index: idx },
                                Twine::from ("%YAML minor version is not fully supported")
                            ));

                            Ok ( (digit_first, 3) )
                        } else {
                            if digit_second == 1 {
                                let idx = self.get_idx ();
                                try! (self.yield_warning (callback, Id { level: level, parent: parent_idx, index: idx }, Twine::from (format! (
                                    "{}. {}.",
                                    "%YAML version 1.1 is supported accordingly to the YAML 1.2 specification, paragraph 5.4",
                                    "This means that non-ASCII line-breaks are considered to be non-break characters"
                                ))));
                            } else {
                                let idx = self.get_idx ();
                                return Err (self.yield_error (
                                    callback,
                                    Id { level: level, parent: parent_idx, index: idx },
                                    Twine::from ("%YAML minor version is not supported")
                                ).unwrap_err ())
                            }

                            Ok ( (digit_first, digit_second) )
                        }
                    })
            })
    }
*/


    fn read_directive_tag<R: Read> (&mut self, reader: &mut R, callback: &mut FnMut (Block) -> Result<(), Twine>, level: usize, parent_idx: usize) -> Result<(), ReadError> {
        self.tokenizer.get_token (reader)
            .ok_or_else (|| {
                let idx = self.get_idx ();
                self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Twine::from ("Unexpected end of the document while parse %TAG directive")).unwrap_err ()
            })
            .and_then (|(token, len, chars)| {
                match token {
                    Token::Indent => {
                        self.skip (reader, len, chars);
                        self.tokenizer.get_token (reader)
                            .ok_or_else (|| {
                                let idx = self.get_idx ();
                                self.yield_error (
                                    callback,
                                    Id { level: level, parent: parent_idx, index: idx },
                                    Twine::from ("Cannot read the handle part of a %TAG directive")
                                ).unwrap_err ()
                            })
                    }
                    _ => {
                        let idx = self.get_idx ();
                        Err (self.yield_error (
                            callback,
                            Id { level: level, parent: parent_idx, index: idx },
                            Twine::from ("%TAG directive should be followed by some space characters")
                        ).unwrap_err ())
                    }
                }
            })
            .and_then (|(token, len, chars)| {
                match token {
                    Token::TagHandle => (),
                    _ => {
                        let idx = self.get_idx ();
                        return Err (self.yield_error (
                            callback,
                            Id { level: level, parent: parent_idx, index: idx },
                            Twine::from ("Handle part of a tag must have the format of a tag handle")
                        ).unwrap_err ())
                    }
                };

                let (more, _) = scan_until_at (len, reader, &self.tokenizer.anchor_stops);
                let handle = self.consume (reader, callback, len + more, chars) ?;

                /* Indent */
                self.tokenizer.get_token (reader)
                    .ok_or_else (|| {
                        let idx = self.get_idx ();
                        self.yield_error (
                            callback,
                            Id { level: level, parent: parent_idx, index: idx },
                            Twine::from ("Cannot read the prefix part of a %TAG directive")
                        ).unwrap_err ()
                    }).and_then (|(token, len, chars)| {
                        match token {
                            Token::Indent => self.skip (reader, len, chars),
                            _ => {
                                let idx = self.get_idx ();
                                return Err (self.yield_error (
                                    callback,
                                    Id { level: level, parent: parent_idx, index: idx },
                                    Twine::from ("%TAG handle should be followed by some space characters")
                                ).unwrap_err ());
                            }
                        };

                        self.tokenizer.get_token (reader)
                            .ok_or_else (|| {
                                let idx = self.get_idx ();
                                self.yield_error (
                                    callback,
                                    Id { level: level, parent: parent_idx, index: idx },
                                    Twine::from ("Cannot read the prefix part of a %TAG directive")
                                ).unwrap_err ()
                            })
                            .and_then (|(token, len, chars)| {
                                let mut read = len;
                                let rchs = chars;
                                match token {
                                    Token::TagHandle => (),
                                    Token::Raw => {
                                        let (more, _) = scan_until_at (len, reader, &self.tokenizer.anchor_stops);
                                        read += more;
                                    },
                                    _ => {
                                        let idx = self.get_idx ();
                                        return Err (self.yield_error (
                                            callback,
                                            Id { level: level, parent: parent_idx, index: idx },
                                            Twine::from ("Prefix part of a tag must have the format of a tag handle or uri")
                                        ).unwrap_err ())
                                    }
                                };

                                
                                let prefix = self.consume (reader, callback, read, rchs) ?;

                                let idx = self.get_idx ();
                                self.yield_block (Block::new (
                                    Id { level: level, parent: parent_idx, index: idx },
                                    BlockType::DirectiveTag ( (handle, prefix) )
                                ), callback)
                            })
                    })
            })
    }


    fn emit_doc_border (&mut self, callback: &mut FnMut (Block) -> Result<(), Twine>, level: usize, parent_idx: usize, token: Token) -> Result<(), ReadError> {
        let idx = self.get_idx ();
        self.yield_block (Block::new (
            Id { level: level, parent: parent_idx, index: idx },
            match token { Token::DocumentStart => BlockType::DocStart, _ => BlockType::DocEnd }
        ), callback)
    }


    fn read_seq_block_item<R: Read> (&mut self, reader: &mut R, callback: &mut FnMut (Block) -> Result<(), Twine>, ctx: &Context, level: usize, parent_idx: usize, indent: &mut usize, mut accel: Option<(Token, usize, usize)>) -> Result<(), ReadError> {
        let prev_indent: usize = *indent;
        if let Some ( (token, len, chars) ) = if accel.is_none () { self.tokenizer.get_token (reader) } else { accel.take () } {
            match token {
                Token::Dash => {
                    self.skip (reader, len, chars);
                    *indent = self.cursor;
                }
                _ => {
                    let idx = self.get_idx ();
                    return self.yield_error (
                        callback,
                        Id { level: level, parent: parent_idx, index: idx },
                        Twine::from ("Unexpected token (expected was '-')")
                    )
                }
            }
        } else {
            let idx = self.get_idx ();
            return self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Twine::from (format! ("Unexpected end of the document ({}:{})", file! (), line! ())))
        }


        'top: loop {
            if let Some ( (token, len, chars) ) = self.tokenizer.get_token (reader) {
                match token {
                    Token::BOM32BE |
                    Token::BOM32LE |
                    Token::BOM16BE |
                    Token::BOM16LE |
                    Token::BOM8 => {
                        try! (self.check_bom (reader, callback, level, parent_idx, len));
                        self.skip (reader, len, chars);
                    }

                    Token::Indent => {
                        self.skip (reader, len, chars);
                        *indent = self.cursor;
                    }

                    _ => {
                        let indent = self.cursor;
                        let mut cur_idx = self.index;
                        try! (self.read_node_sblockval (reader, callback, &ctx, indent, level, parent_idx, &mut cur_idx, Some ( (token, len, chars) ), &mut None, &mut None, prev_indent));
                        break 'top;
                    }
                };
            } else {
                // end of the doc in here
                return self.yield_null (callback, level, parent_idx);
            }
        }

        Ok ( () )
    }


    fn read_node_flow<R: Read> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block) -> Result<(), Twine>,
        ctx: &Context,
        indent: usize,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        accel: Option<(Token, usize, usize)>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        self.read_node_ (reader, callback, ctx, indent, level, parent_idx, cur_idx, accel, overanchor, overtag, true, false, None)
    }

    fn read_node_mblockval<R: Read> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block) -> Result<(), Twine>,
        ctx: &Context,
        indent: usize,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        accel: Option<(Token, usize, usize)>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        self.read_node_ (reader, callback, ctx, indent, level, parent_idx, cur_idx, accel, overanchor, overtag, false, true, None)
    }

    fn read_node_sblockval<R: Read> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block) -> Result<(), Twine>,
        ctx: &Context,
        indent: usize,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        accel: Option<(Token, usize, usize)>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>,
        orig_indent: usize
    ) -> Result<(), ReadError> {
        self.read_node_ (reader, callback, ctx, indent, level, parent_idx, cur_idx, accel, overanchor, overtag, false, false, Some(orig_indent))
    }

    fn read_node<R: Read> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block) -> Result<(), Twine>,
        ctx: &Context,
        indent: usize,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        accel: Option<(Token, usize, usize)>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        self.read_node_ (reader, callback, ctx, indent, level, parent_idx, cur_idx, accel, overanchor, overtag, false, false, None)
    }


    fn read_node_<R: Read> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block) -> Result<(), Twine>,
        ctx: &Context,
        indent: usize,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        mut accel: Option<(Token, usize, usize)>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>,
        flow: bool,
        map_block_val: bool,
        _seq_block_indent: Option<usize>
    ) -> Result<(), ReadError> {
        let ctx = Context::new (ctx, ContextKind::Node, self.cursor, level);
        const ALIAS_READ: u8 = 4;

        const NEWLINE_PASSED: u8 = 8;
        const INDENT_PASSED: u8 = 16;
        const AFTER_SPACE: u8 = 32;
        const LINE_BREAK: u8 = 64;
        const AFTER_NEWLINE: u8 = 128;

        let mut state: u8 = 0;

        let mut anchor: Option<Marker> = None;
        let mut tag: Option<Marker> = None;

        let mut flow_idx: usize = 0;
        let mut flow_opt: Option<Block> = None;

        let mut map_block_val_pass: bool = false;

        let float_start = if let Some ( (ref token, _, _) ) = accel {
            match *token {
                Token::Anchor    |
                Token::TagHandle => true,
                _ => false
            }
        } else { false };

        'top: loop {
            if let Some ( (token, len, chars) ) = if accel.is_none () { self.tokenizer.get_token (reader) } else { accel.take () } {
                loop {
                    match token {
                        Token::BOM32BE |
                        Token::BOM32LE |
                        Token::BOM16BE |
                        Token::BOM16LE |
                        Token::BOM8 => {
                            try! (self.check_bom (reader, callback, level, parent_idx, len));
                            self.skip (reader, len, chars);
                        }


                        Token::Tab    |
                        Token::Indent
                            if is (state, ALIAS_READ) => self.skip (reader, len, chars),


                        _ if is (state, ALIAS_READ) => break 'top,


                        Token::TagHandle |
                        Token::Anchor    |
                        Token::Alias
                            if flow_idx > 0 => break 'top,


                        Token::DocumentStart |
                        Token::DocumentEnd
                            if !flow && flow_idx > 0 && indent == 0 => break 'top,


                        Token::Comma |
                        Token::DictionaryEnd |
                        Token::SequenceEnd
                            if flow_idx > 0 && flow => break 'top,


                        Token::Colon if flow_idx > 0 => {
                            if scan_one_at (len, reader, &self.tokenizer.spaces_and_line_breakers).is_some () {
                                break 'top;
                            } else if flow && self.tokenizer.cset.comma.read_at (len, reader).is_some () {
                                break 'top;
                            } else {
                                match flow_opt {
                                    Some (Block { id, cargo: BlockType::Node (Node {
                                        anchor: _,
                                        tag: _,
                                        content: NodeKind::Scalar (mut chunk)
                                    }) }) => {
                                        try! (self.yield_block (Block::new (id, BlockType::Node (Node {
                                            anchor: None,
                                            tag: None,
                                            content: NodeKind::LiteralBlockOpen
                                        })), callback));

                                        *cur_idx = id.index;
                                        flow_opt = Some (Block::new (id, BlockType::Node (Node {
                                            anchor: None,
                                            tag: None,
                                            content: NodeKind::LiteralBlockClose
                                        })));

                                        self.rtrim (&mut chunk);

                                        let idx = self.get_idx ();
                                        try! (self.yield_block (Block::new (
                                            Id { level: level + 1, parent: flow_idx, index: idx },
                                            BlockType::Literal (chunk)
                                        ), callback));
                                    },
                                    _ => ()
                                };

                                let marker = self.consume (reader, callback, len, chars) ?;
                                let idx = self.get_idx ();
                                try! (self.yield_block (Block::new (
                                    Id { level: level + 1, parent: flow_idx, index: idx },
                                    BlockType::Literal (marker)
                                ), callback));
                                *cur_idx = idx;
                            };
                        }


                        Token::Indent if flow_idx > 0 && is (state, NEWLINE_PASSED) && not (state, INDENT_PASSED) => {
                            let mut pass = false;
                            let mut ctxptr: &Context = &ctx;
                            loop {
                                if ctxptr.parent.is_none () { break; }

                                ctxptr = ctxptr.parent.as_ref ().unwrap ();

                                match ctxptr.kind {
                                    ContextKind::SequenceFlow |
                                    ContextKind::MappingFlow => { pass = true; }
                                    ContextKind::MappingBlock if ctxptr.level == level => {
                                        pass = chars > ctxptr.indent;
                                    }
                                    _ => { continue }
                                };

                                break;
                            }

                            if !pass && chars < indent { break 'top; }

                            // spare one space / to be used in a literal block to join pieces
                            let (len_, chars_) = if not (state, AFTER_NEWLINE) {
                                (len - self.tokenizer.cset.space.len (), if chars > 0 { chars - 1 } else { 0 })
                            } else {
                                (len, chars)
                            };

                            self.skip (reader, len_, chars_);
                            on (&mut state, INDENT_PASSED);

                            if len_ > 0 { on (&mut state, AFTER_SPACE) };
                        }


                        Token::Tab if flow_idx > 0 && is (state, NEWLINE_PASSED) => {
                            self.skip (reader, len, chars);

                            if indent == 0 && not (state, INDENT_PASSED) {
                                // let mut chunk: Chunk = Chunk::with_capacity (self.tokenizer.cset.space.len ());
                                // chunk.push_slice (self.tokenizer.spaces[0].as_slice ());
                                let rune = Rune::from (self.tokenizer.spaces[0].clone ());

                                match flow_opt {
                                    Some (Block { id, cargo: BlockType::Node (Node {
                                        anchor: _,
                                        tag: _,
                                        content: NodeKind::Scalar (mut chunk)
                                    }) }) => {
                                        try! (self.yield_block (Block::new (id, BlockType::Node (Node {
                                            anchor: None,
                                            tag: None,
                                            content: NodeKind::LiteralBlockOpen
                                        })), callback));

                                        *cur_idx = id.index;
                                        flow_opt = Some (Block::new (id, BlockType::Node (Node {
                                            anchor: None,
                                            tag: None,
                                            content: NodeKind::LiteralBlockClose
                                        })));

                                        self.rtrim (&mut chunk);

                                        let idx = self.get_idx ();
                                        try! (self.yield_block (Block::new (
                                            Id { level: level + 1, parent: flow_idx, index: idx },
                                            BlockType::Literal (chunk)
                                        ), callback));
                                    },
                                    _ => ()
                                };

                                let idx = self.get_idx ();
                                try! (self.yield_block (Block::new (
                                    Id { level: level + 1, parent: flow_idx, index: idx },
                                    BlockType::Rune (rune, 1)
                                    // BlockType::Literal (chunk)
                                ), callback));
                                *cur_idx = idx;
                            }

                            on (&mut state, AFTER_SPACE | INDENT_PASSED);
                        }


                        _ if
                            flow_idx > 0
                            && (indent == 0
                                || (ctx.parent.is_some ()
                                    && match ctx.parent.as_ref ().unwrap ().kind {
                                        ContextKind::MappingFlow | ContextKind::SequenceFlow => true,
                                        _ => false
                                    }))
                            && is (state, NEWLINE_PASSED) && not (state, INDENT_PASSED) => { on (&mut state, INDENT_PASSED); }


                        Token::Comment if flow_idx > 0 && is (state, AFTER_SPACE) => {
                            self.skip (reader, len, chars);
                            self.nl ();
                            on (&mut state, NEWLINE_PASSED);
                            off (&mut state, INDENT_PASSED);
                        }


                        Token::Newline if flow_idx > 0 && not (state, NEWLINE_PASSED) => {
                            // skip only one space

                            let len_ =
                                if len >= self.tokenizer.cset.crlf.len () && self.tokenizer.cset.crlf.read (reader).is_some () {
                                    self.tokenizer.cset.crlf.len ()
                                }
                                else if len >= self.tokenizer.cset.line_feed.len () && self.tokenizer.cset.line_feed.read (reader).is_some () {
                                    self.tokenizer.cset.line_feed.len ()
                                }
                                else if len >= self.tokenizer.cset.carriage_return.len () && self.tokenizer.cset.carriage_return.read (reader).is_some () {
                                    self.tokenizer.cset.carriage_return.len ()
                                } else { len };

                            let chars_ = if chars > 0 { 1 } else { 0 };

                            self.skip (reader, len_, chars_);
                            self.nl ();

                            on (&mut state, NEWLINE_PASSED | AFTER_SPACE);
                            off (&mut state, INDENT_PASSED);
                        }


                        _ if flow_idx > 0 && is (state, INDENT_PASSED) => {
                            let mut skip = false;
                            let mut ctxptr: &Context = &ctx;
                            loop {
                                if ctxptr.parent.is_none () { break; }

                                ctxptr = ctxptr.parent.as_ref ().unwrap ();

                                match ctxptr.kind {
                                    ContextKind::SequenceFlow => {
                                        skip = self.check_next_is_right_square (reader, 0, false);
                                    }
                                    ContextKind::MappingFlow => {
                                        skip = self.check_next_is_right_curly (reader, 0, false);
                                    }
                                    _ => { continue }
                                };

                                break;
                            }

                            if skip {
                                self.consume (reader, callback, len, chars) ?;
                                break;
                            }

                            match flow_opt {
                                Some (Block { id, cargo: BlockType::Node (Node {
                                    anchor: _,
                                    tag: _,
                                    content: NodeKind::Scalar (mut chunk)
                                }) }) => {
                                    try! (self.yield_block (Block::new (id, BlockType::Node (Node {
                                        anchor: None,
                                        tag: None,
                                        content: NodeKind::LiteralBlockOpen
                                    })), callback));

                                    *cur_idx = id.index;
                                    flow_opt = Some (Block::new (id, BlockType::Node (Node {
                                        anchor: None,
                                        tag: None,
                                        content: NodeKind::LiteralBlockClose
                                    })));

                                    self.rtrim (&mut chunk);

                                    let idx = self.get_idx ();
                                    try! (self.yield_block (Block::new (
                                        Id { level: level + 1, parent: flow_idx, index: idx },
                                        BlockType::Literal (chunk)
                                    ), callback));
                                },
                                _ => ()
                            };


                            match token {
                                Token::Indent => {
                                    let marker = self.consume (reader, callback, len, chars) ?;
                                    let idx = self.get_idx ();
                                    try! (self.yield_block (Block::new (
                                        Id { level: level + 1, parent: flow_idx, index: idx },
                                        BlockType::Literal (marker)
                                    ), callback));
                                    *cur_idx = idx;

                                    on (&mut state, AFTER_SPACE | INDENT_PASSED);
                                }

                                Token::Newline => {
                                    let marker = self.consume (reader, callback, len, chars) ?;
                                    let idx = self.get_idx ();
                                    try! (self.yield_block (Block::new (
                                        Id { level: level + 1, parent: flow_idx, index: idx },
                                        BlockType::Literal (marker)
                                    ), callback));
                                    *cur_idx = idx;

                                    off (&mut state, INDENT_PASSED);
                                    on (&mut state, AFTER_SPACE | NEWLINE_PASSED | AFTER_NEWLINE);
                                }

                                Token::Comment => {
                                    let len = self.tokenizer.cset.hashtag.len ();
                                    let marker = self.consume (reader, callback, len, 1) ?;

                                    let idx = self.get_idx ();
                                    try! (self.yield_block (Block::new (
                                        Id { level: level + 1, parent: flow_idx, index: idx },
                                        BlockType::Literal (marker)
                                    ), callback));
                                    *cur_idx = idx;

                                    off (&mut state, AFTER_SPACE | NEWLINE_PASSED);
                                    on (&mut state, INDENT_PASSED);
                                }

                                _ => {
                                    if self.cursor == 0 { // 9.03 - we couldn't spare a space
                                        // let mut chunk: Chunk = Chunk::with_capacity (self.tokenizer.cset.space.len ());
                                        // chunk.push_slice (self.tokenizer.cset.space.as_slice ());

                                        let idx = self.get_idx ();
                                        let rune = Rune::from (self.tokenizer.cset.space.clone ());
                                        try! (self.yield_block (Block::new (
                                            Id { level: level + 1, parent: flow_idx, index: idx },
                                            BlockType::Rune (rune, 1)
                                            // BlockType::Literal (chunk)
                                        ), callback));
                                        *cur_idx = idx;
                                    }

                                    let mut chunk = match token {
                                        Token::Directive => {
                                            let (len, _) = scan_until_at (0, reader, &self.tokenizer.raw_stops);
                                            self.consume (reader, callback, len, 1) ?
                                        }
                                        _ => self.consume (reader, callback, len, chars) ?
                                    };
                                    
                                    self.rtrim (&mut chunk);

                                    let idx = self.get_idx ();
                                    try! (self.yield_block (Block::new (
                                        Id { level: level + 1, parent: flow_idx, index: idx },
                                        BlockType::Literal (chunk)
                                    ), callback));
                                    *cur_idx = idx;

                                    off (&mut state, NEWLINE_PASSED | AFTER_SPACE /*| INDENT_PASSED*/);
                                }
                            }
                        }


                        _ if flow_idx > 0 => break 'top,


                        Token::ReservedCommercialAt => {
                            let idx = self.get_idx ();
                            try! (self.yield_error (
                                callback,
                                Id { level: level, parent: parent_idx, index: idx },
                                Twine::from ("@ character is reserved and may not be used to start a plain scalar")
                            ));
                        }


                        Token::ReservedGraveAccent => {
                            let idx = self.get_idx ();
                            try! (self.yield_error (
                                callback,
                                Id { level: level, parent: parent_idx, index: idx },
                                Twine::from ("` character is reserved and may not be used to start a plain scalar")
                            ));
                        }


                        Token::Indent if is (state, LINE_BREAK) => {
                            let is_hyphen = self.tokenizer.cset.hyphen_minus.read_at (len, reader).is_some ();
                            let yes = if float_start {
                                if is_hyphen {
                                    if indent == 0 { true } else {
                                        let mut result = true;
                                        let mut ctxptr: &Context = &ctx;
                                        loop {
                                            if ctxptr.parent.is_none () { break; }

                                            ctxptr = ctxptr.parent.as_ref ().unwrap ();

                                            match ctxptr.kind {
                                                ContextKind::SequenceBlock => {
                                                    result = self.cursor + len > ctxptr.indent;
                                                },
                                                _ => { continue }
                                            };

                                            break;
                                        }
                                        result
                                    }
                                } else if self.tokenizer.cset.vertical_bar.read_at (len, reader).is_some () {
                                    true 
                                } else if self.tokenizer.cset.greater_than.read_at (len, reader).is_some () {
                                    true
                                } else {
                                    if ctx.parent.is_some () {
                                        let par = ctx.parent.as_ref ().unwrap ();
                                        match par.kind {
                                            ContextKind::MappingBlock => {
                                                if par.parent.is_some () {
                                                    let par = par.parent.as_ref ().unwrap ();
                                                    chars > par.indent
                                                } else { false }
                                            }
                                            _ => false
                                        }
                                    } else { false }
                                }
                            } else { false };

                            if yes || chars >= indent {
                                let cidx = *cur_idx;

                                if is_hyphen {
                                    self.skip (reader, len, 1);
                                    let idx = self.get_idx ();
                                    *cur_idx = idx;

                                    try! (self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                        anchor: if anchor.is_none () { overanchor.take () } else { anchor.take () },
                                        tag: if tag.is_none () { overtag.take () } else { tag.take () },
                                        content: NodeKind::Sequence
                                    })), callback));

                                    try! (self.read_seq_block (reader, callback, &ctx, level + 1, idx, None));
                                } else {
                                    try! (self.read_layer_expected (
                                        reader,
                                        callback,
                                        &ctx,
                                        level,
                                        parent_idx,
                                        cur_idx,
                                        if anchor.is_none () { overanchor } else { &mut anchor },
                                        if tag.is_none () { overtag } else { &mut tag }
                                    ))
                                };

                                if cidx == *cur_idx {
                                    let idx = self.get_idx ();

                                    *cur_idx = idx;

                                    let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                                        (anchor, tag)
                                    } else {
                                        (
                                            if anchor.is_none () { overanchor.take () } else { anchor },
                                            if tag.is_none () { overtag.take () } else { tag }
                                        )
                                    };

                                    return self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                        anchor: anchor,
                                        tag: tag,
                                        content: NodeKind::Null
                                    })), callback);
                                }

                                break 'top;

                            } else {
                                let idx = self.get_idx ();

                                *cur_idx = idx;

                                let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                                    (anchor, tag)
                                } else {
                                    (
                                        if anchor.is_none () { overanchor.take () } else { anchor },
                                        if tag.is_none () { overtag.take () } else { tag }
                                    )
                                };

                                return self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                    anchor: anchor,
                                    tag: tag,
                                    content: NodeKind::Null
                                })), callback);
                            }
                        }


                        _ if is (state, LINE_BREAK) => {
                            let doit = if indent == 0 { true } else {
                                match token {
                                    Token::Dash => {
                                        let mut result = true;
                                        let mut ctxptr: &Context = &ctx;
                                        loop {
                                            if ctxptr.parent.is_none () { break; }

                                            ctxptr = ctxptr.parent.as_ref ().unwrap ();

                                            match ctxptr.kind {
                                                ContextKind::SequenceBlock => {
                                                    result = self.cursor > ctxptr.indent;
                                                },
                                                _ => { continue }
                                            };

                                            break;
                                        }
                                        result
                                    },
                                    _ => {
                                        let mut result = false;
                                        let mut ctxptr: &Context = &ctx;
                                        loop {
                                            if ctxptr.parent.is_none () { break; }

                                            ctxptr = ctxptr.parent.as_ref ().unwrap ();

                                            match ctxptr.kind {
                                                ContextKind::Layer => {
                                                    result = ctxptr.indent == 0;
                                                },
                                                _ => { }
                                            };

                                            break;
                                        }
                                        result
                                    }
                                }
                            };

                            if doit {
                                let cidx = *cur_idx;

                                match token {
                                    Token::Dash => {
                                        let idx = self.get_idx ();
                                        *cur_idx = idx;

                                        try! (self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                            anchor: if anchor.is_none () { overanchor.take () } else { anchor.take () },
                                            tag: if tag.is_none () { overtag.take () } else { tag.take () },
                                            content: NodeKind::Sequence
                                        })), callback));

                                        try! (self.read_seq_block (reader, callback, &ctx, level + 1, idx, Some ( (token, len, chars) )));
                                    },
                                    _ => try! (self.read_layer_expected (
                                        reader,
                                        callback,
                                        &ctx,
                                        level,
                                        parent_idx,
                                        cur_idx,
                                        if anchor.is_none () { overanchor } else { &mut anchor },
                                        if tag.is_none () { overtag } else { &mut tag }
                                    ))
                                };

                                if cidx == *cur_idx {
                                    let idx = self.get_idx ();

                                    *cur_idx = idx;

                                    let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                                        (anchor, tag)
                                    } else {
                                        (
                                            if anchor.is_none () { overanchor.take () } else { anchor },
                                            if tag.is_none () { overtag.take () } else { tag }
                                        )
                                    };

                                    return self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                        anchor: anchor,
                                        tag: tag,
                                        content: NodeKind::Null
                                    })), callback);
                                }

                                break 'top;

                            } else {
                                let idx = self.get_idx ();

                                *cur_idx = idx;

                                let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                                    (anchor, tag)
                                } else {
                                    (
                                        if anchor.is_none () { overanchor.take () } else { anchor },
                                        if tag.is_none () { overtag.take () } else { tag }
                                        )
                                };

                                return self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                    anchor: anchor,
                                    tag: tag,
                                    content: NodeKind::Null
                                })), callback);
                            }
                        }


                        Token::Alias if anchor.is_none () && tag.is_none () => {
                            let ast_len = self.tokenizer.cset.asterisk.len ();
                            self.skip (reader, ast_len, 0);
                            let marker = self.consume (reader, callback, len - ast_len, chars) ?;

                            let idx = self.get_idx ();
                            try! (self.yield_block (Block::new (
                                Id { level: level, parent: parent_idx, index: idx },
                                BlockType::Alias (marker)
                            ), callback));

                            *cur_idx = idx;

                            on (&mut state, ALIAS_READ);
                        }


                        Token::Anchor if anchor.is_none () => {
                            let amp_len = self.tokenizer.cset.ampersand.len ();
                            self.skip (reader, amp_len, 0);
                            anchor = Some ( self.consume (reader, callback, len - amp_len, chars) ? );
                        }


                        Token::TagHandle if tag.is_none () => {
                            tag = Some ( self.consume (reader, callback, len, chars) ? );
                        }


                        Token::Dash => {
                            let idx = self.get_idx ();

                            try! (self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                anchor: if anchor.is_none () { overanchor.take () } else { anchor.take () },
                                tag: if tag.is_none () { overtag.take () } else { tag.take () },
                                content: NodeKind::Sequence
                            })), callback));

                            *cur_idx = idx;

                            return self.read_seq_block (reader, callback, &ctx, level + 1, idx, Some ( (token, len, chars) ));
                        }


                        Token::Question => {
                            let idx = self.get_idx ();
                            *cur_idx = self.index;

                            try! (self.yield_block (Block::new (
                                Id { level: level, parent: parent_idx, index: idx },
                                BlockType::Node (Node {
                                    anchor: if anchor.is_none () { overanchor.take () } else { anchor.take () },
                                    tag: if tag.is_none () { overtag.take () } else { tag.take () },
                                    content: NodeKind::Mapping
                                })
                            ), callback));

                            return self.read_map_block_explicit (reader, callback, &ctx, level + 1, idx, Some ( (token, len, chars) ))
                        }


                        Token::DictionaryStart |
                        Token::SequenceStart => {
                            self.skip (reader, len, chars);

                            let new_idx = self.get_idx ();

                            *cur_idx = new_idx;

                            return match token {
                                Token::SequenceStart => self.read_seq_flow (reader, callback, &ctx, new_idx, level, parent_idx, indent, anchor, tag, overanchor, overtag),
                                _ => self.read_map_flow (reader, callback, &ctx, new_idx, level, parent_idx, indent, anchor, tag, overanchor, overtag)
                            }
                        }


                        Token::StringSingle |
                        Token::StringDouble => {
                            let marker = self.consume (reader, callback, len, chars) ?;
                            let idx = self.get_idx ();

                            *cur_idx = idx;

                            let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                                (anchor, tag)
                            } else {
                                (
                                    if anchor.is_none () { overanchor.take () } else { anchor },
                                    if tag.is_none () { overtag.take () } else { tag }
                                )
                            };

                            return self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                anchor: anchor,
                                tag: tag,
                                content: NodeKind::Scalar (marker)
                            })), callback);
                        }


                        Token::GT |
                        Token::Pipe => {
                            return self.read_scalar_block (
                                reader,
                                callback,
                                &ctx,
                                level,
                                parent_idx,
                                cur_idx,
                                Some ( (token, len, chars) ),
                                anchor.take (),
                                tag,
                                overanchor,
                                overtag
                            );
                        }


                        Token::Comment => self.skip (reader, len, chars),


                        Token::Newline => {
                            self.skip (reader, len, chars);
                            self.nl ();

                            on (&mut state, LINE_BREAK);
                        }


                        Token::Tab    |
                        Token::Indent => self.skip (reader, len, chars),


                        Token::Colon if map_block_val && !map_block_val_pass => {
                            if scan_one_at (len, reader, &self.tokenizer.spaces_and_line_breakers).is_some () {
                                break 'top;
                            }

                            map_block_val_pass = true;
                            continue;
                        }


                        Token::Colon |
                        Token::Comma if tag.is_none () && anchor.is_none () => {
                            flow_idx = self.get_idx ();
                            *cur_idx = flow_idx;

                            let mut marker = self.consume (reader, callback, len, chars) ?;

                            let blen = self.data.marker_len (&marker);
                            self.rtrim (&mut marker);
                            let alen = self.data.marker_len (&marker);

                            if blen != alen { on (&mut state, AFTER_SPACE); }

                            flow_opt = Some (Block::new (Id { level: level, parent: parent_idx, index: flow_idx }, BlockType::Node (Node {
                                anchor: None,
                                tag: None,
                                content: NodeKind::Scalar (marker)
                            })));

                            on (&mut state, INDENT_PASSED);
                            off (&mut state, NEWLINE_PASSED);
                        }


                        Token::Raw => {
                            flow_idx = self.get_idx ();
                            *cur_idx = flow_idx;

                            let mut marker = self.consume (reader, callback, len, chars) ?;

                            let blen = self.data.marker_len (&marker);
                            self.rtrim (&mut marker);
                            let alen = self.data.marker_len (&marker);

                            if blen != alen { on (&mut state, AFTER_SPACE); }

                            flow_opt = Some (Block::new (Id { level: level, parent: parent_idx, index: flow_idx }, BlockType::Node (Node {
                                anchor: None,
                                tag: None,
                                content: NodeKind::Scalar (marker)
                            })));

                            on (&mut state, INDENT_PASSED);
                            off (&mut state, NEWLINE_PASSED);
                        }


                        _ if flow_idx == 0 && (tag.is_some () || anchor.is_some ()) => {
                            let idx = self.get_idx ();

                            *cur_idx = idx;

                            let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                                (anchor, tag)
                            } else {
                                (
                                    if anchor.is_none () { overanchor.take () } else { anchor },
                                    if tag.is_none () { overtag.take () } else { tag }
                                )
                            };

                            return self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                anchor: anchor,
                                tag: tag,
                                content: NodeKind::Null
                            })), callback);
                        }


                        _ => {
                            let idx = self.get_idx ();
                            return Err (self.yield_error (
                                callback,
                                Id { level: level, parent: parent_idx, index: idx },
                                Twine::from (format! (r"Unexpected token ({}:{})", file! (), line! ()))
                            ).unwrap_err ())
                        }
                    }
                    break;
                }
            } else {
                if flow_idx == 0 && not (state, ALIAS_READ) {
                    let idx = self.get_idx ();

                    *cur_idx = idx;

                    let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                        (anchor, tag)
                    } else {
                        (
                            if anchor.is_none () { overanchor.take () } else { anchor },
                            if tag.is_none () { overtag.take () } else { tag }
                        )
                    };

                    return self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                        anchor: anchor,
                        tag: tag,
                        content: NodeKind::Null
                    })), callback);
                }

                break;
            }
        }

        match flow_opt {
            Some (Block { id, cargo: BlockType::Node (Node {
                anchor: _,
                tag: _,
                content: NodeKind::Scalar (mut chunk)
            }) }) => {
                self.rtrim (&mut chunk);

                let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                    (anchor, tag)
                } else {
                    (
                        if anchor.is_none () { overanchor.take () } else { anchor },
                        if tag.is_none () { overtag.take () } else { tag }
                    )
                };

                *cur_idx = id.index;

                try! (self.yield_block (Block::new (id, BlockType::Node (Node {
                    anchor: anchor,
                    tag: tag,
                    content: NodeKind::Scalar (chunk)
                })), callback));
            }

            Some (Block { id, cargo: BlockType::Node (Node {
                anchor: _,
                tag: _,
                content: NodeKind::LiteralBlockClose
            }) }) => {

                let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                    (anchor, tag)
                } else {
                    (
                        if anchor.is_none () { overanchor.take () } else { anchor },
                        if tag.is_none () { overtag.take () } else { tag }
                    )
                };

                *cur_idx = id.index;

                try! (self.yield_block (Block::new (id, BlockType::Node (Node {
                    anchor: anchor,
                    tag: tag,
                    content: NodeKind::LiteralBlockClose
                })), callback));
            }

            Some (_) => unreachable! (),

            None => {
                if tag.is_some () || anchor.is_some () {
                    let idx = self.get_idx ();

                    *cur_idx = idx;

                    let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                        (anchor, tag)
                    } else {
                        (
                            if anchor.is_none () { overanchor.take () } else { anchor },
                            if tag.is_none () { overtag.take () } else { tag }
                        )
                    };

                    return self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                        anchor: anchor,
                        tag: tag,
                        content: NodeKind::Null
                    })), callback);
                }
            }
        };

        Ok ( () )
    }


    fn rtrim (&self, marker: &mut Marker) {
        let rtrimsize: usize = {
            let chunk = self.data.chunk (marker);;
            let chunk_slice = chunk.as_slice ();
            let mut ptr = chunk_slice.len ();

            'rtop: loop {
                for word in self.tokenizer.spaces_and_line_breakers.iter () {
                    if word.len () > ptr { continue; }

                    if word.contained_at (chunk_slice, ptr - word.len ()) {
                        ptr -= word.len ();
                        continue 'rtop;
                    }
                }

                break;
            }

            chunk_slice.len () - ptr
        };

        if rtrimsize > 0 {
            let trlen = self.data.marker_len (marker) - rtrimsize;
            
            *marker = self.data.resize (marker.clone (), trlen);
        }
    }


    fn check_next_is_colon<R: Read> (&mut self, reader: &mut R, indent: usize, mut newlined: bool) -> bool {
        let mut pos = 0;
        let mut ind = 0;

        loop {
            if let Some (_) = self.tokenizer.cset.colon.read_at (pos, reader) {
                if newlined { return ind >= indent; }

                return true;
            }

            if let Some ( (idx, _) ) = scan_one_at (pos, reader, &self.tokenizer.spaces) {
                pos += self.tokenizer.spaces[idx].len ();
                ind += 1;

                continue;
            }

            if let Some ( (idx, _) ) = scan_one_at (pos, reader, &self.tokenizer.line_breakers) {
                pos += self.tokenizer.line_breakers[idx].len ();
                ind = 0;
                newlined = true;

                continue;
            }

            if let Some (add) = self.tokenizer.cset.hashtag.read_at (pos, reader) {
                pos += add;

                let (skept, _) = scan_until_at (pos, reader, &self.tokenizer.line_breakers);
                pos += skept;
                ind = 0;
                newlined = true;

                continue;
            }

            break;
        }

        false
    }


    fn check_next_is_right_curly<R: Read> (&mut self, reader: &mut R, indent: usize, mut newlined: bool) -> bool {
        let mut pos = 0;
        let mut ind = 0;

        loop {
            if let Some (_) = self.tokenizer.cset.bracket_curly_right.read_at (pos, reader) {
                if newlined { return ind >= indent; }

                return true;
            }

            if let Some ( (idx, _) ) = scan_one_at (pos, reader, &self.tokenizer.spaces) {
                pos += self.tokenizer.spaces[idx].len ();
                ind += 1;

                continue;
            }

            if let Some ( (idx, _) ) = scan_one_at (pos, reader, &self.tokenizer.line_breakers) {
                pos += self.tokenizer.line_breakers[idx].len ();
                ind = 0;
                newlined = true;

                continue;
            }

            if let Some (add) = self.tokenizer.cset.hashtag.read_at (pos, reader) {
                pos += add;

                let (skept, _) = scan_until_at (pos, reader, &self.tokenizer.line_breakers);
                pos += skept;
                ind = 0;
                newlined = true;

                continue;
            }

            break;
        }

        false
    }


    fn check_next_is_right_square<R: Read> (&mut self, reader: &mut R, indent: usize, mut newlined: bool) -> bool {
        let mut pos = 0;
        let mut ind = 0;

        loop {
            if let Some (_) = self.tokenizer.cset.bracket_square_right.read_at (pos, reader) {
                if newlined { return ind >= indent; }

                return true;
            }

            if let Some ( (idx, _) ) = scan_one_at (pos, reader, &self.tokenizer.spaces) {
                pos += self.tokenizer.spaces[idx].len ();
                ind += 1;

                continue;
            }

            if let Some ( (idx, _) ) = scan_one_at (pos, reader, &self.tokenizer.line_breakers) {
                pos += self.tokenizer.line_breakers[idx].len ();
                ind = 0;
                newlined = true;

                continue;
            }

            if let Some (add) = self.tokenizer.cset.hashtag.read_at (pos, reader) {
                pos += add;

                let (skept, _) = scan_until_at (pos, reader, &self.tokenizer.line_breakers);
                pos += skept;
                ind = 0;
                newlined = true;

                continue;
            }

            break;
        }

        false
    }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use super::skimmer::reader::SliceReader;

    use tokenizer::Tokenizer;

    use txt::get_charset_utf8;

    use std::sync::mpsc::channel;



    #[test]
    fn test_directives () {
        let src = "%YAML 1.2\n%TAG !e! tag://example.com,2015:testapp/\n---\n...";

        let (sender, receiver) = channel ();
        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        ).unwrap_or_else (|err| { assert! (false, format! ("Unexpected result: {}", err)); });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (0, block.id.index);

            if let BlockType::Datum (..) = block.cargo {} else { assert! (false, "unexpected cargo") }
        } else { assert! (false, "no datum") }


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::DirectiveYaml ( (1, 2) ) = block.cargo {
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (2, block.id.index);

            if let BlockType::DirectiveTag ( (handle, tag) ) = block.cargo {
                let handle = reader.data.chunk (&handle);
                let tag = reader.data.chunk (&tag);
                let handle = handle.as_slice ();
                let tag = tag.as_slice ();

                assert_eq! (3, handle.len ());
                assert_eq! (handle, "!e!".as_bytes ());

                assert_eq! (tag.len (), "tag://example.com,2015:testapp/".as_bytes ().len ());
                assert_eq! (tag, "tag://example.com,2015:testapp/".as_bytes ());
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (3, block.id.index);

            if let BlockType::DocStart = block.cargo { } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (4, block.id.index);

            if let BlockType::DocEnd = block.cargo { } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_yaml_error_1 () {
        let expected_message = r"Unexpected end of the document while parse %YAML directive";

        let src = "%YAML";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });

        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (5, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_yaml_error_2 () {
        let expected_message = r"Any %YAML directive should be followed by some space characters";

        let src = "%YAML/1.2";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });

        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (5, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_yaml_error_3 () {
        let expected_message = r"Cannot read the version part of the %YAML directive";

        let src = "%YAML ";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });

        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (6, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_yaml_error_4 () {
        let expected_message = r"%YAML version is malformed";

        let src = "%YAML -";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });

        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (0, block.id.index);

            if let BlockType::Datum (..) = block.cargo {} else { assert! (false, "unexpected cargo") }
        } else { assert! (false, "no datum") }

        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (7, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_yaml_error_5 () {
        let expected_message = r"%YAML major version is not supported";

        let src = "%YAML 2";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });

        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (0, block.id.index);

            if let BlockType::Datum (..) = block.cargo {} else { assert! (false, "unexpected cargo") }
        } else { assert! (false, "no datum") }

        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (7, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_yaml_error_6 () {
        let expected_message = r"%YAML major version is not supported";

        let src = "%YAML 10.2";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (0, block.id.index);

            if let BlockType::Datum (..) = block.cargo {} else { assert! (false, "unexpected cargo") }
        } else { assert! (false, "no datum") }

        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (10, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_yaml_error_7 () {
        let expected_message = r"%YAML version is malformed";

        let src = "%YAML 1.a";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (0, block.id.index);

            if let BlockType::Datum (..) = block.cargo {} else { assert! (false, "unexpected cargo") }
        } else { assert! (false, "no datum") }

        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (9, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_yaml_error_8 () {
        let expected_message = r"%YAML minor version is not supported";

        let src = "%YAML 1.0";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (0, block.id.index);

            if let BlockType::Datum (..) = block.cargo {} else { assert! (false, "unexpected cargo") }
        } else { assert! (false, "no datum") }

        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (9, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_yaml_warning_1 () {
        let expected_message = r"%YAML minor version is not fully supported";

        let src = "%YAML 1.21";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        ).unwrap_or_else (|err| { assert! (false, format! ("Unexpected result: {}", err)); });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (0, block.id.index);

            if let BlockType::Datum (..) = block.cargo {} else { assert! (false, "unexpected cargo") }
        } else { assert! (false, "no datum") }


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Warning (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (10, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (2, block.id.index);

            if let BlockType::DirectiveYaml ( (1, 3) ) = block.cargo {
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_yaml_warning_2 () {
        let expected_message = format! (
            r"{}. {}.",
            r"%YAML version 1.1 is supported accordingly to the YAML 1.2 specification, paragraph 5.4",
            r"This means that non-ASCII line-breaks are considered to be non-break characters"
        );

        let src = "%YAML 1.1";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        ).unwrap_or_else (|err| { assert! (false, format! ("Unexpected result: {}", err)); });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (0, block.id.index);

            if let BlockType::Datum (..) = block.cargo {} else { assert! (false, "unexpected cargo") }
        } else { assert! (false, "no datum") }


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Warning (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (9, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (2, block.id.index);

            if let BlockType::DirectiveYaml ( (1, 1) ) = block.cargo {
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_tag_error_1 () {
        let expected_message = r"Unexpected end of the document while parse %TAG directive";

        let src = "%TAG";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (4, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_tag_error_2 () {
        let expected_message = r"%TAG directive should be followed by some space characters";

        let src = "%TAG/";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (4, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_tag_error_3 () {
        let expected_message = r"Cannot read the handle part of a %TAG directive";

        let src = "%TAG ";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (5, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_tag_error_4 () {
        let expected_message = r"Handle part of a tag must have the format of a tag handle";

        let src = "%TAG testo";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (5, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_tag_error_5 () {
        let expected_message = r"Cannot read the prefix part of a %TAG directive";

        let src = "%TAG !tag!";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (0, block.id.index);

            if let BlockType::Datum (..) = block.cargo {} else { assert! (false, "unexpected cargo") }
        } else { assert! (false, "no datum") }


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (10, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_tag_error_6 () {
        let expected_message = r"Cannot read the prefix part of a %TAG directive";

        let src = "%TAG !tag! ";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (0, block.id.index);

            if let BlockType::Datum (..) = block.cargo {} else { assert! (false, "unexpected cargo") }
        } else { assert! (false, "no datum") }


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (11, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_tag_error_7 () {
        let expected_message = r"Prefix part of a tag must have the format of a tag handle or uri";

        let src = "%TAG !tag! - testo";
        let (sender, receiver) = channel ();

        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        )
            .err ()
            .ok_or_else (|| { assert! (false, "There must be an error") })
            .ok ()
            .map_or ((), |err| { assert_eq! (expected_message, format! ("{}", err)) });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (0, block.id.index);

            if let BlockType::Datum (..) = block.cargo {} else { assert! (false, "unexpected cargo") }
        } else { assert! (false, "no datum") }


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::Error (msg, pos) = block.cargo {
                assert_eq! (expected_message, msg);
                assert_eq! (11, pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }



    #[test]
    fn test_directive_tag_ex_2_24 () {
        let src = "%TAG ! tag:clarkevans.com,2002:\n---\n";

        let (sender, receiver) = channel ();
        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        ).unwrap_or_else (|err| { assert! (false, format! ("Unexpected result: {:?}", err)); });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (0, block.id.index);

            if let BlockType::Datum (..) = block.cargo {} else { assert! (false, "unexpected cargo") }
        } else { assert! (false, "no datum") }


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::DirectiveTag ( (handle, tag) ) = block.cargo {
                let handle = reader.data.chunk (&handle);
                let tag = reader.data.chunk (&tag);
                let handle = handle.as_slice ();
                let tag = tag.as_slice ();

                assert_eq! (1, handle.len ());
                assert_eq! (handle, "!".as_bytes ());

                assert_eq! (tag.len (), "tag:clarkevans.com,2002:".as_bytes ().len ());
                assert_eq! (tag, "tag:clarkevans.com,2002:".as_bytes ());
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }

        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (2, block.id.index);

            if let BlockType::DocStart = block.cargo { } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }
}
