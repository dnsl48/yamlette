use crate::io::Input;
use super::context::{ReadContext, ReadContextKind};

#[derive(Debug)]
pub struct Skimmer<'a, 'b> {
    context: ReadContext<'a, 'b>,
    input: Input<'b>,
}

impl<'a, 'b> Skimmer<'a, 'b>
where
    'b: 'a
{
    pub fn new(input: Input<'b>) -> Self {
        Self { input, context: ReadContext::new(input.clone()) }
    }
}
