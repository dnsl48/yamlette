use super::Node;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EnvelopeIndex {
    pub level: usize,
    pub parent: usize,
    pub index: usize,
}

impl EnvelopeIndex {
    pub fn new(level: usize, parent: usize, index: usize) -> Self {
        Self { level, parent, index }
    }
}

#[derive(Debug)]
pub struct Envelope<'a> {
    pub index: EnvelopeIndex,
    pub node: Node<'a>,
}

