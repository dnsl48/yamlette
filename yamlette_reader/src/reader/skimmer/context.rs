use crate::io::Input;

#[derive(Copy, Clone, Debug)]
pub enum ReadContextKind {
    Base,
    Layer,
    Node,
    MappingBlock,
    MappingFlow,
    ScalarBlock,
    SequenceBlock,
    SequenceFlow,
}

impl Default for ReadContextKind {
    fn default() -> Self {
        ReadContextKind::Base
    }
}

#[derive(Clone, Debug)]
pub struct ReadContext<'a, 'b> {
    parent: Option<&'a ReadContext<'a, 'b>>,
    kind: ReadContextKind,

    input: Input<'b>,

    indent: usize,
    layer: usize,

    // level: usize,
    // parent: usize,
    index: usize,
}

impl<'a, 'b> ReadContext<'a, 'b>
where
    'b: 'a
{
    pub fn new(input: Input<'b>) -> Self {
        Self {
            parent: None,
            kind: ReadContextKind::default(),
            input,

            indent: 0,
            layer: 0,

            // level: 0,
            // parent: 0,
            index: 0,
        }
    }

    // fn _new(parent: &'a ReadContext, kind: ReadContextKind, indent: usize, level: usize) -> Self {
    //     Self {
    //         parent: Some(parent),
    //         kind,

    //         layer: match parent.kind {
    //             ReadContextKind::Default => 0,
    //             _ => parent.layer + 1,
    //         },
    //         level,
    //         indent,
    //     }
    // }

    // pub fn get_parent(&mut self) -> Option<&'a Self> {
    //     self.parent
    // }

    // pub fn get_parent_kind(&self) -> Option<ReadContextKind> {
    //     self.parent.as_ref().map(|p| p.kind)
    // }
}
