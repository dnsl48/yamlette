extern crate skimmer;

// use self::skimmer::symbol::CopySymbol;


use std::mem;


use model::renderer::{ Renderer, Node };



#[derive (Debug)]
pub enum Rope {
    Empty,
    Node ([Node; 1]),
    Many (Vec<Node>)
}



impl Rope {
    pub fn with_capacity (size: usize) -> Rope { Rope::Many (Vec::with_capacity (size)) }

    pub fn clear (&mut self) {
        *self = match *self {
            Rope::Empty => Rope::Empty,
            Rope::Node (_) => Rope::Empty,
            Rope::Many (ref mut vec) => {
                let mut v = mem::replace (vec, Vec::new ());
                v.clear ();
                Rope::Many (v)
            }
        };
    }

    pub fn len (&self) -> usize {
        match *self {
            Rope::Empty => 0,
            Rope::Node (_) => 1,
            Rope::Many (ref nodes) => nodes.len ()
        }
    }


    pub fn is_multiline (&self) -> bool {
        match *self {
            Rope::Empty => (),
            Rope::Node (_) => (),
            Rope::Many (ref nodes) => {
                let len = nodes.len ();
                if len <= 1 { return false; }

                let mut passed_nls = false;

                for node in nodes.iter ().rev () {
                    if node.is_newline () {
                        if passed_nls { return true }
                    } else if !passed_nls {
                        passed_nls = true;
                    }
                }
            }
        };

        false
    }


    pub fn is_flow_opening (&self) -> bool {
        match *self {
            Rope::Empty => false,
            Rope::Node (ref nodes) => nodes[0].is_flow_opening (),
            Rope::Many (ref nodes) => nodes.len () > 0 && nodes[0].is_flow_opening ()
        }
    }


    pub fn is_flow_dict_opening (&self) -> bool {
        match *self {
            Rope::Empty => false,
            Rope::Node (ref nodes) => nodes[0].is_flow_dict_opening (),
            Rope::Many (ref nodes) => nodes.len () > 0 && nodes[0].is_flow_dict_opening ()
        }
    }


    pub fn last_line_bytes_len (&self, renderer: &Renderer) -> (usize, bool) {
        match *self {
            Rope::Empty => (0, false),
            Rope::Node (ref nodes) => self._line_bytes_len (renderer, nodes.iter ()),
            Rope::Many (ref nodes) => self._line_bytes_len (renderer, nodes.iter ().rev ())
        }
    }


    pub fn first_line_bytes_len (&self, renderer: &Renderer) -> (usize, bool) {
        match *self {
            Rope::Empty => (0, false),
            Rope::Node (ref nodes) => self._line_bytes_len (renderer, nodes.iter ()),
            Rope::Many (ref nodes) => self._line_bytes_len (renderer, nodes.iter ())
        }
    }

    fn _line_bytes_len<'a, 'b, 'c, Iter: Iterator<Item=&'a Node>> (&'b self, renderer: &'c Renderer, nodes: Iter) -> (usize, bool) {
        let mut len = 0;
        let mut nl = false;

        for node in nodes {
            match *node {
                Node::StringNewline (ref s) => { len += s.len (); nl = true; break; }
                Node::Newline => { nl = true; break; }
                Node::NewlineIndent (_) => { nl = true; break; }
                Node::NewlineIndentHyphenSpace (_) => { nl = true; break; }
                Node::NewlineIndentQuestionSpace (_) => { nl = true; break; }
                Node::CommaNewlineIndent (_) => { len += renderer.node_len (&Node::Comma); nl = true; break; }
                Node::ColonNewlineIndent (_) |
                Node::ColonNewline => { len += renderer.node_len (&Node::Colon); nl = true; break; }
                Node::QuestionNewlineIndent (_) |
                Node::QuestionNewline => { len += renderer.node_len (&Node::Question); nl = true; break; }
                Node::TripleHyphenNewline => { len += renderer.node_len (&Node::Hyphen) * 3; nl = true; break; }
                Node::TripleDotNewline => { len += renderer.node_len (&Node::Dot) * 3; nl = true; break; }

                ref node @ _ => len += renderer.node_len (node)
            }
        }

        (len, nl)
    }


    pub fn bytes_len (&self, renderer: &Renderer) -> usize {
        match *self {
            Rope::Empty => 0,
            Rope::Node (ref node) => renderer.node_len (&node[0]),
            Rope::Many (ref nodes) => {
                let mut size = 0;
                for node in nodes { size += renderer.node_len (node); }
                size
            }
        }
    }


    pub fn render (self, renderer: &Renderer) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::with_capacity (self.bytes_len (renderer));

        match self {
            Rope::Empty => (),
            Rope::Node (mut node) => renderer.render_into_vec (&mut vec, mem::replace (&mut node[0], Node::Empty)),
            Rope::Many (nodes) => for node in nodes { renderer.render_into_vec (&mut vec, node); }
        }

        vec
    }


    pub fn push (&mut self, node: Node) {
        let is_many = match *self {
            Rope::Many (_) => true,
            _ => false
        };

        if !is_many {
            *self = match *self {
                Rope::Empty => Rope::Many (Vec::with_capacity (1)),
                Rope::Node (ref mut node)  => {
                    let nd = mem::replace (&mut node[0], Node::Empty);
                    let mut v = Vec::with_capacity (2);
                    v.push (nd);
                    Rope::Many (v)
                }
                Rope::Many (_) => unreachable! ()
            };
        }

        match *self {
            Rope::Many (ref mut vec) => vec.push (node),
            _ => unreachable! ()
        };
    }


    pub fn indent (&mut self, len: usize) {
        match *self {
            Rope::Empty => (),
            Rope::Node (ref mut node) => node[0].indent (len),
            Rope::Many (ref mut nodes) => for node in nodes { node.indent (len); }
        }
    }


    pub fn knit (&mut self, rope: &mut Rope) {
        let is_empty = match *self {
            Rope::Empty => true,
            _ => false
        };

        if is_empty {
            let ro = mem::replace (rope, Rope::Empty);
            *self = ro;
            return;
        }

        let is_node = match *self {
            Rope::Node (_) => true,
            _ => false
        };

        if is_node {
            *self = match *self {
                Rope::Node (ref mut node) => {
                    let node = mem::replace (&mut node[0], Node::Empty);
                    let mut vec = Vec::with_capacity (1 + rope.len ());
                    vec.push (node);
                    Rope::Many (vec)
                }
                _ => unreachable! ()
            };
        }

        match *self {
            Rope::Many (ref mut vec) => {
                match *rope {
                    Rope::Empty => (),
                    Rope::Node (ref mut node) => {
                        let node = mem::replace (&mut node[0], Node::Empty);
                        vec.push (node);
                    }
                    Rope::Many (ref mut other) => vec.append (other)
                };

                mem::replace (rope, Rope::Empty);
            }
            _ => unreachable! ()
        }
    }


    pub fn unrope<'a, 'b, 'c> (&'a self, ptr: &'b mut &'a [Node], renderer: &'c Renderer, index: usize, threshold: usize) -> (usize, usize, bool) {
        match *self {
            Rope::Empty => {
                *ptr = &[];
                (0, 0, true)
            }
            Rope::Node (ref node) => {
                if index == 0 {
                    *ptr = node;
                    (renderer.node_len (&node[0]), 0, true)
                } else {
                    *ptr = &[];
                    (0, 0, true)
                }
            }
            Rope::Many (ref nodes) => {
                if index >= nodes.len () {
                    *ptr = &[];
                    (0, 0, true)
                } else {
                    let len = nodes.len ();
                    let first = index;
                    let mut last = index;
                    let mut tot_len: usize = 0;

                    loop {
                        let node = &nodes[last];
                        let nlen = renderer.node_len (node) as usize;

                        tot_len += nlen;

                        if tot_len >= threshold {
                            if first == last { } else {
                                last -= 1;
                                tot_len -= nlen;
                            }
                            break;
                        }

                        if last == len-1 { break; }

                        last += 1;
                    }

                    last += 1;

                    *ptr = &nodes[first .. last];
                    (tot_len, last, last == len)
                }
            }
        }
    }
}



impl From<Node> for Rope {
    fn from (node: Node) -> Rope { Rope::Node ([node]) }
}



impl From<Vec<Node>> for Rope {
    fn from (nodes: Vec<Node>) -> Rope { Rope::Many (nodes) }
}
