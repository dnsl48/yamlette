extern crate skimmer;

use self::skimmer::{ Chunk, Data, Datum, Marker };
// use self::skimmer::symbol::{ Combo, CopySymbol, Symbol };


use model::{ Model, TaggedValue, Schema };
use model::yamlette::literal::{ self, Literal };
use reader::{ Id, Block, BlockType, NodeKind };
use sage::conveyor::Clue;
use sage::YamlVersion;

use std::borrow::Cow;
use std::io;
use std::sync::Arc;
use std::sync::mpsc::{ sync_channel, SyncSender, Receiver };
use std::thread::{ Builder, JoinHandle };




#[derive (Debug)]
pub enum Response {
    Error (Id, Cow<'static, str>),
    TagHandle (Id, Cow<'static, str>, Cow<'static, str>),
    Alias (Id, String),
    Node (Id, Option<String>, Node)
}



#[derive (Debug)]
pub enum Node {
    MetaMap (Option<String>, Option<Id>),
    MetaSeq (Option<String>),

    Dictionary (Cow<'static, str>, Option<Id>),
    Sequence (Cow<'static, str>),

    Scalar (TaggedValue),
    Literal (String)
}



#[derive (Debug)]
pub enum Message<D> {
    Request (Request<D>),
    Signal (Signal<D>)
}



#[derive (Debug)]
pub enum Request<D> {
    ReadBlock (Block<D>),
    ReadDirectiveTag ( Id, Marker, Marker ),
    ReadLiteralBlock ( Id, Option<Marker>, Option<Marker>, Vec<Result<Marker, (u8, usize)>> )
}



#[derive (Debug)]
pub enum Signal<D> {
    Reset,
    Datum (D),
    TagHandle (Arc<(Cow<'static, str>, Cow<'static, str>)>),
    Terminate,
    Version (YamlVersion)
}




pub struct Ant<S, D> {
    idx: u8,
    cin: Receiver<Message<D>>,
    out: SyncSender<(u8, Clue)>,

    data: Data<D>,
    schema: Arc<S>,
    tag_handles: Vec<Arc<(Cow<'static, str>, Cow<'static, str>)>>,

    yaml_version: YamlVersion
}



impl<S, D> Ant<S, D>
  where
    S: Schema + 'static,
    D: Datum + Sync + Send + 'static
{
    pub fn run (
        idx: u8,
        out: SyncSender<(u8, Clue)>,
        schema: Arc<S>,
        yaml_version: YamlVersion,
        tag_handles: Vec<Arc<(Cow<'static, str>, Cow<'static, str>)>>
    )
        -> io::Result<(SyncSender<Message<D>>, JoinHandle<()>)>
    {
        let (to_me, cin) = sync_channel (1);

        let handle = try! (Builder::new ().name (format! ("sage_ant_{}", idx)).spawn (move || {
            ( Ant {
                idx: idx,
                cin: cin,
                out: out,

                data: Data::with_capacity (4),
                schema: schema,
                tag_handles: tag_handles,

                yaml_version: yaml_version
            } ).execute ()
        }));

        Ok ( (to_me, handle) )
    }


    pub fn execute (mut self) -> () {
        let schema: &S = &self.schema.as_ref ();

        let model_literal_opt: Option<&Model> = schema.look_up_model (literal::TAG);

        let model_literal: &Model = if model_literal_opt.is_some ()
                         && model_literal_opt.as_ref ().unwrap ().is_decodable ()
                         && model_literal_opt.as_ref ().unwrap ().is_encodable ()
            { model_literal_opt.unwrap () }
        else
            { panic! ("Undefined literal model") };

        let model_literal: &Literal = if let Some (model) = model_literal.as_any ().downcast_ref::<Literal> () {
            model
        } else { panic! ("Cannot downcast Literal model") };


        'top: loop {
            if let Ok (msg) = self.cin.recv () {
                match msg {
                    Message::Signal ( signal ) => match signal {
                        Signal::Terminate => break 'top,
                        Signal::Datum (arc) => self.data.push (arc),
                        Signal::Version (ver) => self.yaml_version = ver,
                        Signal::TagHandle ( arc ) => Self::set_tag_handle (&mut self.tag_handles, arc),
                        Signal::Reset => self.tag_handles.clear ()
                    },

                    Message::Request (request) => { self.handle (request, model_literal).ok (); }
                }
            } else { break 'top }
        }

        ()
    }


    fn handle (
        &self,
        request: Request<D>,
        model_literal: &Literal
    ) -> Result<(), ()> {
        match request {
            Request::ReadBlock ( block ) => self.read_block ( block, model_literal ),
            Request::ReadDirectiveTag ( id, shorthand, prefix ) => self.read_directive_tag (id, shorthand, prefix, model_literal),
            Request::ReadLiteralBlock ( id, anchor, tag, vec ) => self.read_literal_block ( id, anchor, tag, Err (vec), model_literal )
        }
    }


    fn decode (&self, model: &Model, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        match self.yaml_version {
            YamlVersion::V1x1 => model.decode11 (explicit, value),
            YamlVersion::V1x2 => model.decode   (explicit, value)
        }
    }


    fn read_directive_tag (
        &self,
        id: Id,
        shorthand: Marker,
        prefix: Marker,
        model_literal: &Literal
    ) -> Result<(), ()> {
        let shorthand = self.data.chunk (&shorthand);
        let shorthand: Result<String, ()> = model_literal.bytes_to_string (shorthand.as_slice ());

        let prefix = self.data.chunk (&prefix);
        let prefix: Result<String, ()> = model_literal.bytes_to_string (prefix.as_slice ());

        if shorthand.is_err () {
            self.out.send ((self.idx, Clue::Response (Response::Error ( id, Cow::from ("Cannot decode shorthand") )))).unwrap ();
        } else if prefix.is_err () {
            self.out.send ((self.idx, Clue::Response (Response::Error ( id, Cow::from ("Cannot decode prefix") )))).unwrap ();
        } else {
            self.out.send ((self.idx, Clue::Response (Response::TagHandle (id, Cow::from (shorthand.unwrap ()), Cow::from (prefix.unwrap ()))))).unwrap ();
        }

        Ok ( () )
    }


    fn read_literal_block (
        &self,
        id: Id,
        anchor: Option<Marker>,
        tag: Option<Marker>,
        vec: Result<Marker, Vec<Result<Marker, (u8, usize)>>>,
        model_literal: &Literal
    ) -> Result<(), ()> {
        self.read_scalar (id, anchor, tag, model_literal, vec)
    }


    fn read_block (
        &self,
        block: Block<D>,
        model_literal: &Literal
    ) -> Result<(), ()> {
        match block.cargo {
            BlockType::Alias ( marker ) => self.read_alias (block.id, marker, model_literal),

            BlockType::BlockMap ( firstborn_id, anchor, tag ) => self.read_map ( block.id, Some (firstborn_id), anchor, tag, model_literal ),

            BlockType::Literal ( marker ) => self.read_literal (block.id, marker, model_literal),
            BlockType::Byte ( byte, amount ) => self.read_byte (block.id, model_literal, byte, amount),

            BlockType::Node ( node ) => match node.content {
                NodeKind::LiteralBlockOpen |
                NodeKind::LiteralBlockClose => unreachable! (),

                NodeKind::Null => self.read_null (block.id, node.anchor, node.tag, model_literal),

                NodeKind::Mapping => self.read_map ( block.id, None, node.anchor, node.tag, model_literal ),
                NodeKind::Sequence => self.read_seq ( block.id, node.anchor, node.tag, model_literal ),

                NodeKind::Scalar ( marker ) => self.read_scalar (block.id, node.anchor, node.tag, model_literal, Ok (marker))
            },

            BlockType::Datum (..) => unreachable! (),
            BlockType::DirectiveTag ( .. ) => unreachable! (),
            BlockType::DirectiveYaml ( .. ) => unreachable! (),
            BlockType::DocStart => unreachable! (),
            BlockType::DocEnd => unreachable! (),
            BlockType::Error ( .. ) => unreachable! (),
            BlockType::Warning ( .. ) => unreachable! (),
            BlockType::StreamEnd => unreachable! ()
        }
    }


    fn read_alias (&self, id: Id, marker: Marker, model_literal: &Literal) -> Result<(), ()> {
        let chunk = self.data.chunk (&marker);
        let alias = model_literal.bytes_to_string (chunk.as_slice ());

        match alias {
            Ok (string) => self.out.send ((self.idx, Clue::Response (Response::Alias (id, string)))),
            Err ( () ) => self.out.send ((self.idx, Clue::Response (Response::Error (id, Cow::from ("Cannot decode alias")))))
        }.unwrap ();

        Ok ( () )
    }


    fn read_literal (&self, id: Id, marker: Marker, model_literal: &Literal) -> Result<(), ()> {
        let chunk = self.data.chunk (&marker);
        let literal = model_literal.bytes_to_string (chunk.as_slice ());

        match literal {
            Ok (string) => self.out.send ((self.idx, Clue::Response (Response::Node (id, None, Node::Literal (string))))),
            Err ( () ) => self.out.send ((self.idx, Clue::Response (Response::Error (id, Cow::from ("Cannot decode literal")))))
        }.unwrap ();

        Ok ( () )
    }


    fn read_byte (&self, id: Id, model_literal: &Literal, byte: u8, amount: usize) -> Result<(), ()> {
        let literal = model_literal.bytes_to_string_times (&[byte], amount);

        match literal {
            Ok (string) => self.out.send ((self.idx, Clue::Response (Response::Node (id, None, Node::Literal (string))))),
            Err ( () ) => self.out.send ((self.idx, Clue::Response (Response::Error (id, Cow::from ("Cannot decode literal")))))
        }.unwrap ();

        Ok ( () )
    }


    fn read_anchor (&self, block_id: &Id, model_literal: &Literal, anchor: Option<Marker>) -> Result<Option<String>, ()> {
        if let Some (anchor) = anchor {
            let chunk = self.data.chunk (&anchor);
            let result = model_literal.bytes_to_string (chunk.as_slice ());

            match result {
                Ok (string) => Ok (Some (string)),
                Err ( () ) => {
                    self.out.send ((self.idx, Clue::Response (Response::Error (block_id.clone (), Cow::from ("Cannot decode anchor"))))).unwrap ();
                    return Err ( () );
                }
            }
        } else { Ok (None) }
    }


    fn read_tag (&self, block_id: &Id, model_literal: &Literal, tag: Option<Marker>) -> Result<Option<String>, ()> {
        let tag = if let Some (tag) = tag {
            let chunk = self.data.chunk (&tag);
            let result = model_literal.bytes_to_string (chunk.as_slice ());

            match result {
                Err ( () ) => {
                    self.out.send ((self.idx, Clue::Response (Response::Error (block_id.clone (), Cow::from ("Cannot decode tag"))))).unwrap ();
                    return Err ( () )
                }
                Ok (tag) => Some (tag)
            }
        } else { None };

        Ok (tag)
    }


    fn read_model<F: FnMut (&Model, bool) -> bool> (&self, tag: Option<String>, block_id: &Id, predicate: F) -> Result<&Model, ()> {
        let tag = if let Some (tag) = tag { tag } else { String::with_capacity (0) };

        let model: Option<(&Model, bool)> = self.lookup_model (&tag, predicate);

        match model {
            None => {
                self.out.send ((self.idx, Clue::Response (Response::Error (block_id.clone (), Cow::from (format! ("Could not find appropriate model (tag {})", tag)))))).unwrap ();
                Err ( () )
            }

            Some ( (model, _) ) => Ok (model)
        }
    }


    fn read_map (
        &self,
        block_id: Id,
        firstborn_id: Option<Id>,
        anchor: Option<Marker>,
        tag: Option<Marker>,
        model_literal: &Literal
    ) -> Result<(), ()> {
        let anchor: Option<String> = try! (self.read_anchor (&block_id, model_literal, anchor));
        let tag: Option<String> = try! (self.read_tag (&block_id, model_literal, tag));

        let model: Option<(&Model, bool)> = {
            let empty = String::with_capacity (0);
            let tag: &String = if let Some (ref tag) = tag { tag } else { &empty };

            self.lookup_model (&tag, |m, _| { m.is_dictionary () })
        };

        let node = match model {
            Some ( (model, _) ) => Node::Dictionary (model.get_tag ().clone (), firstborn_id),
            None => Node::MetaMap (tag, firstborn_id)
        };

        let response = Response::Node (block_id.clone (), anchor, node);

        self.out.send ((self.idx, Clue::Response (response))).unwrap ();

        Ok ( () )
    }


    fn read_seq (
        &self,
        block_id: Id,
        anchor: Option<Marker>,
        tag: Option<Marker>,
        model_literal: &Literal
    ) -> Result<(), ()> {
        let anchor: Option<String> = try! (self.read_anchor (&block_id, model_literal, anchor));
        let tag: Option<String> = try! (self.read_tag (&block_id, model_literal, tag));

        let model: Option<(&Model, bool)> = {
            let empty = String::with_capacity (0);
            let tag: &String = if let Some (ref tag) = tag { tag } else { &empty };

            self.lookup_model (&tag, |m, _| { m.is_sequence () })
        };


        let node = match model {
            Some ( (model, _) ) => Node::Sequence (model.get_tag ().clone ()),
            None => Node::MetaSeq (tag)
        };


        let response = Response::Node (block_id, anchor, node);

        self.out.send ((self.idx, Clue::Response (response))).unwrap ();

        Ok ( () )
    }


    fn read_null (&self, block_id: Id, anchor: Option<Marker>, tag: Option<Marker>, model_literal: &Literal) -> Result<(), ()> {
        let anchor: Option<String> = try! (self.read_anchor (&block_id, model_literal, anchor));
        let tag: Option<String> = try! (self.read_tag (&block_id, model_literal, tag));

        let model = try! (self.read_model (tag, &block_id, |m, _| { !m.is_collection () && m.has_default () }));

        let node = Node::Scalar (model.get_default ());
        let response = Response::Node (block_id, anchor, node);

        self.out.send ((self.idx, Clue::Response (response))).unwrap ();

        Ok ( () )
    }


    fn read_scalar (&self, block_id: Id, anchor: Option<Marker>, tag: Option<Marker>, model_literal: &Literal, marker: Result<Marker, Vec<Result<Marker, (u8, usize)>>>) -> Result<(), ()> {
        let anchor: Option<String> = try! (self.read_anchor (&block_id, model_literal, anchor));
        let tag: Option<String> = try! (self.read_tag (&block_id, model_literal, tag));

        let mut decoded: Result<TaggedValue, ()> = Err ( () );

        let chunk = match marker {
            Ok (marker) => self.data.chunk (&marker),
            Err (ref markers) => {
                let mut len = 0;
                for m in markers {
                    match *m {
                        Ok (ref marker) => { len += self.data.marker_len (marker); }
                        Err ((_, amount)) => { len += amount; }
                    }
                }
                let mut v: Vec<u8> = Vec::with_capacity (len);
                for m in markers {
                    match *m {
                        Ok (ref marker) => { v.extend (self.data.chunk (marker).as_slice ()); }
                        Err ((byte, amount)) => {
                            for _ in 0 .. amount { v.push (byte); }
                        }
                    }
                }
                Chunk::from (v)
            }
        };

        let model: Option<(&Model, bool)> = {
            let empty = String::with_capacity (0);
            let tag: &String = if let Some (ref tag) = tag { tag } else { &empty };

            self.lookup_model (&tag, |m, e| {
                if !m.is_decodable () { return false }
                decoded = self.decode (m, e, chunk.as_slice ());
                decoded.is_ok ()
            })
        };

        let node = if decoded.is_ok () {
            Node::Scalar (decoded.unwrap ())
        } else {
            match model {
                Some ( (model, explicit) ) => Node::Scalar (try! (self.decode (model, explicit, chunk.as_slice ()))),
                None => {
                    let mut meta: Result<TaggedValue, ()> = Err ( () );

                    if let Some (m) = self.schema.get_metamodel () {
                        let empty = String::with_capacity (0);
                        let tag: &String = if let Some (ref tag) = tag { tag } else { &empty };

                        meta = m.meta_init (
                            anchor.clone (),
                            self.resolve_tag (&tag),
                            chunk.as_slice ()
                        );
                    }

                    match meta {
                        Err ( _ ) => {
                            self.out.send (
                                (self.idx, Clue::Response (
                                    Response::Error (
                                        block_id,
                                        Cow::from (format! (
                                            "Could not find appropriate model (tag {})",
                                            match tag {
                                                Some (t) => t,
                                                None => String::from ("")
                                            }
                                        ))
                                    )
                                ))
                            ).unwrap ();
                            return Err ( () )
                        }

                        Ok (tagged_value) => Node::Scalar (tagged_value)
                    }
                }
            }
        };


        let response = Response::Node (block_id, anchor, node);

        self.out.send ((self.idx, Clue::Response (response))).unwrap ();

        Ok ( () )
    }


    fn lookup_model<T: AsRef<str>, F: FnMut (&Model, bool) -> bool> (&self, tag: &T, mut predicate: F) -> Option<(&Model, bool)> {
        let schema: &Schema = self.schema.as_ref ();

        let tag = tag.as_ref ();
        let tag = if tag.len () == 0 { "" } else { tag.as_ref () };

        if tag.starts_with ("!<") && tag.ends_with (">") {
            let tag: &str = &tag[2 .. tag.len () - 1];

            if tag.len () > 0 {
                if let Some (m) = schema.look_up_model (tag) {
                    return Some ( (m, true) )
                }
            }
        } else {
            let mut result: bool = false;
            let mut parts: Option<(&str, &str)> = None;

            for arc in self.tag_handles.iter ().rev () {
                let prefix_value: &(Cow<'static, str>, Cow<'static, str>) = arc;
                let prefix: &str = prefix_value.0.as_ref ();

                if tag.starts_with (prefix) {
                    let (_, suffix) = tag.split_at (prefix.len ());
                    let value: &str = prefix_value.1.as_ref ();

                    parts = Some ( (value, suffix) );

                    break;
                }
            }

            if let Some ( (start, end) ) = parts {
                if start.len () > 0 && end.len () > 0 && !start.contains(' ') {
                    if let Some (m) = schema.look_up_model_callback (&mut |m| {
                        let t = m.get_tag ();
                        let t: &str = t.as_ref ();
                        if t.len () == start.len () + end.len () && t.starts_with (start) && t.ends_with (end) && predicate (m, true) {
                            result = true;
                            true
                        } else { false }
                    }) { return Some ((m, result)) };

                } else if tag.len () > 0 && start.len () > 0 && end.len () == 0 {
                    for word in start.split_whitespace () {
                        if let Some (m) = schema.look_up_model_callback (&mut |m| {
                            let t = m.get_tag ();
                            let t: &str = t.as_ref ();
                            if t == word {
                                if predicate (m, true) {
                                    result = true;
                                    true
                                } else { false }
                            } else if (word.ends_with (":") || word.ends_with (",")) && t.starts_with (word) && predicate (m, false) {
                                result = false;
                                true
                            } else { false }
                        }) { return Some ((m, result)) }
                    }
                } else if tag.len () == 0 || (start.len () > 0 && (start.ends_with (":") || start.ends_with (","))) {
                    if let Some (m) = schema.look_up_model_callback (&mut |m| {
                        let t = m.get_tag ();
                        let t: &str = t.as_ref ();
                        if t.len () > start.len () && t.starts_with (start) && predicate (m, false) {
                            result = false;
                            true
                        } else { false }
                    }) { return Some ((m, result)) }
                }
            }
        };

        None
    }


    fn resolve_tag<T: AsRef<str>> (&self, tag: &T) -> Option<String> {
        let tag = tag.as_ref ();
        let tag = if tag.len () == 0 { "" } else { tag.as_ref () };

        if tag.starts_with ("!<") && tag.ends_with (">") {
            let tag: &str = &tag[2 .. tag.len () - 1];

            return Some ( format! ("{}", tag) );
        }

        let mut parts: Option<(&str, &str)> = None;

        for arc in self.tag_handles.iter ().rev () {
            let prefix_value: &(Cow<'static, str>, Cow<'static, str>) = arc;
            let prefix: &str = prefix_value.0.as_ref ();
            let value: &str = prefix_value.1.as_ref ();

            if tag.starts_with (prefix) && !value.contains (' ') {
                let (_, suffix) = tag.split_at (prefix.len ());
                parts = Some ( (value, suffix) );

                break;
            }
        }

        if let Some ( (p, s) ) = parts {
            Some (format! ("{}{}", p, s))
        } else { None }
    }


    fn set_tag_handle (tag_handles: &mut Vec<Arc<(Cow<'static, str>, Cow<'static, str>)>>, tag_handle: Arc<(Cow<'static, str>, Cow<'static, str>)>) {
        let mut fnd: bool = false;
        let mut idx: usize = 0;

        for i in 0 .. tag_handles.len () {
            let th: &(Cow<'static, str>, Cow<'static, str>) = &tag_handles[i];

            if th.0.as_ref () == *&tag_handle.0.as_ref () {
                fnd = true;
                idx = i;
                break;
            }
        }

        if fnd {
            tag_handles[idx] = tag_handle;
        } else {
            tag_handles.push (tag_handle);
        }
    }
}
