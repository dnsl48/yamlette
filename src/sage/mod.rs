extern crate skimmer;

mod ant;
pub mod conveyor;


use self::conveyor::{ Conveyor, Clue };

use self::skimmer::data::Datum;

use model::TaggedValue;
use model::schema::Schema;
use reader::{ Block, Id };
use txt::Twine;

use std::io;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::mpsc::{ Receiver, SyncSender };
use std::thread::{ self, JoinHandle };



pub struct Sage<S, D> {
    conv: (JoinHandle<Result<(), SageError>>, SyncSender<Clue>, Receiver<Idea>),
    _sage: PhantomData<S>,
    _datum: PhantomData<D>
}



impl<S, D> Sage<S, D>
  where
    S: Schema + 'static,
    D: Datum + Sync + Send + 'static
{
    pub fn new (pipe: Receiver<Block<D>>, schema: S) -> io::Result<Sage<S, D>> {
        let conv = try! (Conveyor::run (pipe, schema));
        Ok (Sage {
            conv: conv,
            _sage: PhantomData,
            _datum: PhantomData
        })
    }


    pub fn set_yaml_version (&self, version: YamlVersion) -> Result<(), SageError> {
        self.conv.1.send (Clue::Version (version)).or_else (|_| {
            Err ( SageError::Error (Twine::from ("Sage has passed away")) )
        })
    }


    pub fn terminate (&self) -> Result<(), SageError> {
        self.conv.1.send (Clue::Terminate).or_else (|_| {
            Err ( SageError::Error (Twine::from ("Sage has passed away")) )
        })
    }


    pub fn join (self) -> thread::Result<Result<(), SageError>> {
        self.conv.0.join ()
    }
}



impl<S, D> Deref for Sage<S, D>
  where
    S: Schema + 'static,
    D: Datum + Sync + Send + 'static
{
    type Target = Receiver<Idea>;

    fn deref (&self) -> &Receiver<Idea> { &self.conv.2 }
}




pub enum SageError {
    Error (Twine),
    IoError (io::Error)
}




#[derive(Debug)]
pub enum Idea {
    Done,

    Dawn,
    Dusk,

    Alias (Id, String),
    Error (Id, Twine),

    NodeMetaMap (Id, Option<String>, Option<String>, Option<Id>),
    NodeMetaSeq (Id, Option<String>, Option<String>),

    NodeDictionary (Id, Option<String>, Twine, Option<Id>),
    NodeSequence (Id, Option<String>, Twine),
    NodeScalar (Id, Option<String>, TaggedValue),
    NodeLiteral (Id, Option<String>, String),

    ReadError (Id, usize, Twine),
    ReadWarning (Id, usize, Twine)
}




#[derive (Eq, PartialEq, Clone, Copy, Hash, Debug)]
pub enum YamlVersion {
    V1x1,
    V1x2
}
