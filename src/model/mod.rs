pub mod yaml;
pub mod yamlette;
pub mod schema;


pub mod renderer;
pub mod rope;
pub mod style;
pub mod tagged_value;



extern crate skimmer;

// use self::skimmer::symbol::{ CopySymbol, Combo };

// use txt::encoding::{ Encoding, Unicode };

use std::any::Any;
use std::borrow::Cow;
use std::iter::Iterator;


pub use self::renderer::{ EncodedString, Renderer, Node };
pub use self::rope::Rope;
pub use self::schema::Schema;
pub use self::style::CommonStyles;
pub use self::tagged_value::TaggedValue;


// TODO: decode/encode Errors&Warnings




pub trait Tagged : Any {
    fn get_tag (&self) -> Cow<'static, str>;

    fn as_any (&self) -> &Any;

    fn as_mut_any (&mut self) -> &mut Any;
}




pub trait Model : Send + Sync {
    // type Char: CopySymbol;
    // type DoubleChar: CopySymbol + Combo;

    fn get_tag (&self) -> Cow<'static, str>;

    fn as_any (&self) -> &Any;

    fn as_mut_any (&mut self) -> &mut Any;


    // fn get_encoding (&self) -> Encoding;


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


    fn encode (&self, _renderer: &Renderer, _value: TaggedValue, _tags: &mut Iterator<Item=&(Cow<'static, str>, Cow<'static, str>)>) -> Result<Rope, TaggedValue> { panic! ("Model is not encodable"); }

    fn compose (&self, _renderer: &Renderer, _value: TaggedValue, _tags: &mut Iterator<Item=&(Cow<'static, str>, Cow<'static, str>)>, _children: &mut [Rope]) -> Rope { panic! ("Model is not composable"); }
}




pub fn model_issue_rope (model: &Model, node: Node, issue_tag: bool, alias: Option<Cow<'static, str>>, tags: &mut Iterator<Item=&(Cow<'static, str>, Cow<'static, str>)>) -> Rope {
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



pub fn model_alias (_model: &Model, alias: Cow<'static, str>) -> Node {
    match alias {
        Cow::Borrowed (alias) => Node::AmpersandString (EncodedString::from (alias.as_bytes ())),
        Cow::Owned (alias) => Node::AmpersandString (EncodedString::from (alias.into_bytes ()))
    }
}



pub fn model_tag (model: &Model, tags: &mut Iterator<Item=&(Cow<'static, str>, Cow<'static, str>)>) -> Node {
    match model.get_tag () {
        Cow::Borrowed (tag) => _model_tag_static_str (tag, tags),
        Cow::Owned (ref tag) => _model_tag_string (tag, tags)
    }
}



fn _model_tag_static_str (tag: &'static str, tags: &mut Iterator<Item=&(Cow<'static, str>, Cow<'static, str>)>) -> Node {
    for &(ref shortcut, ref handle) in tags {
        let h = handle.as_ref ();
        if h.len () == 0 || h.contains (' ') { continue; }

        if tag.starts_with (h) {
            return match *shortcut {
                Cow::Borrowed (s) => {
                    let f = s.as_bytes ();
                    let l = tag[h.len () ..].as_bytes ();
                    Node::StringConcat (EncodedString::from (f), EncodedString::from (l))
                }
                Cow::Owned (ref s) => {
                    let mut string = Vec::with_capacity (s.len () + (tag.len () - h.len ()));

                    string.extend (s.as_bytes ());
                    string.extend (tag[h.len () ..].as_bytes ());

                    Node::String (EncodedString::from (string))
                }
            }
        }
    }

    Node::StringSpecificTag (EncodedString::from (tag.as_bytes ()))
}



fn _model_tag_string (tag: &str, tags: &mut Iterator<Item=&(Cow<'static, str>, Cow<'static, str>)>) -> Node {
    for &(ref shortcut, ref handle) in tags {
        let h = handle.as_ref ();
        if h.len () == 0 || h.contains (' ') { continue; }

        if tag.starts_with (h) {
            return match *shortcut {
                Cow::Borrowed (s) => {
                    let f = s.as_bytes ();
                    let l = tag[h.len () ..].as_bytes ();

                    let mut string = Vec::with_capacity (f.len () + l.len ());
                    string.extend (f);
                    string.extend (l);
                    Node::String (EncodedString::from (string))

                }
                Cow::Owned (ref s) => {
                    let mut string = Vec::with_capacity (s.len () + (tag.len () - h.len ()));

                    /*
                    match encoding.str_to_bytes (s.as_ref ()) {
                        Ok (f) => string.extend (f),
                        Err (v) => string.extend (v)
                    }

                    match encoding.str_to_bytes (&tag[h.len () ..]) {
                        Ok (f) => string.extend (f),
                        Err (v) => string.extend (v)
                    }
                    */

                    string.extend (s.as_bytes ());
                    string.extend (tag[h.len () ..].as_bytes ());

                    Node::String (EncodedString::from (string))
                }
            }
        }
    }

    let v: Vec<u8> = Vec::from (tag);
    Node::StringSpecificTag (EncodedString::from (v))

    /*
    match encoding.str_to_bytes (tag) {
        Ok (s) => Node::StringSpecificTag (EncodedString::from (Vec::from (s))),
        Err (v) => Node::StringSpecificTag (EncodedString::from (v))
    }
    */
}
