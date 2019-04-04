extern crate skimmer;

use model::renderer::{Node, Renderer};
use model::style::CommonStyles;
use model::{model_alias, model_tag, Model, Rope, Tagged, TaggedValue};

use std::any::Any;
use std::borrow::Cow;
use std::iter::Iterator;
// use std::marker::PhantomData;

pub static TAG: &'static str = "tag:yaml.org,2002:pairs";

#[derive(Clone, Copy)]
pub struct Pairs;

impl Pairs {
    pub fn get_tag() -> Cow<'static, str> {
        Cow::from(TAG)
    }
}

impl Model for Pairs {
    fn get_tag(&self) -> Cow<'static, str> {
        Self::get_tag()
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut Any {
        self
    }

    fn is_dictionary(&self) -> bool {
        true
    }

    fn compose(
        &self,
        renderer: &Renderer,
        value: TaggedValue,
        tags: &mut Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
        children: &mut [Rope],
    ) -> Rope {
        compose(self, renderer, value, tags, children)
    }
}

pub fn compose(
    model: &Model,
    renderer: &Renderer,
    value: TaggedValue,
    tags: &mut Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
    children: &mut [Rope],
) -> Rope {
    let value: PairsValue =
        match <TaggedValue as Into<Result<PairsValue, TaggedValue>>>::into(value) {
            Ok(value) => value,
            Err(_) => panic!("Not a PairsValue"),
        };

    if children.len() == 0 {
        return compose_empty(model, value, tags);
    }

    if value.styles.flow() {
        if value.styles.multiline() {
            compose_flow_multiline(model, value, tags, children)
        } else if value.styles.respect_threshold() {
            compose_flow_respect_threshold(model, renderer, value, tags, children)
        } else {
            compose_flow_no_threshold(model, value, tags, children)
        }
    } else {
        compose_block(model, value, tags, children)
    }
}

fn compose_empty(
    model: &Model,
    mut value: PairsValue,
    tags: &mut Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
) -> Rope {
    if let Some(alias) = value.take_alias() {
        if value.styles.issue_tag() {
            Rope::from(vec![
                model_tag(model, tags),
                Node::Space,
                model_alias(model, alias),
                Node::Space,
                Node::SquareBrackets,
            ])
        } else {
            Rope::from(vec![
                model_alias(model, alias),
                Node::Space,
                Node::SquareBrackets,
            ])
        }
    } else {
        if value.styles.issue_tag() {
            Rope::from(vec![
                model_tag(model, tags),
                Node::Space,
                Node::SquareBrackets,
            ])
        } else {
            Rope::from(Node::SquareBrackets)
        }
    }
}

fn compose_flow_multiline(
    model: &Model,
    mut value: PairsValue,
    tags: &mut Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
    children: &mut [Rope],
) -> Rope {
    let indent_len = value.styles.indent() as usize;
    let issue_tag = value.styles.issue_tag();
    let alias = value.take_alias();

    let mut rope_length = 3;

    if issue_tag {
        rope_length += 2;
    }
    if alias.is_some() {
        rope_length += 2;
    }

    for child in children.iter() {
        rope_length += child.len() + 1; // colon+space / comma+newline
    }

    let mut rope = Rope::with_capacity(rope_length);

    if issue_tag {
        rope.push(model_tag(model, tags));
        if let Some(alias) = alias {
            rope.push(Node::Space);
            rope.push(model_alias(model, alias));
        }
        rope.push(Node::Space);
    } else if let Some(alias) = alias {
        rope.push(model_alias(model, alias));
        rope.push(Node::Space);
    }

    rope.push(Node::SquareBracketOpen);
    rope.push(Node::NewlineIndent(indent_len));

    let last_child_idx = children.len() - 1;
    let penult_child_idx = children.len() - 2;

    let mut i = 0;
    loop {
        if i > last_child_idx {
            break;
        }

        {
            let key = unsafe { children.get_unchecked_mut(i) };

            key.indent(indent_len);

            rope.knit(key);
        }

        if i == last_child_idx {
            rope.push(Node::ColonNewline);
            break;
        } else {
            rope.push(Node::ColonSpace);
        }

        {
            let val = unsafe { children.get_unchecked_mut(i + 1) };

            rope.knit(val);
        }

        if i == penult_child_idx {
            rope.push(Node::NewlineIndent(0));
        } else {
            rope.push(Node::CommaNewlineIndent(indent_len));
        }

        i += 2;
    }

    rope.push(Node::SquareBracketClose);

    rope
}

fn compose_flow_respect_threshold(
    model: &Model,
    renderer: &Renderer,
    mut value: PairsValue,
    tags: &mut Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
    children: &mut [Rope],
) -> Rope {
    let indent_len = value.styles.indent() as usize;
    let compact = value.styles.compact();
    let threshold = value.styles.threshold() as usize;
    let issue_tag = value.styles.issue_tag();
    let alias = value.take_alias();

    let mut rope_length = 3;

    if issue_tag {
        rope_length += 2;
    }
    if alias.is_some() {
        rope_length += 2;
    }

    for child in children.iter() {
        rope_length += child.len() + 2;
    }

    let mut rope = Rope::with_capacity(rope_length);

    if issue_tag {
        rope.push(model_tag(model, tags));
        if let Some(alias) = alias {
            rope.push(Node::Space);
            rope.push(model_alias(model, alias));
        }
        rope.push(Node::Space);
    } else if let Some(alias) = alias {
        rope.push(model_alias(model, alias));
        rope.push(Node::Space);
    }

    rope.push(Node::SquareBracketOpen);
    if !compact {
        rope.push(Node::Space);
    }

    let last_child_idx = children.len() - 1;
    let penult_child_idx = children.len() - 2;

    let comma_len = renderer.node_len(&Node::Comma);
    let colon_len = renderer.node_len(&Node::Colon);
    let space_len = renderer.node_len(&Node::Space);

    let mut line_len = rope.bytes_len(renderer);

    let mut i = 0;
    loop {
        if i > last_child_idx {
            break;
        }

        {
            let key = unsafe { children.get_unchecked_mut(i) };

            key.indent(indent_len);

            let (key_first_line_len, nl) = key.first_line_bytes_len(renderer);

            if !compact {
                line_len += space_len;
            }
            line_len += key_first_line_len;

            if i != 0 {
                if line_len > threshold {
                    rope.push(Node::NewlineIndent(0));
                    line_len = if nl {
                        let (last_line_len, _) = key.last_line_bytes_len(renderer);
                        last_line_len
                    } else {
                        key_first_line_len
                    };
                } else {
                    if !compact {
                        rope.push(Node::Space);
                    }

                    if nl {
                        let (last_line_len, _) = key.last_line_bytes_len(renderer);
                        line_len = last_line_len;
                    }
                }
            }

            rope.knit(key);
        }

        if i == last_child_idx {
            rope.push(Node::Colon);
            break;
        }

        {
            let val = unsafe { children.get_unchecked_mut(i + 1) };

            val.indent(indent_len);

            let (first_line_len, nl) = val.first_line_bytes_len(renderer);

            line_len += colon_len + space_len + first_line_len + comma_len;

            if line_len > threshold {
                rope.push(Node::ColonNewline);
                if nl {
                    let (last_line_len, _) = val.last_line_bytes_len(renderer);
                    line_len = last_line_len;
                } else {
                    line_len = first_line_len;
                }
            } else {
                rope.push(Node::ColonSpace);

                if nl {
                    let (last_line_len, _) = val.last_line_bytes_len(renderer);
                    line_len = last_line_len;
                }
            }

            rope.knit(val);
        }

        if i != penult_child_idx {
            rope.push(Node::Comma);
        } else if !compact {
            rope.push(Node::Space);
        }

        i += 2;
    }

    rope.push(Node::SquareBracketClose);

    rope
}

fn compose_flow_no_threshold(
    model: &Model,
    mut value: PairsValue,
    tags: &mut Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
    children: &mut [Rope],
) -> Rope {
    let indent_len = value.styles.indent() as usize;
    let compact = value.styles.compact();
    let issue_tag = value.styles.issue_tag();
    let alias = value.take_alias();

    let mut rope_length = 3;

    if issue_tag {
        rope_length += 2;
    }
    if alias.is_some() {
        rope_length += 2;
    }

    for child in children.iter() {
        rope_length += child.len() + 1; // colon+space / comma+newline
    }

    let mut rope = Rope::with_capacity(rope_length);

    if issue_tag {
        rope.push(model_tag(model, tags));
        if let Some(alias) = alias {
            rope.push(Node::Space);
            rope.push(model_alias(model, alias));
        }
        rope.push(Node::Space);
    } else if let Some(alias) = alias {
        rope.push(model_alias(model, alias));
        rope.push(Node::Space);
    }

    rope.push(Node::SquareBracketOpen);
    if !compact {
        rope.push(Node::Space);
    }

    let last_child_idx = children.len() - 1;
    let penult_child_idx = children.len() - 2;

    let mut i = 0;
    loop {
        if i > last_child_idx {
            break;
        }

        {
            let key = unsafe { children.get_unchecked_mut(i) };

            key.indent(indent_len);

            rope.knit(key);
        }

        if i == last_child_idx {
            rope.push(Node::Colon);
            break;
        } else {
            rope.push(Node::ColonSpace);
        }

        {
            let val = unsafe { children.get_unchecked_mut(i + 1) };

            val.indent(indent_len);

            rope.knit(val);
        }

        if i != penult_child_idx {
            if compact {
                rope.push(Node::Comma);
            } else {
                rope.push(Node::CommaSpace);
            }
        } else if !compact {
            rope.push(Node::Space);
        }

        i += 2;
    }

    rope.push(Node::SquareBracketClose);

    rope
}

fn compose_block(
    model: &Model,
    mut value: PairsValue,
    tags: &mut Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
    children: &mut [Rope],
) -> Rope {
    let indent_len = value.styles.indent() as usize;
    let issue_tag = value.styles.issue_tag();
    let alias = value.take_alias();

    let mut rope_length = if issue_tag { 3 } else { 1 };
    for child in children.iter() {
        rope_length += child.len() + 3;
    }
    if alias.is_some() {
        rope_length += 2;
    }

    let mut rope = Rope::with_capacity(rope_length);

    if issue_tag {
        rope.push(model_tag(model, tags));
        if let Some(alias) = alias {
            rope.push(Node::Space);
            rope.push(model_alias(model, alias));
        }
        rope.push(Node::NewlineIndent(0));
    } else if let Some(alias) = alias {
        rope.push(model_alias(model, alias));
        rope.push(Node::NewlineIndent(0));
    }

    let last_child_idx = children.len() - 1;
    let penult_child_idx = if children.len() < 2 {
        0
    } else {
        children.len() - 2
    };

    let mut i = 0;

    loop {
        if i > last_child_idx {
            break;
        }

        if i == 0 {
            rope.push(Node::HyphenSpace);
        }

        {
            let key = unsafe { children.get_unchecked_mut(i) };

            let is_multiline = key.is_multiline();
            let is_flow = key.is_flow_opening();

            if is_multiline && !is_flow {
                rope.push(Node::QuestionNewlineIndent(indent_len));
                key.indent(indent_len);
            }

            rope.knit(key);
        }

        if i == last_child_idx {
            rope.push(Node::ColonNewline);
            break;
        }

        {
            let val = unsafe { children.get_unchecked_mut(i + 1) };

            let is_multiline = val.is_multiline();
            let is_flow = val.is_flow_opening();

            if is_multiline && !is_flow {
                rope.push(Node::ColonNewlineIndent(indent_len));
                val.indent(indent_len);
                rope.knit(val);

                rope.push(Node::IndentHyphenSpace(0));
            } else {
                rope.push(Node::ColonSpace);
                rope.knit(val);

                if i == penult_child_idx {
                    rope.push(Node::Newline);
                } else {
                    rope.push(Node::NewlineIndentHyphenSpace(0));
                }
            }
        }

        i += 2;
    }

    rope
}

#[derive(Debug)]
pub struct PairsValue {
    styles: CommonStyles,
    alias: Option<Cow<'static, str>>,
}

impl PairsValue {
    pub fn new(styles: CommonStyles, alias: Option<Cow<'static, str>>) -> PairsValue {
        PairsValue {
            styles: styles,
            alias: alias,
        }
    }

    pub fn take_alias(&mut self) -> Option<Cow<'static, str>> {
        self.alias.take()
    }
}

impl Tagged for PairsValue {
    fn get_tag(&self) -> Cow<'static, str> {
        Cow::from(TAG)
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_mut_any(&mut self) -> &mut Any {
        self as &mut Any
    }
}

#[cfg(all(test, not(feature = "dev")))]
mod tests {
    use super::*;

    // use txt::get_charset_utf8;

    #[test]
    fn tag() {
        let pairs = Pairs; // ::new (&get_charset_utf8 ());

        assert_eq!(pairs.get_tag(), TAG);
    }
}
