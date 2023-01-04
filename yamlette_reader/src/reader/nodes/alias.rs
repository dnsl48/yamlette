use crate::io::Input;

#[derive(Clone, Debug)]
pub struct Alias<'a> {
    input: Input<'a>,
    alias: &'a str
}

impl<'a> Alias<'a> {
    pub fn new(input: Input<'a>) -> Self {
        Self {
            input,
            alias: &input.fragment()[1..]
        }
    }
}