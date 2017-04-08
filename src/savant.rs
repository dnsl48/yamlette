extern crate skimmer;

use model::schema::Schema;
use model::{ Model, TaggedValue };

use txt::Twine;

use sage::{ Idea, YamlVersion, SageError };

use self::skimmer::{ Chunk, Data, Datum, Marker };
// use self::skimmer::symbol::{ Combo, CopySymbol, Symbol };

use reader::{ Block, BlockType, Node, NodeKind };

use std::marker::PhantomData;



pub struct Savant<S, D> {
    yaml_version: YamlVersion,
    data: Data<D>,
    schema: S,
    tag_handles: Vec<(Twine, Twine)>,
    buf_literal_block: Option<(usize, Vec<Result<Marker, (u8, usize)>>)>,
    _datum: PhantomData<D>
}



impl<S, D> Savant<S, D>
  where
    S: Schema + 'static,
    D: Datum + 'static
{
    pub fn new (schema: S) -> Savant<S, D> {
        let mut tag_handles: Vec<(Twine, Twine)>;

        {
            let th = schema.get_tag_handles ();
            tag_handles = Vec::with_capacity (th.len ());
            for th in th.iter ().rev () { tag_handles.push ((th.0.clone (), th.1.clone ())); }
        }

        Savant {
            yaml_version: YamlVersion::V1x2,
            data: Data::with_capacity (32),
            schema: schema,
            tag_handles: tag_handles,
            buf_literal_block: None,
            _datum: PhantomData
        }
    }


    pub fn think (&mut self, block: Block<D>) -> Result<Option<Idea>, SageError> {
        match block.cargo {
            BlockType::StreamEnd => {
                self.data.clear ();
                Ok (Some (Idea::Done))
            }

            BlockType::Datum (datum) => {
                self.data.push (datum);
                Ok (None)
            }

            BlockType::DirectiveYaml (version) => self.set_version (version),

            BlockType::DirectiveTag ( (tag, handle) ) => {
                let s = Twine::from (self.read_literal (tag) ?);
                let h = Twine::from (self.read_literal (handle) ?);
                self.reg_tag_handle (s, h)
            }

            BlockType::DocStart => Ok (Some (Idea::Dawn)),

            BlockType::DocEnd => Ok (Some (Idea::Dusk)),

            BlockType::Error (message, position) => Ok (Some (Idea::ReadError (block.id, position, message))),

            BlockType::Warning (message, position) => Ok (Some (Idea::ReadWarning (block.id, position, message))),

            BlockType::Node ( Node { anchor: _, tag: _, content: NodeKind::LiteralBlockOpen } ) => {
                self.buf_literal_block = Some ((block.id.index, Vec::with_capacity (32)));
                Ok (None)
            }

            BlockType::Literal ( .. ) if self.buf_literal_block.is_some () => {
                if let BlockType::Literal (chunk) = block.cargo {
                    if let Some ( (idx, ref mut vec) ) = self.buf_literal_block {
                        if idx != block.id.parent { panic! ("Unexpected literal!") }
                        vec.push (Ok (chunk));
                    }
                };

                Ok (None)
            }

            BlockType::Byte ( .. ) if self.buf_literal_block.is_some () => {
                if let BlockType::Byte (byte, amount) = block.cargo {
                    if let Some ( (idx, ref mut vec) ) = self.buf_literal_block {
                        if idx != block.id.parent { panic! ("Unexpected literal!") }
                        vec.push (Err ((byte, amount)));
                    }
                };

                Ok (None)
            }

            BlockType::Node ( Node { anchor, tag, content: NodeKind::LiteralBlockClose } ) => {
                let (idx, vec) = self.buf_literal_block.take ().unwrap ();
                if idx != block.id.index { panic! ("Unexpected literal block!") }
                let (anchor, value) = self.read_literal_block (anchor, tag, Err (vec)) ?;
                Ok (Some (Idea::NodeScalar (block.id, anchor, value)))
            }

            BlockType::Alias ( .. ) |
            BlockType::BlockMap ( .. ) |
            BlockType::Literal ( .. ) |
            BlockType::Byte ( .. ) |
            BlockType::Node ( .. ) => Ok (Some (self.read_block (block) ?))
        }
    }


    fn set_version (&mut self, version: (u8, u8)) -> Result<Option<Idea>, SageError> {
        if version.0 != 1 { return Err (SageError::Error (Twine::from (format! ("Unsupported yaml version {}.{}", version.0, version.1)))) }
        let ver = if version.1 == 1 { YamlVersion::V1x1 } else { YamlVersion::V1x2 };
        self.yaml_version = ver;
        Ok (None)
    }


    fn reg_tag_handle (&mut self, shorthand: Twine, prefix: Twine) -> Result<Option<Idea>, SageError> {
        let mut fnd: bool = false;
        let mut idx: usize = 0;

        for i in 0 .. self.tag_handles.len () {
            let ref th = self.tag_handles[i];

            if th.0 == shorthand.as_ref () {
                fnd = true;
                idx = i;
                break;
            }
        }

        if fnd {
            self.tag_handles[idx] = (shorthand, prefix);
        } else {
            self.tag_handles.push ((shorthand, prefix));
        }

        Ok (None)
    }


    fn read_block (&self, block: Block<D>) -> Result<Idea, SageError> {
        match block.cargo {
            BlockType::Alias ( marker ) => Ok (Idea::Alias (block.id, self.read_alias (marker) ?)),

            BlockType::BlockMap ( firstborn_id, anchor, tag ) => {
                match self.read_map (anchor, tag) ? {
                    Ok ((tag, anchor)) => Ok (Idea::NodeDictionary (block.id, anchor, tag, Some (firstborn_id))),
                    Err ((tag, anchor)) => Ok (Idea::NodeMetaMap (block.id, anchor, tag, Some (firstborn_id)))
                }
            }

            BlockType::Literal ( marker ) => Ok (Idea::NodeLiteral (block.id, None, self.read_literal (marker) ?)),

            BlockType::Byte ( byte, amount ) => Ok (Idea::NodeLiteral (block.id, None, self.read_byte (byte, amount) ?)),

            BlockType::Node ( node ) => match node.content {
                NodeKind::LiteralBlockOpen |
                NodeKind::LiteralBlockClose => unreachable! (),

                NodeKind::Null => {
                    let (anchor, value) = self.read_null (node.anchor, node.tag) ?;
                    Ok (Idea::NodeScalar (block.id, anchor, value))
                }

                NodeKind::Mapping => {
                    match self.read_map (node.anchor, node.tag) ? {
                        Ok ((tag, anchor)) => Ok (Idea::NodeDictionary (block.id, anchor, tag, None)),
                        Err ((tag, anchor)) => Ok (Idea::NodeMetaMap (block.id, anchor, tag, None))
                    }
                }

                NodeKind::Sequence => {
                    match self.read_seq (node.anchor, node.tag) ? {
                        Ok ((tag, anchor)) => Ok (Idea::NodeSequence (block.id, anchor, tag)),
                        Err ((tag, anchor)) => Ok (Idea::NodeMetaSeq (block.id, anchor, tag))
                    }
                }

                NodeKind::Scalar ( marker ) => {
                    let (anchor, value) = self.read_scalar (node.anchor, node.tag, Ok (marker)) ?;
                    Ok (Idea::NodeScalar (block.id, anchor, value))
                }
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



    fn read_literal_block (
        &mut self,
        anchor: Option<Marker>,
        tag: Option<Marker>,
        vec: Result<Marker, Vec<Result<Marker, (u8, usize)>>>
    ) -> Result<(Option<String>, TaggedValue), SageError>
    { self.read_scalar (anchor, tag, vec) }


    fn read_scalar (
        &self,
        anchor: Option<Marker>,
        tag: Option<Marker>,
        marker: Result<Marker, Vec<Result<Marker, (u8, usize)>>>
    ) -> Result<(Option<String>, TaggedValue), SageError> {
        let anchor: Option<String> = self.read_anchor (anchor) ?;
        let tag: Option<String> = self.read_tag (tag) ?;

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
        let chunk = chunk.as_slice ();

        let model: Option<(&Model, bool)> = {
            if let Some (ref tag) = tag {
                self.lookup_model (tag, |m, e| {
                    if !m.is_decodable () { return false }
                    decoded = self.decode (m, e, chunk);
                    decoded.is_ok ()
                })
            } else {
                if let Some (v) = match self.yaml_version {
                    YamlVersion::V1x1 => self.schema.try_decodable_models_11 (chunk),
                    YamlVersion::V1x2 => self.schema.try_decodable_models (chunk)
                } { decoded = Ok (v) };

                None
            }
        };

        let node = if decoded.is_ok () {
            decoded.unwrap ()
        } else {
            match model {
                Some ( (model, explicit) ) => {
                    match self.decode (model, explicit, chunk) {
                        Ok ( v ) => v,
                        Err ( () ) => return Err ( SageError::Error (Twine::from ("Could not decode value")) )
                    }
                }
                None => {
                    let mut meta: Result<TaggedValue, ()> = Err ( () );

                    if let Some (m) = self.schema.get_metamodel () {
                        let empty = String::with_capacity (0);
                        let tag: &String = if let Some (ref tag) = tag { tag } else { &empty };

                        meta = m.meta_init (
                            anchor.clone (),
                            self.resolve_tag (&tag),
                            chunk
                        );
                    }

                    match meta {
                        Err ( _ ) => {
                            return Err ( SageError::Error (Twine::from (format! (
                                "Could not find appropriate model (tag {})",
                                match tag {
                                    Some (t) => t,
                                    None => String::from ("")
                                }
                            ))) );
                        }

                        Ok (tagged_value) => tagged_value
                    }
                }
            }
        };

        Ok ( (anchor, node) )
    }


    fn read_map (&self, anchor: Option<Marker>, tag: Option<Marker>) -> Result<Result<(Twine, Option<String>), (Option<String>, Option<String>)>, SageError> {
        let anchor: Option<String> = self.read_anchor (anchor) ?;
        let tag: Option<String> = self.read_tag (tag) ?;

        let (ftag, found): (Twine, bool) = if let Some (ref tag) = tag {
            // let empty = String::with_capacity (0);
            // let tag: &String = if let Some (ref tag) = tag { tag } else { &empty };

            if let Some ( (m, f) ) = self.lookup_model (tag, |m, _| { m.is_dictionary () }) {
                (m.get_tag ().clone (), f)
            } else {
                (Twine::empty (), false)
            }
        } else {
            (self.schema.get_tag_model_map ().clone (), true)
        };

        if found {
            Ok (Ok ((ftag, anchor)))
        } else {
            Ok (Err ((tag, anchor)))
        }
    }


    fn read_seq (&self, anchor: Option<Marker>, tag: Option<Marker>) -> Result<Result<(Twine, Option<String>), (Option<String>, Option<String>)>, SageError> {
        let anchor: Option<String> = self.read_anchor (anchor) ?;
        let tag: Option<String> = self.read_tag (tag) ?;

        let (ftag, found): (Twine, bool) = if let Some (ref tag) = tag {
            // let empty = String::with_capacity (0);
            // let tag: &String = if let Some (ref tag) = tag { tag } else { &empty };

            if let Some ( (m, f) ) = self.lookup_model (tag, |m, _| { m.is_sequence () }) {
                (m.get_tag ().clone (), f)
            } else {
                (Twine::empty (), false)
            }
        } else {
            (self.schema.get_tag_model_seq ().clone (), true)
        };

        if found {
            Ok (Ok ((ftag, anchor)))
        } else {
            Ok (Err ((tag, anchor)))
        }
    }



    fn read_null (&self, anchor: Option<Marker>, tag: Option<Marker>) -> Result<(Option<String>, TaggedValue), SageError> {
        let anchor: Option<String> = self.read_anchor (anchor) ?;
        let tag: Option<String> = self.read_tag (tag) ?;

        let tagged_value = if let Some (tag) = tag {
            let model = self.read_model (tag, |m, _| { !m.is_collection () && m.has_default () }) ?;
            model.get_default ()
        } else {
            self.schema.get_model_null ().get_default ()
        };

        Ok ((anchor, tagged_value))
    }



    fn read_anchor (&self, anchor: Option<Marker>) -> Result<Option<String>, SageError> {
        if let Some (anchor) = anchor {
            let chunk = self.data.chunk (&anchor);
            let result = self.schema.get_model_literal ().bytes_to_string (chunk.as_slice ());

            match result {
                Ok (string) => Ok (Some (string)),
                Err ( () ) => Err ( SageError::Error (Twine::from ("Cannot decode anchor")) )
            }
        } else { Ok (None) }
    }


    fn read_tag (&self, tag: Option<Marker>) -> Result<Option<String>, SageError> {
        if let Some (tag) = tag {
            let chunk = self.data.chunk (&tag);
            let result = self.schema.get_model_literal ().bytes_to_string (chunk.as_slice ());

            match result {
                Ok (tag) => Ok (Some (tag)),
                Err ( () ) => Err ( SageError::Error (Twine::from ("Cannot decode tag")) )
            }
        } else { Ok (None) }
    }


    fn read_alias (&self, marker: Marker) -> Result<String, SageError> {
        let chunk = self.data.chunk (&marker);
        let alias = self.schema.get_model_literal ().bytes_to_string (chunk.as_slice ());

        match alias {
            Ok (alias) => Ok (alias),
            Err ( () ) => Err ( SageError::Error (Twine::from ("Cannot decode alias")) )
        }
    }


    fn read_literal (&self, marker: Marker) -> Result<String, SageError> {
        let chunk = self.data.chunk (&marker);
        let literal = self.schema.get_model_literal ().bytes_to_string (chunk.as_slice ());

        match literal {
            Ok (literal) => Ok (literal),
            Err ( () ) => Err ( SageError::Error (Twine::from ("Cannot decode literal")) )
        }
    }


    fn read_byte (&self, byte: u8, amount: usize) -> Result<String, SageError> {
        let literal = self.schema.get_model_literal ().bytes_to_string_times (&[byte], amount);

        match literal {
            Ok (literal) => Ok (literal),
            Err ( () ) => Err ( SageError::Error (Twine::from ("Cannot decode literal")) )
        }
    }


    fn read_model<F: FnMut (&Model, bool) -> bool> (&self, tag: String, predicate: F) -> Result<&Model, SageError> {
        let model: Option<(&Model, bool)> = self.lookup_model (&tag, predicate);

        match model {
            Some ( (model, _) ) => Ok (model),

            None => Err (SageError::Error (Twine::from (format! ("Could not find appropriate model (tag {})", tag))))
        }
    }


    fn lookup_model<T: AsRef<str>, F: FnMut (&Model, bool) -> bool> (&self, tag: &T, mut predicate: F) -> Option<(&Model, bool)> {
        let tag = tag.as_ref ();

        if tag.starts_with ("!<") && tag.ends_with (">") {
            let tag: &str = &tag[2 .. tag.len () - 1];

            if tag.len () > 0 {
                if let Some (m) = self.schema.look_up_model (tag) { return Some ( (m, true) ) }
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
                    if let Some (m) = self.schema.look_up_model_callback (&mut |m| {
                        let t: &str = m.get_tag ().as_ref ();
                        if t.len () == start.len () + end.len () && t.starts_with (start) && t.ends_with (end) && predicate (m, true) {
                            result = true;
                            true
                        } else { false }
                    }) { return Some ((m, result)) };

                } else if tag.len () > 0 && start.len () > 0 && end.len () == 0 {
                    for word in start.split_whitespace () {
                        if let Some (m) = self.schema.look_up_model_callback (&mut |m| {
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
                    if let Some (m) = self.schema.look_up_model_callback (&mut |m| {
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

            return Some ( String::from (tag) );
        }

        let mut parts: Option<(&str, &str)> = None;

        for prefix_value in self.tag_handles.iter ().rev () {
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


    fn decode (&self, model: &Model, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        match self.yaml_version {
            YamlVersion::V1x1 => model.decode11 (explicit, value),
            YamlVersion::V1x2 => model.decode   (explicit, value)
        }
    }
}
