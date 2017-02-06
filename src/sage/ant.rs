extern crate skimmer;

use self::skimmer::reader::Chunk;

use model::{ Model, TaggedValue, Schema };
use model::yamlette::literal::{ self, Literal };
use reader::{ self, Id, Block, BlockType, NodeKind };
use sage::conveyor::Clue;
use sage::YamlVersion;

use txt::Twine;

use std::io;
use std::sync::Arc;
use std::sync::mpsc::{ sync_channel, SyncSender, Receiver };
use std::thread::{ Builder, JoinHandle };




pub enum Response {
    Error (Id, Twine),
    TagHandle (Id, Twine, Twine),
    Alias (Id, String),
    Node (Id, Option<String>, Node)
}




pub enum Node {
    MetaMap (Option<String>, Option<Id>),
    MetaSeq (Option<String>),

    Dictionary (Twine, Option<Id>),
    Sequence (Twine),

    Scalar (TaggedValue),
    Literal (String)
}



#[derive (Debug)]
pub enum Message {
    Request (Request),
    Signal (Signal)
}



#[derive (Debug)]
pub enum Request {
    ReadBlock (Block),
    ReadDirectiveTag ( Id, Chunk, Chunk ),
    ReadLiteralBlock ( Id, Option<Chunk>, Option<Chunk>, Vec<Chunk> )
}



#[derive (Debug)]
pub enum Signal {
    Reset,
    TagHandle (Arc<(Twine, Twine)>),
    Terminate,
    Version (YamlVersion)
}




pub struct Ant {
    idx: u8,
    cin: Receiver<Message>,
    out: SyncSender<(u8, Clue)>,

    schema: Arc<Box<Schema>>,
    tag_handles: Vec<Arc<(Twine, Twine)>>,

    yaml_version: YamlVersion
}



impl Ant {
    pub fn run (
        idx: u8,
        out: SyncSender<(u8, Clue)>,
        schema: Arc<Box<Schema>>,
        yaml_version: YamlVersion,
        tag_handles: Vec<Arc<(Twine, Twine)>>
    )
        -> io::Result<(SyncSender<Message>, JoinHandle<()>)>
    {
        let (to_me, cin) = sync_channel (1);

        let handle = try! (Builder::new ().name (format! ("sage_ant_{}", idx)).spawn (move || {
            ( Ant {
                idx: idx,
                cin: cin,
                out: out,

                schema: schema,
                tag_handles: tag_handles,

                yaml_version: yaml_version
            } ).execute ()
        }));

        Ok ( (to_me, handle) )
    }


    pub fn execute (mut self) -> () {
        let schema: &Schema = self.schema.as_ref ().as_ref ();

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
        request: Request,
        model_literal: &Literal
    ) -> Result<(), ()> {
        match request {
            Request::ReadDirectiveTag ( id, shorthand, prefix ) => self.read_directive_tag (id, shorthand, prefix, model_literal),

            Request::ReadBlock ( block ) => self.read_block ( block, model_literal ),

            Request::ReadLiteralBlock ( id, anchor, tag, vec ) => self.read_literal_block ( id, anchor, tag, vec, model_literal )
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
        shorthand: Chunk,
        prefix: Chunk,
        model_literal: &Literal
    ) -> Result<(), ()> {
        let shorthand: Result<String, ()> = model_literal.bytes_to_string (&shorthand);
        let prefix: Result<String, ()> = model_literal.bytes_to_string (&prefix);

        if shorthand.is_err () {
            self.out.send ((self.idx, Clue::Response (Response::Error ( id, Twine::from ("Cannot decode shorthand") )))).unwrap ();
        } else if prefix.is_err () {
            self.out.send ((self.idx, Clue::Response (Response::Error ( id, Twine::from ("Cannot decode prefix") )))).unwrap ();
        } else {
            self.out.send ((self.idx, Clue::Response (Response::TagHandle (id, Twine::from (shorthand.unwrap ()), Twine::from (prefix.unwrap ()))))).unwrap ();
        }

        Ok ( () )
    }


    fn read_literal_block (
        &self,
        id: Id,
        anchor: Option<Chunk>,
        tag: Option<Chunk>,
        vec: Vec<Chunk>,
        model_literal: &Literal
    ) -> Result<(), ()> {
        let mut total_len = 0;
        for chunk in &vec { total_len += chunk.len (); }
        let mut value = Vec::with_capacity (total_len);
        for chunk in vec { value.extend_from_slice (&chunk); }

        let block = Block::new (id, BlockType::Node (reader::Node { anchor: anchor, tag: tag, content: NodeKind::Scalar (Chunk::from (value)) }));

        self.read_block (block, model_literal)
    }


    fn read_block (
        &self,
        block: Block,
        model_literal: &Literal
    ) -> Result<(), ()> {
        match block.cargo {
            BlockType::Alias ( chunk ) => self.read_alias (block.id, chunk, model_literal),

            BlockType::BlockMap ( firstborn_id, anchor, tag ) => self.read_map ( block.id, Some (firstborn_id), anchor, tag, model_literal ),

            BlockType::Literal ( chunk ) => self.read_literal (block.id, chunk, model_literal),

            BlockType::Node ( node ) => match node.content {
                NodeKind::LiteralBlockOpen |
                NodeKind::LiteralBlockClose => unreachable! (),

                NodeKind::Null => self.read_null (block.id, node.anchor, node.tag, model_literal),

                NodeKind::Mapping => self.read_map ( block.id, None, node.anchor, node.tag, model_literal ),
                NodeKind::Sequence => self.read_seq ( block.id, node.anchor, node.tag, model_literal ),

                NodeKind::Scalar ( chunk ) => self.read_scalar (block.id, node.anchor, node.tag, model_literal, chunk)
            },

            BlockType::DirectiveTag ( (_, _) ) => unreachable! (),
            BlockType::DirectiveYaml ( (_, _) ) => unreachable! (),
            BlockType::DocStart => unreachable! (),
            BlockType::DocEnd => unreachable! (),
            BlockType::Error (_, _) => unreachable! (),
            BlockType::Warning (_, _) => unreachable! (),
            BlockType::StreamEnd => unreachable! ()
        }
    }


    fn read_alias (&self, id: Id, chunk: Chunk, model_literal: &Literal) -> Result<(), ()> {
        let alias = model_literal.bytes_to_string (&chunk);

        match alias {
            Ok (string) => self.out.send ((self.idx, Clue::Response (Response::Alias (id, string)))),
            Err ( () ) => self.out.send ((self.idx, Clue::Response (Response::Error (id, Twine::from ("Cannot decode alias")))))
        }.unwrap ();

        Ok ( () )
    }


    fn read_literal (&self, id: Id, chunk: Chunk, model_literal: &Literal) -> Result<(), ()> {
        let literal = model_literal.bytes_to_string (&chunk);

        match literal {
            Ok (string) => self.out.send ((self.idx, Clue::Response (Response::Node (id, None, Node::Literal (string))))),
            Err ( () ) => self.out.send ((self.idx, Clue::Response (Response::Error (id, Twine::from ("Cannot decode alias")))))
        }.unwrap ();

        Ok ( () )
    }


    fn read_anchor (&self, block_id: Id, model_literal: &Literal, anchor: Option<Chunk>) -> Result<Option<String>, ()> {
        if let Some (anchor) = anchor {
            let result = model_literal.bytes_to_string (&anchor);

            match result {
                Ok (string) => Ok (Some (string)),
                Err ( () ) => {
                    self.out.send ((self.idx, Clue::Response (Response::Error (block_id, Twine::from ("Cannot decode anchor"))))).unwrap ();
                    return Err ( () );
                }
            }
        } else { Ok (None) }
    }


    fn read_tag (&self, block_id: Id, model_literal: &Literal, tag: Option<Chunk>) -> Result<Option<String>, ()> {
        let tag = if let Some (tag) = tag {
            let result = model_literal.bytes_to_string (&tag);

            match result {
                Err ( () ) => {
                    self.out.send ((self.idx, Clue::Response (Response::Error (block_id, Twine::from ("Cannot decode tag"))))).unwrap ();
                    return Err ( () )
                }
                Ok (tag) => Some (tag)
            }
        } else { None };

        Ok (tag)
    }


    fn read_model<F: FnMut (&Model, bool) -> bool> (&self, tag: Option<String>, block_id: Id, predicate: F) -> Result<&Model, ()> {
        let tag = if let Some (tag) = tag { tag } else { String::with_capacity (0) };

        let model: Option<(&Model, bool)> = self.lookup_model (&tag, predicate);

        match model {
            None => {
                self.out.send ((self.idx, Clue::Response (Response::Error (block_id, Twine::from (format! ("Could not find appropriate model (tag {})", tag)))))).unwrap ();
                Err ( () )
            }

            Some ( (model, _) ) => Ok (model)
        }
    }


    fn read_map (
        &self,
        block_id: Id,
        firstborn_id: Option<Id>,
        anchor: Option<Chunk>,
        tag: Option<Chunk>,
        model_literal: &Literal
    ) -> Result<(), ()> {
        let anchor: Option<String> = try! (self.read_anchor (block_id, model_literal, anchor));
        let tag: Option<String> = try! (self.read_tag (block_id, model_literal, tag));


        let model: Option<(&Model, bool)> = {
            let empty = String::with_capacity (0);
            let tag: &String = if let Some (ref tag) = tag { tag } else { &empty };

            self.lookup_model (&tag, |m, _| { m.is_dictionary () })
        };


        let node = match model {
            Some ( (model, _) ) => Node::Dictionary (model.get_tag ().clone (), firstborn_id),
            None => Node::MetaMap (tag, firstborn_id)
        };


        let response = Response::Node (block_id, anchor, node);

        self.out.send ((self.idx, Clue::Response (response))).unwrap ();

        Ok ( () )
    }


    fn read_seq (
        &self,
        block_id: Id,
        anchor: Option<Chunk>,
        tag: Option<Chunk>,
        model_literal: &Literal
    ) -> Result<(), ()> {
        let anchor: Option<String> = try! (self.read_anchor (block_id, model_literal, anchor));
        let tag: Option<String> = try! (self.read_tag (block_id, model_literal, tag));

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


    fn read_null (&self, block_id: Id, anchor: Option<Chunk>, tag: Option<Chunk>, model_literal: &Literal) -> Result<(), ()> {
        let anchor: Option<String> = try! (self.read_anchor (block_id, model_literal, anchor));
        let tag: Option<String> = try! (self.read_tag (block_id, model_literal, tag));

        let model = try! (self.read_model (tag, block_id, |m, _| { !m.is_collection () && m.has_default () }));

        let node = Node::Scalar (model.get_default ());
        let response = Response::Node (block_id, anchor, node);

        self.out.send ((self.idx, Clue::Response (response))).unwrap ();

        Ok ( () )
    }


    fn read_scalar (&self, block_id: Id, anchor: Option<Chunk>, tag: Option<Chunk>, model_literal: &Literal, chunk: Chunk) -> Result<(), ()> {
        let anchor: Option<String> = try! (self.read_anchor (block_id, model_literal, anchor));
        let tag: Option<String> = try! (self.read_tag (block_id, model_literal, tag));

        let mut decoded: Result<TaggedValue, ()> = Err ( () );

        let model: Option<(&Model, bool)> = {
            let empty = String::with_capacity (0);
            let tag: &String = if let Some (ref tag) = tag { tag } else { &empty };

            self.lookup_model (&tag, |m, e| {
                if !m.is_decodable () { return false }
                decoded = self.decode (m, e, &chunk);
                decoded.is_ok ()
            })
        };

        let node = if decoded.is_ok () {
            Node::Scalar (decoded.unwrap ())
        } else {
            match model {
                Some ( (model, explicit) ) => Node::Scalar (try! (self.decode (model, explicit, &chunk))),
                None => {
                    let mut meta: Result<TaggedValue, ()> = Err ( () );

                    if let Some (m) = self.schema.get_metamodel () {
                        let empty = String::with_capacity (0);
                        let tag: &String = if let Some (ref tag) = tag { tag } else { &empty };

                        meta = m.meta_init (
                            anchor.clone (),
                            self.resolve_tag (&tag),
                            &chunk
                        );
                    }

                    match meta {
                        Err ( _ ) => {
                            self.out.send (
                                (self.idx, Clue::Response (
                                    Response::Error (
                                        block_id,
                                        Twine::from (format! (
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
        let schema: &Schema = self.schema.as_ref ().as_ref ();

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
                let prefix_value: &(Twine, Twine) = arc;
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
                        let t: &str = m.get_tag ().as_ref ();
                        if t.len () == start.len () + end.len () && t.starts_with (start) && t.ends_with (end) && predicate (m, true) {
                            result = true;
                            true
                        } else { false }
                    }) { return Some ((m, result)) };

                } else if tag.len () > 0 && start.len () > 0 && end.len () == 0 {
                    for word in start.split_whitespace () {
                        if let Some (m) = schema.look_up_model_callback (&mut |m| {
                            let t: &str = m.get_tag ().as_ref ();
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
                        let t: &str = m.get_tag ().as_ref ();
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
            let prefix_value: &(Twine, Twine) = arc;
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


    fn set_tag_handle (tag_handles: &mut Vec<Arc<(Twine, Twine)>>, tag_handle: Arc<(Twine, Twine)>) {
        let mut fnd: bool = false;
        let mut idx: usize = 0;

        for i in 0 .. tag_handles.len () {
            let th: &(Twine, Twine) = &tag_handles[i];

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
