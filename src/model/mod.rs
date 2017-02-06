pub mod yaml;
pub mod yamlette;
pub mod schema;

pub mod renderer;
pub mod rope;
pub mod style;
pub mod tagged_value;



extern crate skimmer;

use txt::{ CharSet, Twine };
use txt::encoding::{ Encoding, Unicode };

use std::any::Any;

use std::iter::Iterator;


pub use self::renderer::{ EncodedString, Renderer, Node };
pub use self::rope::Rope;
pub use self::schema::Schema;
pub use self::style::CommonStyles;
pub use self::tagged_value::TaggedValue;


// TODO: decode/encode Errors&Warnings


pub trait Factory : Send {
    fn get_tag (&self) -> &Twine;

    fn build_model (&self, &CharSet) -> Box<Model>;
}




pub trait Tagged : Any {
    fn get_tag (&self) -> &Twine;

    fn as_any (&self) -> &Any;

    fn as_mut_any (&mut self) -> &mut Any;
}




pub trait Model : Send + Sync {
    fn get_tag (&self) -> &Twine;

    fn as_any (&self) -> &Any;

    fn as_mut_any (&mut self) -> &mut Any;


    fn get_encoding (&self) -> Encoding;


    fn is_collection (&self) -> bool { self.is_dictionary () || self.is_sequence () }

    fn is_sequence (&self) -> bool { false }

    fn is_dictionary (&self) -> bool { false }


    fn is_metamodel (&self) -> bool { false }

    fn is_decodable (&self) -> bool { false }

    fn is_encodable (&self) -> bool { false }


    fn has_default (&self) -> bool { false }

    fn get_default (&self) -> TaggedValue { panic! ("Model does not have a default value"); }


    fn meta_init (&self, _anchor: Option<String>, _tag: Option<String>, _value: &[u8]) -> Result<TaggedValue, ()> { panic! ("Model is not a metamodel"); }


    fn decode (&self, _explicit: bool, _value: &[u8]) -> Result<TaggedValue, ()> { panic! ("Model is not decodable"); }

    fn decode11 (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> { self.decode (explicit, value) }


    fn encode (&self, _renderer: &Renderer, _value: TaggedValue, _tags: &mut Iterator<Item=&(Twine, Twine)>) -> Result<Rope, TaggedValue> { panic! ("Model is not encodable"); }

    fn compose (&self, _renderer: &Renderer, _value: TaggedValue, _tags: &mut Iterator<Item=&(Twine, Twine)>, _children: &mut [Rope]) -> Rope { panic! ("Model is not composable"); }
}




pub fn model_issue_rope (model: &Model, node: Node, issue_tag: bool, alias: Option<Twine>, tags: &mut Iterator<Item=&(Twine, Twine)>) -> Rope {
    if let Some (alias) = alias {
        if issue_tag {
            Rope::from (vec! [model_tag (model, tags), Node::Space, model_alias (model, alias), Node::Space, node])
        } else {
            Rope::from (vec! [model_alias (model, alias), Node::Space, node])
        }
    } else {
        if issue_tag {
            Rope::from (vec! [model_tag (model, tags), Node::Space, node])
        } else {
            Rope::from (node)
        }
    }
}



pub fn model_alias (model: &Model, alias: Twine) -> Node {
    match alias {
        Twine::Static (alias) => _model_alias_static_str (alias, model.get_encoding ()),
        Twine::String (alias) => _model_alias_string (alias, model.get_encoding ())
    }
}


fn _model_alias_static_str (alias: &'static str, encoding: Encoding) -> Node {
    match encoding.str_to_bytes (alias) {
        Ok (s) => Node::AmpersandString (EncodedString::from (s)),
        Err (s) => Node::AmpersandString (EncodedString::from (s))
    }
}

fn _model_alias_string (alias: String, encoding: Encoding) -> Node {
    Node::AmpersandString (EncodedString::from (encoding.string_to_bytes (alias)))
}




pub fn model_tag (model: &Model, tags: &mut Iterator<Item=&(Twine, Twine)>) -> Node {
    match *model.get_tag () {
        Twine::Static (tag) => _model_tag_static_str (tag, model.get_encoding (), tags),
        Twine::String (ref tag) => _model_tag_string (tag, model.get_encoding (), tags)
    }
}



fn _model_tag_static_str (tag: &'static str, encoding: Encoding, tags: &mut Iterator<Item=&(Twine, Twine)>) -> Node {
    for &(ref shortcut, ref handle) in tags {
        let h = handle.as_ref ();
        if h.len () == 0 || h.contains (' ') { continue; }

        if tag.starts_with (h) {
            return match *shortcut {
                Twine::Static (s) => {
                    match encoding.str_to_bytes (s) {
                        Ok (f) => match encoding.str_to_bytes (&tag[h.len () ..]) {
                            Ok (l) => Node::StringConcat (EncodedString::from (f), EncodedString::from (l)),
                            Err (l) => {
                                let mut string = Vec::with_capacity (f.len () + l.len ());
                                string.extend (f);
                                string.extend (l);
                                Node::String (EncodedString::from (string))
                            }
                        },
                        Err (f) => match encoding.str_to_bytes (&tag[h.len () ..]) {
                            Ok (l) => {
                                let mut string = Vec::with_capacity (f.len () + l.len ());
                                string.extend (f);
                                string.extend (l);
                                Node::String (EncodedString::from (string))
                            }
                            Err (l) => {
                                let mut string = Vec::with_capacity (f.len () + l.len ());
                                string.extend (f);
                                string.extend (l);
                                Node::String (EncodedString::from (string))
                            }
                        }
                    }
                }
                Twine::String (ref s) => {
                    let mut string = Vec::with_capacity (s.len () + (tag.len () - h.len ()));

                    match encoding.str_to_bytes (s.as_ref ()) {
                        Ok (f) => string.extend (f),
                        Err (v) => string.extend (v)
                    }

                    match encoding.str_to_bytes (&tag[h.len () ..]) {
                        Ok (f) => string.extend (f),
                        Err (v) => string.extend (v)
                    }

                    Node::String (EncodedString::from (string))
                }
            }
        }
    }

    match encoding.str_to_bytes (tag) {
        Ok (s) => Node::StringSpecificTag (EncodedString::from (s)),
        Err (v) => Node::StringSpecificTag (EncodedString::from (v))
    }
}



fn _model_tag_string (tag: &str, encoding: Encoding, tags: &mut Iterator<Item=&(Twine, Twine)>) -> Node {
    for &(ref shortcut, ref handle) in tags {
        let h = handle.as_ref ();
        if h.len () == 0 || h.contains (' ') { continue; }

        if tag.starts_with (h) {
            return match *shortcut {
                Twine::Static (s) => {
                    match encoding.str_to_bytes (s) {
                        Ok (f) => match encoding.str_to_bytes (&tag[h.len () ..]) {
                            Ok (l) => {
                                let mut string = Vec::with_capacity (f.len () + l.len ());
                                string.extend (f);
                                string.extend (l);
                                 Node::String (EncodedString::from (string))
                            },
                            Err (l) => {
                                let mut string = Vec::with_capacity (f.len () + l.len ());
                                string.extend (f);
                                string.extend (l);
                                Node::String (EncodedString::from (string))
                            }
                        },
                        Err (f) => match encoding.str_to_bytes (&tag[h.len () ..]) {
                            Ok (l) => {
                                let mut string = Vec::with_capacity (f.len () + l.len ());
                                string.extend (f);
                                string.extend (l);
                                Node::String (EncodedString::from (string))
                            }
                            Err (l) => {
                                let mut string = Vec::with_capacity (f.len () + l.len ());
                                string.extend (f);
                                string.extend (l);
                                Node::String (EncodedString::from (string))
                            }
                        }
                    }
                }
                Twine::String (ref s) => {
                    let mut string = Vec::with_capacity (s.len () + (tag.len () - h.len ()));

                    match encoding.str_to_bytes (s.as_ref ()) {
                        Ok (f) => string.extend (f),
                        Err (v) => string.extend (v)
                    }

                    match encoding.str_to_bytes (&tag[h.len () ..]) {
                        Ok (f) => string.extend (f),
                        Err (v) => string.extend (v)
                    }

                    Node::String (EncodedString::from (string))
                }
            }
        }
    }

    match encoding.str_to_bytes (tag) {
        Ok (s) => Node::StringSpecificTag (EncodedString::from (Vec::from (s))),
        Err (v) => Node::StringSpecificTag (EncodedString::from (v))
    }
}
