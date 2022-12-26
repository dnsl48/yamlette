extern crate skimmer;

mod ant;
pub mod conveyor;

use self::conveyor::{Clue, Conveyor};

use self::skimmer::data::Datum;

use crate::model::schema::Schema;
use crate::model::TaggedValue;
use crate::reader::{Block, Id};

use std::borrow::Cow;
use std::io;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread::{self, JoinHandle};

pub struct Sage<S, D> {
    conv: (
        JoinHandle<Result<(), SageError>>,
        SyncSender<Clue>,
        Receiver<Idea>,
    ),
    _sage: PhantomData<S>,
    _datum: PhantomData<D>,
}

impl<S, D> Sage<S, D>
where
    S: Schema + 'static,
    D: Datum + Sync + Send + 'static,
{
    pub fn new(pipe: Receiver<Block<D>>, schema: S) -> io::Result<Sage<S, D>> {
        let conv = Conveyor::run(pipe, schema)?;
        Ok(Sage {
            conv,
            _sage: PhantomData,
            _datum: PhantomData,
        })
    }

    pub fn set_yaml_version(&self, version: YamlVersion) -> Result<(), SageError> {
        self.conv
            .1
            .send(Clue::Version(version))
            .or_else(|_| Err(SageError::Error(Cow::from("Sage has passed away"))))
    }

    pub fn terminate(&self) -> Result<(), SageError> {
        self.conv
            .1
            .send(Clue::Terminate)
            .or_else(|_| Err(SageError::Error(Cow::from("Sage has passed away"))))
    }

    pub fn join(self) -> thread::Result<Result<(), SageError>> {
        self.conv.0.join()
    }
}

impl<S, D> Deref for Sage<S, D>
where
    S: Schema + 'static,
    D: Datum + Sync + Send + 'static,
{
    type Target = Receiver<Idea>;

    fn deref(&self) -> &Receiver<Idea> {
        &self.conv.2
    }
}

pub enum SageError {
    Error(Cow<'static, str>),
    IoError(io::Error),
}

#[derive(Debug)]
pub enum Idea {
    Done,

    Dawn,
    Dusk,

    Alias(Id, String),
    Error(Id, Cow<'static, str>),

    NodeMetaMap(Id, Option<String>, Option<String>, Option<Id>),
    NodeMetaSeq(Id, Option<String>, Option<String>),

    NodeDictionary(Id, Option<String>, Cow<'static, str>, Option<Id>),
    NodeSequence(Id, Option<String>, Cow<'static, str>),
    NodeScalar(Id, Option<String>, TaggedValue),
    NodeLiteral(Id, Option<String>, String),

    ReadError(Id, usize, Cow<'static, str>),
    ReadWarning(Id, usize, Cow<'static, str>),
}

#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug)]
pub enum YamlVersion {
    V1x1,
    V1x2,
}
