extern crate skimmer;

// use self::skimmer::symbol::{ CopySymbol, Combo };

use txt::Twine;

use model::{ model_alias, model_tag, Model, Rope, Renderer, Tagged, TaggedValue };
use model::renderer::Node;
use model::style::CommonStyles;

use std::any::Any;
use std::iter::Iterator;
// use std::marker::PhantomData;




pub const TAG: &'static str = "tag:yaml.org,2002:seq";
pub static TWINE_TAG: Twine = Twine::Static (TAG);



pub struct Seq;



impl Seq {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }
/*
    pub fn new (cset: &CharSet<Char, DoubleChar>) -> Seq<Char, DoubleChar> { Seq {
        encoding: cset.encoding,
        _char: PhantomData,
        _dchr: PhantomData
    } }
*/
}



impl Model for Seq {
    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

    // fn get_encoding (&self) -> Encoding { self.encoding }

    fn is_sequence (&self) -> bool { true }

    fn compose (&self, renderer: &Renderer, value: TaggedValue, tags: &mut Iterator<Item=&(Twine, Twine)>, children: &mut [Rope]) -> Rope {
        compose (self, renderer, value, tags, children)
    }
}



pub fn compose (model: &Model, renderer: &Renderer, value: TaggedValue, tags: &mut Iterator<Item=&(Twine, Twine)>, children: &mut [Rope]) -> Rope {
    let value: SeqValue = match <TaggedValue as Into<Result<SeqValue, TaggedValue>>>::into (value) {
        Ok (value) => value,
        Err (_) => panic! ("Not a SeqValue")
    };

    if children.len () == 0 {
        return compose_empty (model, value, tags);
    }

    if value.styles.flow () {
        if value.styles.multiline () {
            compose_flow_multiline (model, value, tags, children)
        } else if value.styles.respect_threshold () {
            compose_flow_respect_threshold (model, renderer, value, tags, children)
        } else {
            compose_flow_no_threshold (model, value, tags, children)
        }
    } else {
        compose_block (model, value, tags, children)
    }
}



fn compose_empty (model: &Model, mut value: SeqValue, tags: &mut Iterator<Item=&(Twine, Twine)>) -> Rope {
    if let Some (alias) = value.take_alias () {
        if value.styles.issue_tag () {
            Rope::from (vec! [model_tag (model, tags), Node::Space, model_alias (model, alias), Node::Space, Node::SquareBrackets])
        } else {
            Rope::from (vec! [model_alias (model, alias), Node::Space, Node::SquareBrackets])
        }
    } else {
        if value.styles.issue_tag () {
            Rope::from (vec! [model_tag (model, tags), Node::Space, Node::SquareBrackets])
        } else {
            Rope::from (Node::SquareBrackets)
        }
    }
}



fn compose_flow_respect_threshold (model: &Model, renderer: &Renderer, mut value: SeqValue, tags: &mut Iterator<Item=&(Twine, Twine)>, children: &mut [Rope]) -> Rope {
    let indent_len = value.styles.indent () as usize;
    let compact = value.styles.compact ();
    let threshold = value.styles.threshold () as usize;
    let issue_tag = value.styles.issue_tag ();
    let alias = value.take_alias ();

    let mut rope_length = if compact { 2 } else { 3 };  // brackets

    if issue_tag { rope_length += 2; }
    if alias.is_some () { rope_length += 2; }

    for child in children.iter () {
        rope_length += child.len () + 2; // comma, space/newline
    }


    let mut rope = Rope::with_capacity (rope_length);

    if issue_tag {
        rope.push (model_tag (model, tags));
        if let Some (alias) = alias {
            rope.push (Node::Space);
            rope.push (model_alias (model, alias));
        }
        rope.push (Node::Space);
    } else if let Some (alias) = alias {
        rope.push (model_alias (model, alias));
        rope.push (Node::Space);
    }

    rope.push (Node::SquareBracketOpen);
    if !compact { rope.push (Node::Space); }

    let children_last_idx = children.len () - 1;
    let mut line_len = rope.bytes_len (renderer);
    let comma_len = renderer.node_len (&Node::Comma);
    let space_len = renderer.node_len (&Node::Space);

    for (idx, child) in children.iter_mut ().enumerate () {
        child.indent (indent_len);

        let (child_first_line_len, _) = child.first_line_bytes_len (renderer);
        line_len += if !compact { comma_len + space_len } else { comma_len };
        line_len += child_first_line_len;

        if idx != 0 {
            if line_len > threshold {
                rope.push (Node::CommaNewlineIndent (0));
                let (last_line_len, _) = child.last_line_bytes_len (renderer);
                line_len = last_line_len;
            } else {
                if compact {
                    rope.push (Node::Comma);
                } else {
                    rope.push (Node::CommaSpace);
                }
            }
        }

        rope.knit (child);

        if idx == children_last_idx && !compact {
            rope.push (Node::Space);
        }
    }

    rope.push (Node::SquareBracketClose);

    rope
}



fn compose_flow_no_threshold (model: &Model, mut value: SeqValue, tags: &mut Iterator<Item=&(Twine, Twine)>, children: &mut [Rope]) -> Rope {
    let indent_len = value.styles.indent () as usize;
    let compact = value.styles.compact ();
    let issue_tag = value.styles.issue_tag ();
    let alias = value.take_alias ();

    let mut rope_length = if compact { 2 } else { 3 };

    if issue_tag { rope_length += 2; }
    if alias.is_some () { rope_length += 2; }

    for child in children.iter () {
        rope_length += child.len () + 1;  // comma/comma+space
    }

    let mut rope = Rope::with_capacity (rope_length);

    if issue_tag {
        rope.push (model_tag (model, tags));
        if let Some (alias) = alias {
            rope.push (Node::Space);
            rope.push (model_alias (model, alias));
        }
        rope.push (Node::Space);
    } else if let Some (alias) = alias {
        rope.push (model_alias (model, alias));
        rope.push (Node::Space);
    }

    rope.push (Node::SquareBracketOpen);
    if !compact { rope.push (Node::Space); }

    let children_last_idx = children.len () - 1;
    for (idx, child) in children.iter_mut ().enumerate () {
        child.indent (indent_len);

        rope.knit (child);

        if idx != children_last_idx {
            if compact {
                rope.push (Node::Comma);
            } else {
                rope.push (Node::CommaSpace);
            }
        } else if !compact {
            rope.push (Node::Space);
        }
    }

    rope.push (Node::SquareBracketClose);

    rope
}



fn compose_flow_multiline (model: &Model, mut value: SeqValue, tags: &mut Iterator<Item=&(Twine, Twine)>, children: &mut [Rope]) -> Rope {
    let indent_len = value.styles.indent () as usize;
    let issue_tag = value.styles.issue_tag ();
    let alias = value.take_alias ();

    let mut rope_length = 3;

    if issue_tag { rope_length += 2; }
    if alias.is_some () { rope_length += 2; }

    for child in children.iter () {
        rope_length += child.len () + 1;  // comma/comma+newline
    }

    let mut rope = Rope::with_capacity (rope_length);

    if issue_tag {
        rope.push (model_tag (model, tags));
        if let Some (alias) = alias {
            rope.push (Node::Space);
            rope.push (model_alias (model, alias));
        }
        rope.push (Node::Space);
    } else if let Some (alias) = alias {
        rope.push (model_alias (model, alias));
        rope.push (Node::Space);
    }

    rope.push (Node::SquareBracketOpen);
    rope.push (Node::NewlineIndent (indent_len));

    let last_child_idx = children.len () - 1;
    for (idx, child) in children.iter_mut ().enumerate () {
        child.indent (indent_len);

        rope.knit (child);

        if idx != last_child_idx {
            rope.push (Node::CommaNewlineIndent (indent_len));
        } else {
            rope.push (Node::NewlineIndent (0));
        }
    }

    rope.push (Node::SquareBracketClose);

    rope
}



fn compose_block (model: &Model, mut value: SeqValue, tags: &mut Iterator<Item=&(Twine, Twine)>, children: &mut [Rope]) -> Rope {
    let indent_len = value.styles.indent () as usize;
    let issue_tag = value.styles.issue_tag ();
    let alias = value.take_alias ();

    let mut rope_length = 1;

    if issue_tag { rope_length += 2; }
    if alias.is_some () { rope_length += 2; }

    for child in children.iter () {
        rope_length += child.len () + 1;
    }

    let mut rope = Rope::with_capacity (rope_length);

    if issue_tag {
        rope.push (model_tag (model, tags));
        if let Some (alias) = alias {
            rope.push (Node::Space);
            rope.push (model_alias (model, alias));
        }
        rope.push (Node::NewlineIndent (0));
    } else if let Some (alias) = alias {
        rope.push (model_alias (model, alias));
        rope.push (Node::NewlineIndent (0));
    }

    let last_child_idx = children.len () - 1;
    for (idx, child) in children.iter_mut ().enumerate () {
        child.indent (indent_len);

        if idx == 0 { rope.push (Node::HyphenSpace); }

        let is_multiline = child.is_multiline ();

        rope.knit (child);

        if idx != last_child_idx {
            if is_multiline {
                rope.push (Node::IndentHyphenSpace (0));
            } else {
                rope.push (Node::NewlineIndentHyphenSpace (0));
            }
        } else if !is_multiline { rope.push (Node::Newline); }
    }

    rope
}




#[derive (Debug)]
pub struct SeqValue {
    styles: CommonStyles,
    alias: Option<Twine>
}



impl SeqValue {
    pub fn new (styles: CommonStyles, alias: Option<Twine>) -> SeqValue { SeqValue { styles: styles, alias: alias } }

    pub fn take_alias (&mut self) -> Option<Twine> { self.alias.take () }
}



impl Tagged for SeqValue {
    fn get_tag (&self) -> &Twine { &TWINE_TAG }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    // use txt::get_charset_utf8;


    #[test]
    fn tag () {
        let seq = Seq; // ::new (&get_charset_utf8 ());

        assert_eq! (seq.get_tag (), TAG);
    }
}
