use std::error::Error;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::Not;

#[inline(always)]
fn is<T: BitAnd<Output = T> + Eq + Copy>(state: T, val: T) -> bool {
    val == state & val
}

#[inline(always)]
fn not<T: BitAnd<Output = T> + Eq + Copy>(state: T, val: T) -> bool {
    !is(state, val)
}

#[inline(always)]
fn on<T: BitOr<Output = T> + Copy>(state: &mut T, val: T) {
    *state = *state | val;
}

#[inline(always)]
fn off<T: BitAnd<Output = T> + Not<Output = T> + Eq + Copy>(state: &mut T, val: T) {
    *state = *state & !val
}

#[derive(Debug)]
pub struct ReadError {
    pub position: usize,
    pub description: Cow<'static, str>,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl Error for ReadError {
    fn description(&self) -> &str {
        self.description.as_ref()
    }
}

impl ReadError {
    pub fn new<T>(description: T) -> ReadError
    where
        T: Into<Cow<'static, str>>,
    {
        ReadError {
            description: description.into(),
            position: 0,
        }
    }

    pub fn set_position(mut self, position: usize) -> ReadError {
        self.position = position;
        self
    }
}

#[derive(Clone, Debug, Hash)]
pub struct Id {
    pub level: usize,
    pub parent: usize,
    pub index: usize,
}

#[derive(Debug)]
pub struct Block<D> {
    pub id: Id,
    pub cargo: BlockType<D>,
}

impl<D> Block<D>
where
    D: Datum,
{
    pub fn new(id: Id, cargo: BlockType<D>) -> Block<D> {
        let block = Block { id, cargo };

        block
    }
}

#[derive(Debug)]
pub enum BlockType<D> {
    Alias(Marker),

    DirectiveTag((Marker, Marker)),
    DirectiveYaml((u8, u8)),

    DocStart,
    DocEnd,

    BlockMap(Id, Option<Marker>, Option<Marker>),
    Literal(Marker),
    Byte(u8, usize),

    Node(Node),

    Error(Cow<'static, str>, usize),
    Warning(Cow<'static, str>, usize),

    StreamEnd,
    Datum(D),
}

#[derive (Debug)]
pub struct Node {
    pub anchor: Option<Marker>,
    pub tag: Option<Marker>,
    pub content: NodeKind
}
