extern crate skimmer;

// extern crate gauger;
// use self::gauger::sample::{ Sample, Timer };

use self::skimmer::{ Data, Datum, Marker, Read /*, Rune*/ };
// use self::skimmer::symbol::{ Combo, CopySymbol };


use reader::tokenizer::{ self, Token };

use std::borrow::Cow;
use std::error::Error;
use std::fmt;

use std::ops::BitAnd;
use std::ops::BitOr;
// use std::ops::BitXor;
use std::ops::Not;

// use std::sync::Arc;



#[inline (always)]
fn is<T: BitAnd<Output=T> + Eq + Copy> (state: T, val: T) -> bool { val == state & val }

#[inline (always)]
fn not<T: BitAnd<Output=T> + Eq + Copy> (state: T, val: T) -> bool { !is (state, val) }

#[inline (always)]
fn on<T: BitOr<Output=T> + Copy> (state: &mut T, val: T) { *state = *state | val; }

#[inline (always)]
// fn off<T: BitAnd<Output=T> + Eq + BitXor<Output=T> + Copy> (state: &mut T, val: T) { *state = *state ^ (val & *state) }
fn off<T: BitAnd<Output=T> + Not<Output=T> + Eq + Copy> (state: &mut T, val: T) { *state = *state & !val }



#[derive (Debug)]
pub struct ReadError {
    pub position: usize,
    pub description: Cow<'static, str>
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
    pub fn new<T> (description: T) -> ReadError where T: Into<Cow<'static, str>> { ReadError { description: description.into (), position: 0 } }

    pub fn pos (mut self, pos: usize) -> ReadError {
        self.position = pos;
        self
    }
}




#[derive (Clone, Debug, Hash)]
pub struct Id {
    pub level: usize,
    pub parent: usize,
    pub index: usize
}




#[derive (Debug)]
pub struct Block<D> {
    pub id: Id,
    pub cargo: BlockType<D>
}



impl<D> Block<D>
  where
    D: Datum
{
    pub fn new (id: Id, cargo: BlockType<D>) -> Block<D> {
        let block = Block {
            id: id,
            cargo: cargo
        };

        block
    }
}




#[derive (Debug)]
pub enum BlockType<D> {
    Alias (Marker),

    DirectiveTag ( (Marker, Marker) ),
    DirectiveYaml ( (u8, u8) ),

    DocStart,
    DocEnd,

    BlockMap (Id, Option<Marker>, Option<Marker>),
    Literal (Marker),
    Byte (u8, usize),

    Node (Node),

    Error (Cow<'static, str>, usize),
    Warning (Cow<'static, str>, usize),

    StreamEnd,
    Datum (D)
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



#[derive (Copy, Clone, Debug)]
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



struct Context<D>
{
    kind: ContextKind,

    layer: usize,
    level: usize,
    indent: usize,

    data: Option<Data<D>>,
    parent: Option<*mut Context<D>>
}



impl<D> Context<D>
  where
    D: Datum + 'static
{
    pub fn zero () -> Context<D> {
        Context {
            parent: None,
            data: Some (Data::with_capacity (4)),
            kind: ContextKind::Zero,
            layer: 0,
            level: 0,
            indent: 0,
        }
    }


    pub fn new (parent: &mut Context<D>, kind: ContextKind, indent: usize, level: usize) -> Context<D> {
        Context {
            kind: kind,
            layer: match parent.kind { ContextKind::Zero => 0, _ => parent.layer + 1 },
            level: level,
            indent: indent,
            data: None,
            parent: Some (parent as *mut Context<D>)
        }
    }


    pub fn get_data (&mut self) -> &mut Data<D> {
        if let Some (data) = self.data.as_mut () {
            data
        } else if let Some (parent) = self.parent.as_mut () {
            unsafe { (**parent).get_data () }
        } else { unreachable! () }
    }


    pub fn get_parent (&mut self) -> Option<&mut Context<D>> {
        if let Some (parent) = self.parent.as_mut () {
            Some (unsafe { &mut **parent })
        } else { None }
    }


    pub fn get_parent_kind (&self) -> Option<ContextKind> {
        self.parent.as_ref ().map (|p| unsafe { (**p).kind })
    }
}



pub struct Reader {
    index: usize,
    line: usize,
    cursor: usize,
    position: usize,
    // data: Data<D>,
    // pub timer: Timer
}



impl Reader {
    pub fn new () -> Reader {
        Reader {
            index: 0,
            line: 0,
            cursor: 0,
            position: 0,
            // data: Data::with_capacity (4)

            // timer: Timer::new ()
        }
    }


    #[inline (always)]
    fn yield_block<D: Datum + 'static> (&mut self, block: Block<D>, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>) -> Result<(), ReadError> {
        // self.timer.stamp ("reader->yblock");
        if let Err (error) = callback (block) {
            // self.timer.stamp ("reader->yblock");
            Err (ReadError::new (error))
        } else {
            // self.timer.stamp ("reader->yblock");
            Ok ( () )
        }
    }


    pub fn read<D: Datum + 'static, R: Read<Datum=D>> (&mut self, mut reader: R, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>) -> Result<(), ReadError> {
        // self.timer.stamp ("reader");

        let mut ctx: Context<D> = Context::zero ();

        let mut cur_idx = self.index;
        let result = self.read_layer (&mut reader, callback, &mut ctx, 0, 0, &mut cur_idx, 0, &mut None, &mut None);
        self.yield_stream_end (callback).ok ();

        // self.timer.stamp ("reader");

        result
    }


    #[inline (always)]
    fn get_idx (&mut self) -> usize {
        self.index += 1;
        self.index
    }


    #[inline (always)]
    fn nl (&mut self) {
        self.line += 1;
        self.cursor = 0;
    }


    #[inline (always)]
    fn skip<D: Datum + 'static, R: Read<Datum=D>> (&mut self, reader: &mut R, len: usize, chars: usize) {
        self.cursor += chars;
        self.position += len;
        reader.skip_long (len);
    }


    fn consume<D, R> (&mut self, ctx: &mut Context<D>, reader: &mut R, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, len: usize, chars: usize) -> Result<Marker, ReadError>
      where
        D: Datum + 'static,
        R: Read<Datum=D>
    {
        self.cursor += chars;
        self.position += len;
        let marker = reader.consume_long (len);

        {
            let mut data = ctx.get_data ();

            if marker.pos2.0 > data.amount () {
                for i in data.amount () .. marker.pos2.0 + 1 {
                    let datum: D = reader.get_datum (i).unwrap ();
                    self.yield_block (Block::new (Id { level: 0, parent: 0, index: i }, BlockType::Datum (datum)), callback) ?;
                    data.push (reader.get_datum (i).unwrap ());
                }
            } else if data.amount () == 0 {
                let datum = reader.get_datum (0).unwrap ();
                self.yield_block (Block::new (Id { level: 0, parent: 0, index: 0 }, BlockType::Datum (datum.clone ())), callback) ?;
                data.push (datum);
            }
        }

        Ok (marker)
    }


    fn yield_stream_end<D: Datum + 'static> (&mut self, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>) -> Result<(), ReadError> {
        self.index = 0;
        self.line = 0;
        self.cursor = 0;
        self.position = 0;
        self.yield_block (Block::new (Id { level: 0, parent: 0, index: 0 }, BlockType::StreamEnd), callback)
    }


    #[inline (always)]
    fn yield_null<D: Datum + 'static> (&mut self, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, level: usize, parent_idx: usize) -> Result<(), ReadError> {
        let idx = self.get_idx ();
        self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
            anchor: None,
            tag: None,
            content: NodeKind::Null
        })), callback)
    }


    #[inline (always)]
    fn yield_error<D: Datum + 'static> (&mut self, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, id: Id, message: Cow<'static, str>) -> Result<(), ReadError> {
        let pos = self.position;
        self.yield_block (Block::new (id, BlockType::Error (message.clone (), pos)), callback) ?;

        Err (ReadError::new (message).pos (pos))
    }


    #[inline (always)]
    fn yield_warning<D: Datum + 'static> (&mut self, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, id: Id, message: Cow<'static, str>) -> Result<(), ReadError> {
        let pos = self.position;
        self.yield_block (Block::new (id, BlockType::Warning (message.clone (), pos)), callback)
    }


    #[inline (always)]
    fn read_layer_propagated<D: Datum + 'static, R: Read<Datum=D>> (&mut self, reader: &mut R, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, ctx: &mut Context<D>, level: usize, parent_idx: usize, anchor: &mut Option<Marker>, tag: &mut Option<Marker>) -> Result<(), ReadError> {
        let mut cur_idx = self.index;
        self.read_layer (reader, callback, ctx, level, parent_idx, &mut cur_idx, 15, anchor, tag) // INDENT_PASSED + INDENT_DEFINED + DIRS_PASSED
    }


    #[inline (always)]
    fn read_layer_expected<D: Datum + 'static, R: Read<Datum=D>> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>,
        ctx: &mut Context<D>,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        anchor: &mut Option<Marker>,
        tag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        self.read_layer (reader, callback, ctx, level, parent_idx, cur_idx, 3, anchor, tag)
    }


    fn read_layer<D: Datum + 'static, R: Read<Datum=D>> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>,
        ctx: &mut Context<D>,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        mut state: u8,
        anchor: &mut Option<Marker>,
        tag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        let mut ctx = Context::new (ctx, ContextKind::Layer, self.cursor, level);

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
            // self.timer.stamp ("reader->get_token");
            if let Some ( (token, len, chars) ) = tokenizer::get_token (reader) {

                // self.timer.stamp ("reader->get_token");
                loop {
                    match token {
                        /*
                        Token::BOM32BE |
                        Token::BOM32LE |
                        Token::BOM16BE |
                        Token::BOM16LE |
                        */
                        Token::BOM8 => {
                            // try! (self.check_bom (reader, callback, level, parent_idx, len));
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
                                Cow::from ("The YAML directive must only be given at most once per document")
                            );
                        }


                        Token::DirectiveYaml if not (state, YAML_PASSED) => {
                            self.skip (reader, len, chars);
                            try! (self.read_directive_yaml (&mut ctx, reader, callback, level, parent_idx));
                            on (&mut state, YAML_PASSED);
                        }


                        Token::DirectiveTag if not (state, DIRS_PASSED) => {
                            self.skip (reader, len, chars);
                            try! (self.read_directive_tag (&mut ctx, reader, callback, level, parent_idx));
                        }


                        Token::Directive if not (state, DIRS_PASSED) => {
                            let idx = self.get_idx ();
                            let line = self.line;

                            try! (self.yield_warning (
                                callback,
                                Id { level: level, parent: parent_idx, index: idx },
                                Cow::from (format! ("Unknown directive at the line {}", line))
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
                            // if let Some ((idx, ilen)) = scan_one_at (len, reader, &tokenizer::line_breakers) {
                            let ilen = tokenizer::scan_one_line_breaker (reader, len);
                            if ilen > 0 {
                                // let bchrs = tokenizer::line_breakers[idx].len_chars ();
                                self.skip (reader, len + ilen, chars + 1);
                                self.nl ();
                                break;
                            }

                            // if let Some (_) = tokenizer::cset.hashtag.read_at (len, reader) {
                            // if reader.contains_copy_at (tokenizer::cset.hashtag, len) {
                            if reader.byte_at (b'#', len) {
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

                            self.read_seq_block (reader, callback, &mut ctx, level + 1, idx, Some ( (token, len, chars) )) ?;

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

                            try! (self.read_map_block_implicit (reader, callback, &mut ctx, level + 1, idx, prev_indent, None));

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

                            try! (self.read_map_block_explicit (reader, callback, &mut ctx, level + 1, idx, Some ( (token, len, chars) )));

                            *cur_idx = self.index;

                            off (&mut state, INDENT_PASSED);
                        }


                        _ if not (state, NODE_PASSED) => {
                            let indent = if is (state, INDENT_DEFINED) { self.cursor } else { 0 };

                            try! (self.read_node (
                                reader,
                                callback,
                                &mut ctx,
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
                            return self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Cow::from ("Unexpected token / 0001"))
                        }
                    }
                    break;
                }
            } else {
                // self.timer.stamp ("reader->get_token");
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


    fn read_scalar_block_literal<D: Datum + 'static, R: Read<Datum=D>> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>,
        ctx: &mut Context<D>,
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
            if let Some ( (token, len, chars) ) = if accel.is_none () { tokenizer::get_token (reader) } else { accel.take () } {
                loop {
                    match token {
                        /*
                        Token::BOM32BE |
                        Token::BOM32LE |
                        Token::BOM16BE |
                        Token::BOM16LE |
                        */
                        Token::BOM8 => {
                            // try! (self.check_bom (reader, callback, level, parent_idx, len));
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

                            lazy_tail = Some ( (self.consume (ctx, reader, callback, len, chars) ?, chars) );

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
                            // let slen = tokenizer::cset.space.len ();
                            self.skip (reader, indent, indent);

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
                            // let mut chunk = Chunk::with_capacity (tokenizer::cset.line_feed.len () * nls);
                            // for _ in 0 .. nls { chunk.push_slice (tokenizer::cset.line_feed.as_slice ()); }
                            
                            let idx = self.get_idx ();
                            // let rune: Rune = tokenizer::cset.line_feed.into ();
                            try! (self.yield_block (Block::new (
                                Id { level: level, parent: parent_idx, index: idx },
                                BlockType::Byte (b'\n', nls)
                                // BlockType::Literal (chunk)
                            ), callback));
                            off (&mut state, HUNGRY | KEEPER);
                            continue;
                        }


                        _ if is (state, HUNGRY) => {
                            // TODO: try to take a part of the indentation instead of making a new chunk

                            // let mut chunk: Chunk;
                            let byte: u8;

                            if is (state, KEEPER) {
                                // chunk = Chunk::with_capacity (tokenizer::cset.line_feed.len ());
                                // chunk.push_slice (tokenizer::cset.line_feed.as_slice ());
                                // rune = tokenizer::cset.line_feed.into ();
                                byte = b'\n';
                            } else {
                                // chunk = Chunk::with_capacity (tokenizer::spaces[0].len ());
                                // chunk.push_slice (tokenizer::spaces[0].as_slice ());
                                // rune = tokenizer::cset.space.into ();
                                byte = b' ';
                            }

                            let idx = self.get_idx ();
                            try! (self.yield_block (Block::new (
                                Id { level: level, parent: parent_idx, index: idx },
                                BlockType::Byte (byte, 1)
                                // BlockType::Literal (chunk)
                            ), callback));

                            off (&mut state, HUNGRY | KEEPER);

                            continue;
                        }


                        _ => {
                            let len = tokenizer::line (reader);
                            let nl = tokenizer::scan_one_line_breaker (reader, len);
                            /*
                            let nl = if let Some ( (_, len) ) = scan_one_at (len, reader, &tokenizer::line_breakers) {
                                len
                            } else { 0 };
                            */

                            let mut tail: usize = 0;
                            let mut tail_nls: usize = 0;
                            lazy_nl = None;
                            lazy_tail = None;

                            if nl > 0 {
                                let mut spaces_add_bytes: usize = 0;
                                let mut spaces_add_chars: usize = 0;

                                loop {
                                    // if let Some ( (idx, len) ) = scan_one_at (len + nl + tail + spaces_add_bytes, reader, &[tokenizer::cset.space]) {
                                    // if reader.contains_copy_at (tokenizer::cset.space, len + nl + tail + spaces_add_bytes) {
                                    if reader.byte_at (b' ', len + nl + tail + spaces_add_bytes) {
                                        spaces_add_bytes += 1;
                                        spaces_add_chars += 1;
                                        continue;
                                    }

                                    // if let Some ( (_, len) ) = scan_one_at (len + nl + tail + spaces_add_bytes, reader, &tokenizer::line_breakers) {
                                    let len = tokenizer::scan_one_line_breaker (reader, len + nl + tail + spaces_add_bytes);
                                    if len > 0 {
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
                                let marker = self.consume (ctx, reader, callback, len, 0) ?;
                                let idx = self.get_idx ();
                                try! (self.yield_block (Block::new (
                                    Id { level: level, parent: parent_idx, index: idx },
                                    BlockType::Literal (marker)
                                ), callback));
                            }


                            if nl > 0 {
                                self.nl ();

                                if is (state, INDENT_DEFINED) {
                                    lazy_nl = Some (self.consume (ctx, reader, callback, nl, 0) ?);

                                    if tail > 0 {
                                        lazy_tail = Some ( (self.consume (ctx, reader, callback, tail, 0) ?, tail_nls) );
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
                    // let mut chunk = Chunk::with_capacity (tokenizer::cset.line_feed.len () * chars);
                    // for _ in 0..chars { chunk.push_slice (tokenizer::cset.line_feed.as_slice ()); }

                    let idx = self.get_idx ();
                    // let rune: Rune = tokenizer::cset.line_feed.into ();
                    try! (self.yield_block (Block::new (
                        Id { level: level, parent: parent_idx, index: idx },
                        BlockType::Byte (b'\n', chars)
                        // BlockType::Literal (chunk)
                    ), callback));
                }
            }
        }

        Ok ( () )
    }


    fn read_scalar_block<D: Datum + 'static, R: Read<Datum=D>> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>,
        ctx: &mut Context<D>,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        mut accel: Option<(Token, usize, usize)>,
        anchor: Option<Marker>,
        tag: Option<Marker>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        let mut ctx = Context::new (ctx, ContextKind::ScalarBlock, self.cursor, level);

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
            if let Some ( (token, len, chars) ) = if accel.is_none () { tokenizer::get_token (reader) } else { accel.take () } {
                loop {
                    match token {
                        /*
                        Token::BOM32BE |
                        Token::BOM32LE |
                        Token::BOM16BE |
                        Token::BOM16LE |
                        */
                        Token::BOM8 => {
                            // try! (self.check_bom (reader, callback, level, parent_idx, len));
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
                            // if let Some ((_, ilen)) = scan_one_at (0, reader, &tokenizer::line_breakers) {
                            let ilen = tokenizer::scan_one_line_breaker (reader, 0);
                            if ilen > 0 { self.skip (reader, ilen, 1); }

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
                            let marker = self.consume (&mut ctx, reader, callback, len, chars) ?;
                            let chunk = ctx.get_data ().chunk (&marker);
                            let chunk_slice = chunk.as_slice ();

                            let mut pos: usize = 0;

                            loop {
                                if not (state, CHOMP_DEFINED) {
                                    // if tokenizer::cset.hyphen_minus.contained_at (chunk_slice, pos) {
                                    match chunk_slice.get (pos).map (|b| *b) {
                                        Some (b'-') => {
                                            pos += 1;
                                            on (&mut state, CHOMP_DEFINED | CHOMP_STRIP);
                                            if indent > 0 { on (&mut state, INDENT_DEFINED) }
                                        }
                                        Some (b'+') => {
                                            pos += 1;
                                            on (&mut state, CHOMP_DEFINED | CHOMP_KEEP);
                                            if indent > 0 { on (&mut state, INDENT_DEFINED) }
                                        }
                                        _ => ()
                                    };
                                    /*
                                    if let Some (b'-') = chunk_slice.get (pos) {
                                        pos += tokenizer::cset.hyphen_minus.len ();
                                        on (&mut state, CHOMP_DEFINED | CHOMP_STRIP);
                                        if indent > 0 { on (&mut state, INDENT_DEFINED) }
                                    // } else if tokenizer::cset.plus.contained_at (chunk_slice, pos) {
                                    } else if let Some (b'+') = chunk_slice.get (pos) {
                                        pos += tokenizer::cset.plus.len ();
                                        on (&mut state, CHOMP_DEFINED | CHOMP_KEEP);
                                        if indent > 0 { on (&mut state, INDENT_DEFINED) }
                                    }
                                    */
                                }

                                if not (state, INDENT_DEFINED) {
                                    // if let Some ( (n, p) ) = tokenizer::cset.extract_dec_at (chunk_slice, pos) {
                                    if let Some (val @ b'0' ... b'9') = chunk_slice.get (pos).map (|b| *b) {
                                        pos += 1;
                                        indent = indent * 10 + ((val - b'0') as usize);

                                        continue;
                                    } else if indent > 0 {
                                        on (&mut state, INDENT_DEFINED);
                                    }
                                }

                                if not (state, CHOMP_DEFINED) {
                                    /*
                                    if tokenizer::cset.hyphen_minus.contained_at (chunk_slice, pos) {
                                        on (&mut state, CHOMP_DEFINED | CHOMP_STRIP);
                                        if indent > 0 { on (&mut state, INDENT_DEFINED) }
                                    } else if tokenizer::cset.plus.contained_at (chunk_slice, pos) {
                                        on (&mut state, CHOMP_DEFINED | CHOMP_KEEP);
                                        if indent > 0 { on (&mut state, INDENT_DEFINED) }
                                    }
                                    */
                                    match chunk_slice.get (pos).map (|b| *b) {
                                        Some (b'-') => {
                                            on (&mut state, CHOMP_DEFINED | CHOMP_STRIP);
                                            if indent > 0 { on (&mut state, INDENT_DEFINED) }
                                        }
                                        Some (b'+') => {
                                            on (&mut state, CHOMP_DEFINED | CHOMP_KEEP);
                                            if indent > 0 { on (&mut state, INDENT_DEFINED) }
                                        }
                                        _ => ()
                                    }
                                }

                                break;
                            }
                        }

                        _ if not (state, HEAD_PASSED) => {
                            let idx = self.get_idx ();
                            return self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Cow::from ("Unexpected token / 0002"))
                        }

                        _ if not (state, FOLDED) => return self.read_scalar_block_literal (
                            reader,
                            callback,
                            &mut ctx,
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
                            // let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                            // let (anchor, tag) = if self.check_next_is_char (tokenizer::cset.colon, reader, indent, false) {
                            let (anchor, tag) = if self.check_next_is_byte (b':', reader, indent, false) {
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
                            &mut ctx,
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
                            // let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                            // let (anchor, tag) = if self.check_next_is_char (tokenizer::cset.colon, reader, indent, false) {
                            let (anchor, tag) = if self.check_next_is_byte (b':', reader, indent, false) {
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


    fn read_seq_block<D: Datum + 'static, R: Read<Datum=D>> (&mut self, reader: &mut R, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, ctx: &mut Context<D>, level: usize, parent_idx: usize, mut accel: Option<(Token, usize, usize)>) -> Result<(), ReadError> {
        let mut ctx = Context::new (ctx, ContextKind::SequenceBlock, self.cursor, level);

        const INDENT_PASSED: u8 = 1; // Indentation has been passed for the line
        const NODE_READ: u8 = 2;

        let mut state: u8 = INDENT_PASSED;
        let indent = self.cursor;

        let mut prev_indent = indent;

        'top: loop {
            if let Some ( (token, len, chars) ) = if accel.is_none () { tokenizer::get_token (reader) } else { accel.take () } {
                loop {
                    match token {
                        /*
                        Token::BOM32BE |
                        Token::BOM32LE |
                        Token::BOM16BE |
                        Token::BOM16LE |
                        */
                        Token::BOM8 => {
                            // try! (self.check_bom (reader, callback, level, parent_idx, len));
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

                            // if let Some ((idx, ilen)) = scan_one_at (len, reader, &tokenizer::line_breakers) {
                            let ilen = tokenizer::scan_one_line_breaker (reader, len);
                            if ilen > 0 {
                                // let bchrs = tokenizer::line_breakers[idx].len_chars ();
                                // let bchrs = 1;
                                self.skip (reader, len + ilen, chars + 1);
                                self.nl ();
                                break;
                            }

                            // if let Some (_) = tokenizer::cset.hashtag.read_at (len, reader) {
                            // if reader.contains_copy_at (tokenizer::cset.hashtag, len) {
                            if reader.byte_at (b'#', len) {
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
                            self.read_seq_block_item (reader, callback, &mut ctx, level, parent_idx, &mut prev_indent, Some ( (token, len, chars) )) ?;
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

                            try! (self.read_map_block_implicit (reader, callback, &mut ctx, level + 1, idx, prev_indent, None));

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



    fn read_seq_flow<D: Datum + 'static, R: Read<Datum=D>> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>,
        ctx: &mut Context<D>,
        idx: usize,
        level: usize,
        parent_idx: usize,
        indent: usize,
        anchor: Option<Marker>,
        tag: Option<Marker>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        let mut ctx = Context::new (ctx, ContextKind::SequenceFlow, self.cursor, level);

        const NODE_PASSED: u8 = 1; // Indentation has been passed for the line
        const MAP_KEY_PASSED: u8 = 2;
        const COLON_IS_RAW: u8 = 4;

        let mut state: u8 = 0;
        let mut cur_idx = idx;

        'top: loop {
            if let Some ( (token, len, chars) ) = tokenizer::get_token (reader) {
                loop {
                    match token {
                        /*
                        Token::BOM32BE |
                        Token::BOM32LE |
                        Token::BOM16BE |
                        Token::BOM16LE |
                        */
                        Token::BOM8 => {
                            // try! (self.check_bom (reader, callback, level + 1, idx, len));
                            self.skip (reader, len, chars);
                        }


                        Token::SequenceEnd => {
                            self.skip (reader, len, chars);

                            // let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                            // let (anchor, tag) = if self.check_next_is_char (tokenizer::cset.colon, reader, indent, false) {
                            let (anchor, tag) = if self.check_next_is_byte (b':', reader, indent, false) {
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
                            try! (self.read_node_flow (reader, callback, &mut ctx, indent, level + 2, new_idx, &mut cur_idx_tmp, None, &mut None, &mut None));

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
                            try! (self.read_node_flow (reader, callback, &mut ctx, indent, level + 2, new_idx, &mut cur_idx, None, &mut None, &mut None));
                        }


                        Token::Colon if is (state, MAP_KEY_PASSED) && not (state, COLON_IS_RAW) => {
                            self.skip (reader, len, chars);
                            try! (self.read_node_flow (reader, callback, &mut ctx, indent, level + 2, cur_idx, &mut cur_idx, None, &mut None, &mut None));
                            off (&mut state, MAP_KEY_PASSED | COLON_IS_RAW);
                            on (&mut state, NODE_PASSED);
                        }


                        Token::Colon if not (state, MAP_KEY_PASSED) && not (state, NODE_PASSED) && not (state, COLON_IS_RAW) => {
                            // if let None = scan_one_at (len, reader, &tokenizer::spaces_and_line_breakers) {
                            if tokenizer::scan_one_spaces_and_line_breakers (reader, len) == 0 {
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
                            try! (self.read_node_flow (reader, callback, &mut ctx, indent, level + 2, new_idx, &mut cur_idx, None, &mut None, &mut None));

                            cur_idx = new_idx;

                            off (&mut state, MAP_KEY_PASSED | NODE_PASSED);
                        }


                        _ => {
                            let indent = self.cursor;
                            try! (self.read_node_flow (reader, callback, &mut ctx, indent, level + 1, idx, &mut cur_idx, Some ( (token, len, chars) ), &mut None, &mut None));
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

    #[inline (always)]
    fn read_map_block_explicit<D: Datum + 'static, R: Read<Datum=D>> (&mut self, reader: &mut R, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, ctx: &mut Context<D>, level: usize, parent_idx: usize, accel: Option<(Token, usize, usize)>) -> Result<(), ReadError> {
        self.read_map_block (reader, callback, ctx, level, parent_idx, 1, accel, None)
    }

    #[inline (always)]
    fn read_map_block_implicit<D: Datum + 'static, R: Read<Datum=D>> (&mut self, reader: &mut R, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, ctx: &mut Context<D>, level: usize, parent_idx: usize, indent: usize, accel: Option<(Token, usize, usize)>) -> Result<(), ReadError> {
        self.read_map_block (reader, callback, ctx, level, parent_idx, 15, accel, Some (indent))
    }


    fn read_map_block<D: Datum + 'static, R: Read<Datum=D>> (&mut self, reader: &mut R, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, ctx: &mut Context<D>, level: usize, parent_idx: usize, mut state: u8, mut accel: Option<(Token, usize, usize)>, indent: Option<usize>) -> Result<(), ReadError> {
        let mut ctx = Context::new (ctx, ContextKind::MappingBlock, if indent.is_some () { *indent.as_ref ().unwrap () } else { self.cursor }, level);
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
            if let Some ( (token, len, chars) ) = if accel.is_none () { tokenizer::get_token (reader) } else { accel.take () } {
                loop {
                    match token {
                        /*
                        Token::BOM32BE |
                        Token::BOM32LE |
                        Token::BOM16BE |
                        Token::BOM16LE |
                        */
                        Token::BOM8 => {
                            // try! (self.check_bom (reader, callback, level, parent_idx, len));
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
                            // if let Some ((idx, ilen)) = scan_one_at (len, reader, &tokenizer::line_breakers) {
                            let ilen = tokenizer::scan_one_line_breaker (reader, len);
                            if ilen > 0 {
                                // let bchrs = tokenizer::line_breakers[idx].len_chars ();
                                self.skip (reader, len + ilen, chars + 1);
                                self.nl ();
                                break;
                            }

                            // if let Some (_) = tokenizer::cset.hashtag.read_at (len, reader) {
                            // if reader.contains_copy_at (tokenizer::cset.hashtag, len) {
                            if reader.byte_at (b'#', len) {
                                self.skip (reader, len, chars);
                                break;
                            }

                            if chars < indent {
                                break 'top

                            } else if chars > indent {
                                self.skip (reader, len, chars);
                                try! (self.read_layer_propagated (reader, callback, &mut ctx, level, parent_idx, &mut None, &mut None));
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
                                // let (len_, _) = scan_while_at (len, reader, &tokenizer::spaces_and_tabs);
                                
                                let len_ = tokenizer::scan_while_spaces_and_tabs (reader, len);
                                
                                // let (len__, idx) = scan_until_at (len + len_, reader, &tokenizer::colon_and_line_breakers);
                                let len__ = tokenizer::scan_until_colon_and_line_breakers (reader, len + len_);

                                // println! ("LEN IS: {}, idx is {:?}", len__, idx);
                                // println! ("len_ is {}", len_);
                                // println! ("len__ is {}", len__);
                                // if let Some ( (idx, _) ) = idx {
                                if reader.has_long (len + len_ + len__ + 1) {
                                    // if idx > 0 {
                                    // if !reader.contains_copy_at (tokenizer::cset.colon, len + len_ + len__) {
                                    if !reader.byte_at (b':', len + len_ + len__) {
                                        // let (len___, _) = scan_while_at (len + len_ + len__, reader, &tokenizer::spaces_and_line_breakers);
                                        let len___ = tokenizer::scan_while_spaces_and_line_breakers (reader, len + len_ + len__);
                                        // if let Some ( (0, _) ) = scan_one_at (len + len_ + len__ + len___, reader, &tokenizer::colon_and_line_breakers) {
                                        // if reader.contains_copy_at (tokenizer::cset.colon, len + len_ + len__ + len___) {
                                        if reader.byte_at (b':', len + len_ + len__ + len___) {
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

                                            try! (self.read_node_mblockval (reader, callback, &mut ctx, indent + 1, level + 1, idx, &mut cur_idx, None, &mut None, &mut None));
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
                            try! (self.read_node_mblockval (reader, callback, &mut ctx, indent, level, parent_idx, &mut cur_idx, Some ( (token, len, chars) ), &mut None, &mut None));
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
                            try! (self.read_node_mblockval (reader, callback, &mut ctx, indent, level, parent_idx, &mut cur_idx, Some ( (token, len, chars) ), &mut None, &mut None));
                            on (&mut state, KEY_PASSED);
                            if self.cursor == 0 { off (&mut state, INDENT_PASSED); }
                        }

                        _ if is (state, SEP_PASSED) && not (state, VAL_PASSED) => {
                            let indent = self.cursor;
                            let mut cur_idx = self.index;
                            let lid = self.line;
                            last_val_idx = cur_idx + 1;

                            try! (self.read_node_mblockval (reader, callback, &mut ctx, indent, level, parent_idx, &mut cur_idx, Some ( (token, len, chars) ), &mut None, &mut None));
                            off (&mut state, VAL_PASSED);
                            // if tokenizer::cset.colon.read (reader).is_some () { on (&mut state, QST_PASSED); }
                            // if reader.contains_copy_at_start (tokenizer::cset.colon) { on (&mut state, QST_PASSED); }
                            if reader.byte_at_start (b':') { on (&mut state, QST_PASSED); }
                            if self.cursor == 0 { off (&mut state, INDENT_PASSED); }

                            if lid < self.line { break; }

                            'inliner: loop {
                                // if let Some (len1_colon) = tokenizer::cset.colon.read (reader) {
                                // if reader.contains_copy_at_start (tokenizer::cset.colon) {
                                if reader.byte_at_start (b':') {
                                    // let (len2_space, _) = scan_while_at (tokenizer::cset.colon.len (), reader, &tokenizer::spaces_and_tabs);
                                    // let len2_space = tokenizer::scan_while_spaces_and_tabs (reader, tokenizer::cset.colon.len ());
                                    let len2_space = tokenizer::scan_while_spaces_and_tabs (reader, 1);
                                    // let (_, idx) = scan_until_at (tokenizer::cset.colon.len () + len2_space, reader, &tokenizer::question_and_line_breakers);
                                    // let len2_question = tokenizer::scan_until_question_and_line_breakers (reader, tokenizer::cset.colon.len () + len2_space);
                                    let len2_question = tokenizer::scan_until_question_and_line_breakers (reader, 1 + len2_space);

                                    // if let Some ( (idx, _) ) = idx {
                                    // if reader.has (tokenizer::cset.colon.len () + len2_space + len2_question + 1) {
                                    if reader.has_long (2 + len2_space + len2_question) {
                                        // if idx > 0 {
                                        // if !reader.contains_copy_at (tokenizer::cset.question, tokenizer::cset.colon.len () + len2_space + len2_question) {
                                        if !reader.byte_at (b'?', 1 + len2_space + len2_question) {
                                            // let slen = tokenizer::cset.colon.len () + len2_space;
                                            // let slen = 1 + len2_space;
                                            self.skip (reader, len2_space + 1, 0);

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

                                            try! (self.read_node_mblockval (reader, callback, &mut ctx, indent + 1, level + 1, idx, &mut cur_idx, None, &mut None, &mut None));
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
                            return self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Cow::from ("Unexpected token / 0003"))
                        }
                    }

                    break;
                }
            } else { break; }
        }

        Ok ( () )
    }


    fn read_map_flow<D: Datum + 'static, R: Read<Datum=D>> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>,
        ctx: &mut Context<D>,
        idx: usize,
        level: usize,
        parent_idx: usize,
        indent: usize,
        anchor: Option<Marker>,
        tag: Option<Marker>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        let mut ctx = Context::new (ctx, ContextKind::MappingFlow, self.cursor, level);

        const QST_PASSED: u8 = 1;
        const KEY_PASSED: u8 = 3;
        const SEP_PASSED: u8 = 4;
        const VAL_PASSED: u8 = 15;

        let mut state: u8 = 0;

        loop {
            if let Some ( (token, len, chars) ) = tokenizer::get_token (reader) {
                match token {
                    /*
                    Token::BOM32BE |
                    Token::BOM32LE |
                    Token::BOM16BE |
                    Token::BOM16LE |
                    */
                    Token::BOM8 => {
                        // try! (self.check_bom (reader, callback, level + 1, idx, len));
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

                        // let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                        // let (anchor, tag) = if self.check_next_is_char (tokenizer::cset.colon, reader, indent, false) {
                        let (anchor, tag) = if self.check_next_is_byte (b':', reader, indent, false) {
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
                        try! (self.read_node_flow (reader, callback, &mut ctx, indent, level + 1, idx, &mut cur_idx, Some ( (token, len, chars) ), &mut None, &mut None));
                        on (&mut state, KEY_PASSED);
                    }

                    _ if is (state, SEP_PASSED) && not (state, VAL_PASSED) => {
                        let indent = self.cursor;
                        let mut cur_idx = self.index;
                        try! (self.read_node_flow (reader, callback, &mut ctx, indent, level + 1, idx, &mut cur_idx, Some ( (token, len, chars) ), &mut None, &mut None));
                        on (&mut state, VAL_PASSED);
                    }

                    _ => { break }
                }
            } else { break; }
        }

        Ok ( () )
    }



    fn read_directive_yaml<D: Datum + 'static, R: Read<Datum=D>> (&mut self, ctx: &mut Context<D>, reader: &mut R, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, level: usize, parent_idx: usize) -> Result<(), ReadError> {
        tokenizer::get_token (reader)
            .ok_or_else (|| {
                let idx = self.get_idx ();
                self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Cow::from ("Unexpected end of the document while parse %YAML directive")).unwrap_err ()
            })
            .and_then (|(token, len, chars)| {
                match token {
                    Token::Indent => {
                        self.skip (reader, len, chars);
                        tokenizer::get_token (reader)
                            .ok_or_else (|| {
                                let idx = self.get_idx ();
                                self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Cow::from ("Cannot read the version part of the %YAML directive")).unwrap_err ()
                            })
                    }
                    _ => {
                        let idx = self.get_idx ();
                        Err (self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Cow::from ("Any %YAML directive should be followed by some space characters")).unwrap_err ())
                    }
                }
            }).and_then (|(_, len, chars)| {
                let marker = self.consume (ctx, reader, callback, len, chars) ?;

                self.check_yaml_version (ctx, callback, level, parent_idx, &marker)
                    .and_then (|ver| {
                        let idx = self.get_idx ();
                        self.yield_block (Block::new (
                            Id { level: level, parent: parent_idx, index: idx },
                            BlockType::DirectiveYaml (ver)
                        ), callback)
                    })
            })
    }


/*
    fn check_bom<D: Datum + 'static, R: Read<Datum=D>> (&mut self, reader: &mut R, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, level: usize, parent_idx: usize, len: usize) -> Result<(), ReadError> {
        let is_my_bom = {
            let bom = reader.slice (len).unwrap ();
            tokenizer::cset.encoding.check_bom (bom)
        };

        if !is_my_bom {
            let idx = self.get_idx ();
            return self.yield_error (
                callback,
                Id { level: level, parent: parent_idx, index: idx },
                Cow::from ("Found a BOM of another encoding")
            )
        }

        Ok ( () )
    }
*/


    fn check_yaml_version<D: Datum + 'static> (&mut self, ctx: &mut Context<D>, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, level: usize, parent_idx: usize, marker: &Marker) -> Result<(u8, u8), ReadError> {
        enum R {
            Err (Cow<'static, str>),
            Warn (Cow<'static, str>, (u8, u8))
        };

        let result = {
            let chunk = ctx.get_data ().chunk (&marker);
            let chunk_slice = chunk.as_slice ();

            // if tokenizer::directive_yaml_version.same_as_slice (chunk_slice) { return Ok ( (1, 2) ) }
            /*
            if chunk_slice.len () == tokenizer::cset.digit_1.len () + tokenizer::cset.full_stop.len () + tokenizer::cset.digit_2.len () &&
               tokenizer::cset.digit_1.contained_at (chunk_slice, 0) &&
               tokenizer::cset.full_stop.contained_at (chunk_slice, tokenizer::cset.digit_1.len ()) &&
               tokenizer::cset.digit_2.contained_at (chunk_slice, tokenizer::cset.digit_1.len () + tokenizer::cset.full_stop.len ())
            {
                return Ok ( (1, 2) )
            }
            */
            if chunk_slice == &[b'1', b'.', b'2'] { return Ok ( (1, 2) ) }

            /*
            if let Some ( (digit_first, digit_first_len) ) = tokenizer::cset.extract_dec (chunk_slice) {
                if digit_first != 1 || !tokenizer::cset.full_stop.contained_at (chunk_slice, digit_first_len) {
                    R::Err (Cow::from ("%YAML major version is not supported"))
                } else {
                    if let Some ( (digit_second, digit_second_len) ) = tokenizer::cset.extract_dec_at (chunk_slice, digit_first_len + tokenizer::cset.full_stop.len ()) {
                        if chunk_slice.len () > digit_first_len + digit_second_len + tokenizer::cset.full_stop.len () {
                            R::Warn (Cow::from ("%YAML minor version is not fully supported"), (digit_first, 3))
                        } else {
                            if digit_second == 1 {
                                R::Warn ( Cow::from (format! (
                                    "{}. {}.",
                                    "%YAML version 1.1 is supported accordingly to the YAML 1.2 specification, paragraph 5.4",
                                    "This means that non-ASCII line-breaks are considered to be non-break characters"
                                )), (digit_first, digit_second) )
                            } else {
                                R::Err (Cow::from ("%YAML minor version is not supported"))
                            }
                        }
                    } else {
                        R::Err ( Cow::from ("%YAML version is malformed") )
                    }
                }
            } else {
                R::Err ( Cow::from ("%YAML version is malformed") )
            }
            */
            if let Some (val @ b'0' ... b'9') = chunk_slice.get (0).map (|b| *b) {
                let digit_first = val - b'0';
                if digit_first != 1 || chunk_slice.get (1).map (|b| *b) != Some (b'.') {
                    R::Err (Cow::from ("%YAML major version is not supported"))
                } else {
                    if let Some (val @ b'0' ... b'9') = chunk_slice.get (2).map (|b| *b) {
                        if chunk_slice.len () > 3 {
                            R::Warn (Cow::from ("%YAML minor version is not fully supported"), (digit_first, 3))
                        } else {
                            let digit_second = val - b'0';
                            if digit_second == 1 {
                                R::Warn ( Cow::from (format! (
                                    "{}. {}.",
                                    "%YAML version 1.1 is supported accordingly to the YAML 1.2 specification, paragraph 5.4",
                                    "This means that non-ASCII line-breaks are considered to be non-break characters"
                                )), (digit_first, digit_second) )
                            } else {
                                R::Err (Cow::from ("%YAML minor version is not supported"))
                            }
                        }
                    } else {
                        R::Err ( Cow::from ("%YAML version is malformed") )
                    }
                }
            } else { R::Err ( Cow::from ("%YAML version is malformed") ) }
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


    fn read_directive_tag<D: Datum + 'static, R: Read<Datum=D>> (&mut self, ctx: &mut Context<D>, reader: &mut R, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, level: usize, parent_idx: usize) -> Result<(), ReadError> {
        tokenizer::get_token (reader)
            .ok_or_else (|| {
                let idx = self.get_idx ();
                self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Cow::from ("Unexpected end of the document while parse %TAG directive")).unwrap_err ()
            })
            .and_then (|(token, len, chars)| {
                match token {
                    Token::Indent => {
                        self.skip (reader, len, chars);
                        tokenizer::get_token (reader)
                            .ok_or_else (|| {
                                let idx = self.get_idx ();
                                self.yield_error (
                                    callback,
                                    Id { level: level, parent: parent_idx, index: idx },
                                    Cow::from ("Cannot read the handle part of a %TAG directive")
                                ).unwrap_err ()
                            })
                    }
                    _ => {
                        let idx = self.get_idx ();
                        Err (self.yield_error (
                            callback,
                            Id { level: level, parent: parent_idx, index: idx },
                            Cow::from ("%TAG directive should be followed by some space characters")
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
                            Cow::from ("Handle part of a tag must have the format of a tag handle")
                        ).unwrap_err ())
                    }
                };

                // let (more, _) = scan_until_at (len, reader, &tokenizer::anchor_stops);
                let more = tokenizer::anchor_stops (reader, len);
                let handle = self.consume (ctx, reader, callback, len + more, chars) ?;

                /* Indent */
                tokenizer::get_token (reader)
                    .ok_or_else (|| {
                        let idx = self.get_idx ();
                        self.yield_error (
                            callback,
                            Id { level: level, parent: parent_idx, index: idx },
                            Cow::from ("Cannot read the prefix part of a %TAG directive")
                        ).unwrap_err ()
                    }).and_then (|(token, len, chars)| {
                        match token {
                            Token::Indent => self.skip (reader, len, chars),
                            _ => {
                                let idx = self.get_idx ();
                                return Err (self.yield_error (
                                    callback,
                                    Id { level: level, parent: parent_idx, index: idx },
                                    Cow::from ("%TAG handle should be followed by some space characters")
                                ).unwrap_err ());
                            }
                        };

                        tokenizer::get_token (reader)
                            .ok_or_else (|| {
                                let idx = self.get_idx ();
                                self.yield_error (
                                    callback,
                                    Id { level: level, parent: parent_idx, index: idx },
                                    Cow::from ("Cannot read the prefix part of a %TAG directive")
                                ).unwrap_err ()
                            })
                            .and_then (|(token, len, chars)| {
                                let mut read = len;
                                let rchs = chars;
                                match token {
                                    Token::TagHandle => (),
                                    Token::Raw => {
                                        // let (more, _) = scan_until_at (len, reader, &tokenizer::anchor_stops);
                                        let more = tokenizer::anchor_stops (reader, len);
                                        read += more;
                                    },
                                    _ => {
                                        let idx = self.get_idx ();
                                        return Err (self.yield_error (
                                            callback,
                                            Id { level: level, parent: parent_idx, index: idx },
                                            Cow::from ("Prefix part of a tag must have the format of a tag handle or uri")
                                        ).unwrap_err ())
                                    }
                                };

                                
                                let prefix = self.consume (ctx, reader, callback, read, rchs) ?;

                                let idx = self.get_idx ();
                                self.yield_block (Block::new (
                                    Id { level: level, parent: parent_idx, index: idx },
                                    BlockType::DirectiveTag ( (handle, prefix) )
                                ), callback)
                            })
                    })
            })
    }


    #[inline (always)]
    fn emit_doc_border<D: Datum + 'static> (&mut self, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, level: usize, parent_idx: usize, token: Token) -> Result<(), ReadError> {
        let idx = self.get_idx ();
        self.yield_block (Block::new (
            Id { level: level, parent: parent_idx, index: idx },
            match token { Token::DocumentStart => BlockType::DocStart, _ => BlockType::DocEnd }
        ), callback)
    }


    fn read_seq_block_item<D: Datum + 'static, R: Read<Datum=D>> (&mut self, reader: &mut R, callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>, ctx: &mut Context<D>, level: usize, parent_idx: usize, indent: &mut usize, mut accel: Option<(Token, usize, usize)>) -> Result<(), ReadError> {
        // let prev_indent: usize = *indent;

        if let Some ( (token, len, chars) ) = if accel.is_none () { tokenizer::get_token (reader) } else { accel.take () } {
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
                        Cow::from ("Unexpected token (expected was '-')")
                    )
                }
            }
        } else {
            let idx = self.get_idx ();
            return self.yield_error (callback, Id { level: level, parent: parent_idx, index: idx }, Cow::from (format! ("Unexpected end of the document ({}:{})", file! (), line! ())))
        }

        'top: loop {
            if let Some ( (token, len, chars) ) = tokenizer::get_token (reader) {
                match token {
                    /*
                    Token::BOM32BE |
                    Token::BOM32LE |
                    Token::BOM16BE |
                    Token::BOM16LE |
                    */
                    Token::BOM8 => {
                        // try! (self.check_bom (reader, callback, level, parent_idx, len));
                        self.skip (reader, len, chars);
                    }


                    Token::Indent => {
                        self.skip (reader, len, chars);
                        *indent = self.cursor;
                    }

                    _ => {
                        let indent = self.cursor;
                        let mut cur_idx = self.index;
                        self.read_node_sblockval (reader, callback, ctx, indent, level, parent_idx, &mut cur_idx, Some ( (token, len, chars) ), &mut None, &mut None) ?;
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


    #[inline (always)]
    fn read_node_flow<D: Datum + 'static, R: Read<Datum=D>> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>,
        ctx: &mut Context<D>,
        indent: usize,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        accel: Option<(Token, usize, usize)>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        self.read_node_ (reader, callback, ctx, indent, level, parent_idx, cur_idx, accel, overanchor, overtag, true, false)
    }


    #[inline (always)]
    fn read_node_mblockval<D: Datum + 'static, R: Read<Datum=D>> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>,
        ctx: &mut Context<D>,
        indent: usize,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        accel: Option<(Token, usize, usize)>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        self.read_node_ (reader, callback, ctx, indent, level, parent_idx, cur_idx, accel, overanchor, overtag, false, true)
    }


    #[inline (always)]
    fn read_node_sblockval<D: Datum + 'static, R: Read<Datum=D>> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>,
        ctx: &mut Context<D>,
        indent: usize,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        accel: Option<(Token, usize, usize)>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        self.read_node_ (reader, callback, ctx, indent, level, parent_idx, cur_idx, accel, overanchor, overtag, false, false)
    }


    #[inline (always)]
    fn read_node<D: Datum + 'static, R: Read<Datum=D>> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>,
        ctx: &mut Context<D>,
        indent: usize,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        accel: Option<(Token, usize, usize)>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>
    ) -> Result<(), ReadError> {
        self.read_node_ (reader, callback, ctx, indent, level, parent_idx, cur_idx, accel, overanchor, overtag, false, false)
    }


    fn read_node_<D: Datum + 'static, R: Read<Datum=D>> (
        &mut self,
        reader: &mut R,
        callback: &mut FnMut (Block<D>) -> Result<(), Cow<'static, str>>,
        ctx: &mut Context<D>,
        indent: usize,
        level: usize,
        parent_idx: usize,
        cur_idx: &mut usize,
        mut accel: Option<(Token, usize, usize)>,
        overanchor: &mut Option<Marker>,
        overtag: &mut Option<Marker>,
        flow: bool,
        map_block_val: bool,
        // _seq_block_indent: Option<usize>
    ) -> Result<(), ReadError> {
        // self.timer.stamp ("rn");
        // self.timer.stamp ("rn->context");
        let mut ctx = &mut Context::new (ctx, ContextKind::Node, self.cursor, level);

        // let ctx = &mut ctx;
        // self.timer.stamp ("rn->context");
        // self.timer.stamp ("rn->vars");

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
        let mut flow_opt: Option<Block<D>> = None;

        let mut map_block_val_pass: bool = false;

        let float_start = if let Some ( (ref token, _, _) ) = accel {
            match *token {
                Token::Anchor    |
                Token::TagHandle => true,
                _ => false
            }
        } else { false };

        // self.timer.stamp ("rn->vars");
        // self.timer.stamp ("rn->loop");

        'top: loop {
            if let Some ( (token, len, chars) ) = if accel.is_none () { tokenizer::get_token (reader) } else { accel.take () } {
                loop {
                    match token {
                        /*
                        Token::BOM32BE |
                        Token::BOM32LE |
                        Token::BOM16BE |
                        Token::BOM16LE |
                        */
                        Token::BOM8 => {
                            // try! (self.check_bom (reader, callback, level, parent_idx, len));
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
                            // if scan_one_at (len, reader, &tokenizer::spaces_and_line_breakers).is_some () {
                            if tokenizer::scan_one_spaces_and_line_breakers (reader, len) > 0 {
                                break 'top;
                            // } else if flow && tokenizer::cset.comma.read_at (len, reader).is_some () {
                            } else if flow && /* reader.contains_copy_at (tokenizer::cset.comma, len)*/ reader.byte_at (b',', len) {
                                break 'top;
                            } else {
                                match flow_opt {
                                    Some (Block { id, cargo: BlockType::Node (Node {
                                        anchor: _,
                                        tag: _,
                                        content: NodeKind::Scalar (mut chunk)
                                    }) }) => {
                                        try! (self.yield_block (Block::new (id.clone (), BlockType::Node (Node {
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

                                        self.rtrim (ctx, &mut chunk);

                                        let idx = self.get_idx ();
                                        try! (self.yield_block (Block::new (
                                            Id { level: level + 1, parent: flow_idx, index: idx },
                                            BlockType::Literal (chunk)
                                        ), callback));
                                    },
                                    _ => ()
                                };

                                let marker = self.consume (ctx, reader, callback, len, chars) ?;
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

                            let mut ctxptr: Option<&mut Context<D>> = Some(&mut ctx);

                            loop {
                                if ctxptr.is_none () { break; }
                                let parent = if let Some (parent) = ctxptr.take ().unwrap ().get_parent () { parent } else { break; };

                                match parent.kind {
                                    ContextKind::SequenceFlow |
                                    ContextKind::MappingFlow => { pass = true; }
                                    ContextKind::MappingBlock if parent.level == level => {
                                        pass = chars > parent.indent;
                                    }
                                    _ => { ctxptr = Some (parent); continue }
                                };

                                break;
                            }

                            if !pass && chars < indent { break 'top; }

                            // spare one space / to be used in a literal block to join pieces
                            let (len_, chars_) = if not (state, AFTER_NEWLINE) {
                                (len - 1, if chars > 0 { chars - 1 } else { 0 })
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
                                // let mut chunk: Chunk = Chunk::with_capacity (tokenizer::cset.space.len ());
                                // chunk.push_slice (tokenizer::spaces[0].as_slice ());
                                // let rune: Rune = tokenizer::cset.space.into ();

                                match flow_opt {
                                    Some (Block { id, cargo: BlockType::Node (Node {
                                        anchor: _,
                                        tag: _,
                                        content: NodeKind::Scalar (mut chunk)
                                    }) }) => {
                                        try! (self.yield_block (Block::new (id.clone (), BlockType::Node (Node {
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

                                        self.rtrim (ctx, &mut chunk);

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
                                    BlockType::Byte (b' ', 1)
                                    // BlockType::Literal (chunk)
                                ), callback));
                                *cur_idx = idx;
                            }

                            on (&mut state, AFTER_SPACE | INDENT_PASSED);
                        }


                        _ if
                            flow_idx > 0
                            && (indent == 0
                                || (if let Some (kind) = ctx.get_parent_kind () {
                                    match kind {
                                        ContextKind::MappingFlow | ContextKind::SequenceFlow => true,
                                        _ => false
                                    }
                                } else { false }))
                            && is (state, NEWLINE_PASSED) && not (state, INDENT_PASSED) => { on (&mut state, INDENT_PASSED); }


                        Token::Comment if flow_idx > 0 && is (state, AFTER_SPACE) => {
                            self.skip (reader, len, chars);
                            self.nl ();
                            on (&mut state, NEWLINE_PASSED);
                            off (&mut state, INDENT_PASSED);
                        }


                        Token::Newline if flow_idx > 0 && not (state, NEWLINE_PASSED) => {
                            // skip only one newline

                            let len_ = match reader.get_byte_at_start () {
                                Some (b'\n') => 1,
                                Some (b'\r') => if let Some (b'\n') = reader.get_byte_at (1) { 2 } else { 1 },
                                _ => len
                            };

                            /*
                            let len_ =
                                // if len >= tokenizer::cset.crlf.len () && tokenizer::cset.crlf.read (reader).is_some () {
                                if len >= tokenizer::cset.crlf.len () && reader.contains_copy_at_start (tokenizer::cset.crlf) {
                                    tokenizer::cset.crlf.len ()
                                }
                                // else if len >= tokenizer::cset.line_feed.len () && tokenizer::cset.line_feed.read (reader).is_some () {
                                else if len >= tokenizer::cset.line_feed.len () && reader.contains_copy_at_start (tokenizer::cset.line_feed) {
                                    tokenizer::cset.line_feed.len ()
                                }
                                // else if len >= tokenizer::cset.carriage_return.len () && tokenizer::cset.carriage_return.read (reader).is_some () {
                                else if len >= tokenizer::cset.carriage_return.len () && reader.contains_copy_at_start (tokenizer::cset.carriage_return) {
                                    tokenizer::cset.carriage_return.len ()
                                } else { len };
                            */

                            let chars_ = if chars > 0 { 1 } else { 0 };

                            self.skip (reader, len_, chars_);
                            self.nl ();

                            on (&mut state, NEWLINE_PASSED | AFTER_SPACE);
                            off (&mut state, INDENT_PASSED);
                        }


                        _ if flow_idx > 0 && is (state, INDENT_PASSED) => {
                            let mut skip = false;

                            {
                                let mut ctxptr: Option<&mut Context<D>> = Some (&mut ctx);
                                loop {
                                    // if ctxptr.parent.is_none () { break; }
                                    // ctxptr = ctxptr.parent.as_ref ().unwrap ();
                                    if ctxptr.is_none () { break; }
                                    let parent = if let Some (parent) = ctxptr.take ().unwrap ().get_parent () { parent } else { break; };

                                    match parent.kind {
                                        ContextKind::SequenceFlow => {
                                            // skip = self.check_next_is_right_square (reader, 0, false);
                                            // skip = self.check_next_is_char (tokenizer::cset.bracket_square_right, reader, 0, false);
                                            skip = self.check_next_is_byte (b']', reader, 0, false);
                                        }
                                        ContextKind::MappingFlow => {
                                            // skip = self.check_next_is_right_curly (reader, 0, false);
                                            // skip = self.check_next_is_char (tokenizer::cset.bracket_curly_right, reader, 0, false);
                                            skip = self.check_next_is_byte (b'}', reader, 0, false);
                                        }
                                        _ => { ctxptr = Some (parent); continue }
                                    };

                                    break;
                                }
                            }

                            if skip {
                                self.consume (ctx, reader, callback, len, chars) ?;
                                break;
                            }

                            match flow_opt {
                                Some (Block { id, cargo: BlockType::Node (Node {
                                    anchor: _,
                                    tag: _,
                                    content: NodeKind::Scalar (mut chunk)
                                }) }) => {
                                    try! (self.yield_block (Block::new (id.clone (), BlockType::Node (Node {
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

                                    self.rtrim (ctx, &mut chunk);

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
                                    let marker = self.consume (ctx, reader, callback, len, chars) ?;
                                    let idx = self.get_idx ();
                                    try! (self.yield_block (Block::new (
                                        Id { level: level + 1, parent: flow_idx, index: idx },
                                        BlockType::Literal (marker)
                                    ), callback));
                                    *cur_idx = idx;

                                    on (&mut state, AFTER_SPACE | INDENT_PASSED);
                                }

                                Token::Newline => {
                                    let marker = self.consume (ctx, reader, callback, len, chars) ?;
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
                                    // let len = tokenizer::cset.hashtag.len ();
                                    let marker = self.consume (ctx, reader, callback, 1, 1) ?;

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
                                        // let mut chunk: Chunk = Chunk::with_capacity (tokenizer::cset.space.len ());
                                        // chunk.push_slice (tokenizer::cset.space.as_slice ());

                                        let idx = self.get_idx ();
                                        // let rune: Rune = tokenizer::cset.space.into ();
                                        try! (self.yield_block (Block::new (
                                            Id { level: level + 1, parent: flow_idx, index: idx },
                                            BlockType::Byte (b' ', 1)
                                            // BlockType::Literal (chunk)
                                        ), callback));
                                        *cur_idx = idx;
                                    }

                                    let mut chunk = match token {
                                        Token::Directive => {
                                            // let (len, _) = scan_until_at (0, reader, &tokenizer::raw_stops);
                                            let len = tokenizer::raw_stops (reader);
                                            self.consume (ctx, reader, callback, len, 1) ?
                                        }
                                        _ => self.consume (ctx, reader, callback, len, chars) ?
                                    };
                                    
                                    self.rtrim (ctx, &mut chunk);

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
                                Cow::from ("@ character is reserved and may not be used to start a plain scalar")
                            ));
                        }


                        Token::ReservedGraveAccent => {
                            let idx = self.get_idx ();
                            try! (self.yield_error (
                                callback,
                                Id { level: level, parent: parent_idx, index: idx },
                                Cow::from ("` character is reserved and may not be used to start a plain scalar")
                            ));
                        }


                        Token::Indent if is (state, LINE_BREAK) => {
                            // let is_hyphen = tokenizer::cset.hyphen_minus.read_at (len, reader).is_some ();
                            // let is_hyphen = reader.contains_copy_at (tokenizer::cset.hyphen_minus, len);
                            let is_hyphen = reader.byte_at (b'-', len);
                            let yes = if float_start {
                                if is_hyphen {
                                    if indent == 0 { true } else {
                                        let mut result = true;
                                        {
                                            let mut ctxptr: Option<&mut Context<D>> = Some (&mut ctx);
                                            loop {
                                                // if ctxptr.parent.is_none () { break; }
                                                // ctxptr = ctxptr.parent.as_ref ().unwrap ();
                                                if ctxptr.is_none () { break; }
                                                let parent = if let Some (parent) = ctxptr.take ().unwrap ().get_parent () { parent } else { break; };

                                                match parent.kind {
                                                    ContextKind::SequenceBlock => {
                                                        result = self.cursor + len > parent.indent;
                                                    },
                                                    _ => { ctxptr = Some (parent); continue }
                                                };

                                                break;
                                            }
                                        }
                                        result
                                    }
                                // } else if tokenizer::cset.vertical_bar.read_at (len, reader).is_some () {
                                } else if /* reader.contains_copy_at (tokenizer::cset.vertical_bar, len) */ reader.byte_at (b'|', len) {
                                    true 
                                // } else if tokenizer::cset.greater_than.read_at (len, reader).is_some () {
                                } else if /* reader.contains_copy_at (tokenizer::cset.greater_than, len) */ reader.byte_at (b'>', len) {
                                    true
                                } else {
                                    // if ctx.parent.is_some () {
                                    if let Some (par) = ctx.get_parent () {
                                        // let par = ctx.parent.as_ref ().unwrap ();
                                        match par.kind {
                                            ContextKind::MappingBlock => {
                                                // if par.parent.is_some () {
                                                if let Some (par) = par.get_parent () {
                                                    // let par = par.parent.as_ref ().unwrap ();
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

                                    try! (self.read_seq_block (reader, callback, &mut ctx, level + 1, idx, None));
                                } else {
                                    try! (self.read_layer_expected (
                                        reader,
                                        callback,
                                        &mut ctx,
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

                                    // let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                                    // let (anchor, tag) = if self.check_next_is_char (tokenizer::cset.colon, reader, indent, false) {
                                    let (anchor, tag) = if self.check_next_is_byte (b':', reader, indent, false) {
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

                                // let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                                // let (anchor, tag) = if self.check_next_is_char (tokenizer::cset.colon, reader, indent, false) {
                                let (anchor, tag) = if self.check_next_is_byte (b':', reader, indent, false) {
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
                                        let mut ctxptr: Option<&mut Context<D>> = Some (&mut ctx);
                                        loop {
                                            // if ctxptr.parent.is_none () { break; }
                                            // ctxptr = ctxptr.parent.as_ref ().unwrap ();

                                            if ctxptr.is_none () { break; }
                                            let parent = if let Some (parent) = ctxptr.take ().unwrap ().get_parent () { parent } else { break; };

                                            match parent.kind {
                                                ContextKind::SequenceBlock => {
                                                    result = self.cursor > parent.indent;
                                                },
                                                _ => { ctxptr = Some (parent); continue }
                                            };

                                            break;
                                        }
                                        result
                                    },
                                    _ => {
                                        let mut result = false;
                                        let mut ctxptr: Option<&mut Context<D>> = Some (&mut ctx);
                                        loop {
                                            if ctxptr.is_none () { break; }
                                            let parent = if let Some (parent) = ctxptr.take ().unwrap ().get_parent () { parent } else { break; };

                                            match parent.kind {
                                                ContextKind::Layer => { result = parent.indent == 0; }
                                                _ => ()
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

                                        try! (self.read_seq_block (reader, callback, &mut ctx, level + 1, idx, Some ( (token, len, chars) )));
                                    },
                                    _ => try! (self.read_layer_expected (
                                        reader,
                                        callback,
                                        &mut ctx,
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

                                    // let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                                    // let (anchor, tag) = if self.check_next_is_char (tokenizer::cset.colon, reader, indent, false) {
                                    let (anchor, tag) = if self.check_next_is_byte (b':', reader, indent, false) {
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

                                // let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                                // let (anchor, tag) = if self.check_next_is_char (tokenizer::cset.colon, reader, indent, false) {
                                let (anchor, tag) = if self.check_next_is_byte (b':', reader, indent, false) {
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
                            // let ast_len = tokenizer::cset.asterisk.len ();
                            self.skip (reader, 1, 0);
                            let marker = self.consume (ctx, reader, callback, len - 1, chars) ?;

                            let idx = self.get_idx ();
                            try! (self.yield_block (Block::new (
                                Id { level: level, parent: parent_idx, index: idx },
                                BlockType::Alias (marker)
                            ), callback));

                            *cur_idx = idx;

                            on (&mut state, ALIAS_READ);
                        }


                        Token::Anchor if anchor.is_none () => {
                            // let amp_len = tokenizer::cset.ampersand.len ();
                            self.skip (reader, 1, 0);
                            anchor = Some ( self.consume (ctx, reader, callback, len - 1, chars) ? );
                        }


                        Token::TagHandle if tag.is_none () => {
                            tag = Some ( self.consume (ctx, reader, callback, len, chars) ? );
                        }


                        Token::Dash => {
                            let idx = self.get_idx ();

                            try! (self.yield_block (Block::new (Id { level: level, parent: parent_idx, index: idx }, BlockType::Node (Node {
                                anchor: if anchor.is_none () { overanchor.take () } else { anchor.take () },
                                tag: if tag.is_none () { overtag.take () } else { tag.take () },
                                content: NodeKind::Sequence
                            })), callback));

                            *cur_idx = idx;

                            return self.read_seq_block (reader, callback, &mut ctx, level + 1, idx, Some ( (token, len, chars) ));
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

                            return self.read_map_block_explicit (reader, callback, &mut ctx, level + 1, idx, Some ( (token, len, chars) ))
                        }


                        Token::DictionaryStart |
                        Token::SequenceStart => {
                            self.skip (reader, len, chars);

                            let new_idx = self.get_idx ();

                            *cur_idx = new_idx;

                            return match token {
                                Token::SequenceStart => self.read_seq_flow (reader, callback, &mut ctx, new_idx, level, parent_idx, indent, anchor, tag, overanchor, overtag),
                                _ => self.read_map_flow (reader, callback, &mut ctx, new_idx, level, parent_idx, indent, anchor, tag, overanchor, overtag)
                            }
                        }


                        Token::StringSingle |
                        Token::StringDouble => {
                            let marker = self.consume (ctx, reader, callback, len, chars) ?;
                            let idx = self.get_idx ();

                            *cur_idx = idx;

                            // let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                            // let (anchor, tag) = if self.check_next_is_char (tokenizer::cset.colon, reader, indent, false) {
                            let (anchor, tag) = if self.check_next_is_byte (b':', reader, indent, false) {
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
                                &mut ctx,
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
                            // if scan_one_at (len, reader, &tokenizer::spaces_and_line_breakers).is_some () {
                            if tokenizer::scan_one_spaces_and_line_breakers (reader, len) > 0 {
                                break 'top;
                            }

                            map_block_val_pass = true;
                            continue;
                        }


                        Token::Colon |
                        Token::Comma if tag.is_none () && anchor.is_none () => {
                            flow_idx = self.get_idx ();
                            *cur_idx = flow_idx;

                            let mut marker = self.consume (ctx, reader, callback, len, chars) ?;

                            // let data = ctx.get_data ();

                            let blen = ctx.get_data ().marker_len (&marker);
                            self.rtrim (ctx, &mut marker);
                            let alen = ctx.get_data ().marker_len (&marker);

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

                            let mut marker = self.consume (ctx, reader, callback, len, chars) ?;

                            // let data = ctx.get_data ();

                            let blen = ctx.get_data ().marker_len (&marker);
                            self.rtrim (ctx, &mut marker);
                            let alen = ctx.get_data ().marker_len (&marker);

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

                            // let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                            // let (anchor, tag) = if self.check_next_is_char (tokenizer::cset.colon, reader, indent, false) {
                            let (anchor, tag) = if self.check_next_is_byte (b':', reader, indent, false) {
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
                                Cow::from (format! (r"Unexpected token ({}:{})", file! (), line! ()))
                            ).unwrap_err ())
                        }
                    }
                    break;
                }
            } else {
                if flow_idx == 0 && not (state, ALIAS_READ) {
                    let idx = self.get_idx ();

                    *cur_idx = idx;

                    // let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                    // let (anchor, tag) = if self.check_next_is_char (tokenizer::cset.colon, reader, indent, false) {
                    let (anchor, tag) = if self.check_next_is_byte (b':', reader, indent, false) {
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
        // self.timer.stamp ("rn->loop");

        // self.timer.stamp ("rn->flopt");
        match flow_opt {
            Some (Block { id, cargo: BlockType::Node (Node {
                anchor: _,
                tag: _,
                content: NodeKind::Scalar (mut chunk)
            }) }) => {
                // self.timer.stamp ("rn->flopt<-scalar");
                // self.timer.stamp ("rn->flopt<-scalar->rtrim");
                self.rtrim (ctx, &mut chunk);
                // self.timer.stamp ("rn->flopt<-scalar->rtrim");

                // self.timer.stamp ("rn->flopt<-scalar->cnis");
                // let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                // let (anchor, tag) = if self.check_next_is_char (tokenizer::cset.colon, reader, indent, false) {
                let (anchor, tag) = if self.check_next_is_byte (b':', reader, indent, false) {
                    (anchor, tag)
                } else {
                    (
                        if anchor.is_none () { overanchor.take () } else { anchor },
                        if tag.is_none () { overtag.take () } else { tag }
                    )
                };
                // self.timer.stamp ("rn->flopt<-scalar->cnis");

                *cur_idx = id.index;

                self.yield_block (Block::new (id, BlockType::Node (Node {
                    anchor: anchor,
                    tag: tag,
                    content: NodeKind::Scalar (chunk)
                })), callback) ?;
                // self.timer.stamp ("rn->flopt<-scalar");
            }

            Some (Block { id, cargo: BlockType::Node (Node {
                anchor: _,
                tag: _,
                content: NodeKind::LiteralBlockClose
            }) }) => {
                // self.timer.stamp ("rn->flopt<-block");
                // let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                // let (anchor, tag) = if self.check_next_is_char (tokenizer::cset.colon, reader, indent, false) {
                let (anchor, tag) = if self.check_next_is_byte (b':', reader, indent, false) {
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
                // self.timer.stamp ("rn->flopt<-block");
            }

            Some (_) => unreachable! (),

            None => {
                // self.timer.stamp ("rn->flopt<-none");
                if tag.is_some () || anchor.is_some () {
                    let idx = self.get_idx ();

                    *cur_idx = idx;

                    // let (anchor, tag) = if self.check_next_is_colon (reader, indent, false) {
                    // let (anchor, tag) = if self.check_next_is_char (tokenizer::cset.colon, reader, indent, false) {
                    let (anchor, tag) = if self.check_next_is_byte (b':', reader, indent, false) {
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
                // self.timer.stamp ("rn->flopt<-none");
            }
        };
        // self.timer.stamp ("rn->flopt");

        // self.timer.stamp ("rn");

        Ok ( () )
    }


    fn rtrim<D: Datum + 'static> (&self, ctx: &mut Context<D>, marker: &mut Marker) {
        let data = ctx.get_data ();
        let rtrimsize: usize = {
            let chunk = data.chunk (marker);;
            let chunk_slice = chunk.as_slice ();
            let mut ptr = chunk_slice.len ();

            loop {
                if ptr == 0 { break; }
                match chunk_slice.get (ptr - 1).map (|b| *b) {
                    Some (b' ') |
                    Some (b'\n') |
                    Some (b'\t') |
                    Some (b'\r') => ptr -= 1,
                    _ => break
                }
            }

            chunk_slice.len () - ptr
        };

        if rtrimsize > 0 {
            let trlen = data.marker_len (marker) - rtrimsize;
            *marker = data.resize (marker.clone (), trlen);
        }
    }


    fn check_next_is_byte<D: Datum + 'static, R: Read<Datum=D>> (&self, byte: u8, reader: &mut R, indent: usize, mut newlined: bool) -> bool {
        let mut pos = 0;
        let mut ind = 0;

        loop {
            match reader.get_byte_at (pos) {
                Some (b) if b == byte => {
                    if newlined { return ind >= indent; }
                    return true;
                }
                Some (b' ') => {
                    pos += 1;
                    ind += 1;
                }
                Some (b'\n') |
                Some (b'\r') => {
                    pos += 1;
                    ind = 0;
                    newlined = true;
                }
                Some (b'#') => {
                    pos += tokenizer::line_at (reader, pos) + 1;
                    ind = 0;
                    newlined = true;
                }
                _ => break
            };
        }

        false
    }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use super::skimmer::reader::SliceReader;

    // use tokenizer::Tokenizer;

    // use txt::get_charset_utf8;

    use std::sync::mpsc::channel;



    #[test]
    fn test_directives () {
        let src = "%YAML 1.2\n%TAG !e! tag://example.com,2015:testapp/\n---\n...";

        let (sender, receiver) = channel ();
        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));
        let mut data = Data::with_capacity (16);

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
        ).unwrap_or_else (|err| { assert! (false, format! ("Unexpected result: {}", err)); });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (0, block.id.index);

            if let BlockType::Datum (datum) = block.cargo { data.push (datum); } else { assert! (false, "unexpected cargo") }
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
                let handle = data.chunk (&handle);
                let tag = data.chunk (&tag);
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
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

        let mut data = Data::with_capacity (16);
        let (sender, receiver) = channel ();
        let mut reader = Reader::new (); // ::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new (src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
        ).unwrap_or_else (|err| { assert! (false, format! ("Unexpected result: {:?}", err)); });


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (0, block.id.index);

            if let BlockType::Datum (datum) = block.cargo { data.push (datum); } else { assert! (false, "unexpected cargo") }
        } else { assert! (false, "no datum") }


        if let Ok (block) = receiver.try_recv () {
            assert_eq! (0, block.id.level);
            assert_eq! (0, block.id.parent);
            assert_eq! (1, block.id.index);

            if let BlockType::DirectiveTag ( (handle, tag) ) = block.cargo {
                let handle = data.chunk (&handle);
                let tag = data.chunk (&tag);
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
