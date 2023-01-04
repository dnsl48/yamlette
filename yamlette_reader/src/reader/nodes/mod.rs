mod alias;

pub use alias::Alias;

#[derive(Debug)]
pub enum Node<'a> {
    Alias(Alias<'a>),
}
