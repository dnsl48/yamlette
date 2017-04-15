mod conductor;
mod performer;

pub mod chord;


use self::conductor::{ Conductor, Hint, Message };

use model::{ Renderer, CommonStyles, TaggedValue, Schema };

use std::borrow::Cow;
use std::io;
use std::sync::mpsc::{ sync_channel, Receiver, SyncSender };
use std::thread::JoinHandle;



pub type Music = Vec<u8>;



pub struct Orchestra {
    styles: CommonStyles,
    pipe: SyncSender<Message>,
    cond: (JoinHandle<Result<(), OrchError>>, Receiver<Music>)
}



impl Orchestra {
    pub fn new<S> (schema: S) -> io::Result<Orchestra>
      where
        S: Schema + Clone + 'static
    {
        let (sender, receiver) = sync_channel (32);

        let styles = schema.get_common_styles ();

        let renderer = Renderer; // ::new (&cset);
        let schema = schema;

        let cond = try! (Conductor::run (receiver, renderer, schema));

        Ok (Orchestra {
            styles: styles,
            pipe: sender,
            cond: cond
        })
    }


    pub fn get_styles (&self) -> CommonStyles { self.styles }


    pub fn play (&self, level: usize, value: TaggedValue) -> Result<(), OrchError> {
        self.pipe.send (Message::Value (level, value)).or_else (|_| {
            Err ( OrchError::Error ("Conductor has quit already".to_string ()) )
        })
    }


    pub fn volume_border_top (&self, print: bool) -> Result<(), OrchError> {
        self.pipe.send (Message::Hint (Hint::BorderTop (print))).or_else (|_| {
            Err ( OrchError::Error ("Conductor has quit already".to_string ()) )
        })
    }


    pub fn volume_border_bot (&self, print: bool) -> Result<(), OrchError> {
        self.pipe.send (Message::Hint (Hint::BorderBot (print))).or_else (|_| {
            Err ( OrchError::Error ("Conductor has quit already".to_string ()) )
        })
    }


    pub fn directive_yaml (&self, print: bool) -> Result<(), OrchError> {
        self.pipe.send (Message::Hint (Hint::DirectiveYaml (print))).or_else (|_| {
            Err ( OrchError::Error ("Conductor has quit already".to_string ()) )
        })
    }


    pub fn directive_tags (&self, tags: Vec<(Cow<'static, str>, Cow<'static, str>)>) -> Result<(), OrchError> {
        self.pipe.send (Message::Hint (Hint::DirectiveTags (tags))).or_else (|_| {
            Err ( OrchError::Error ("Conductor has quit already".to_string ()) )
        })
    }


    pub fn volumes (&self, size: usize) -> Result<(), OrchError> {
        self.pipe.send (Message::Hint (Hint::Volumes (size))).or_else (|_| {
            Err ( OrchError::Error ("Conductor has quit already".to_string ()) )
        })
    }


    pub fn vol_next (&self) -> Result<(), OrchError> {
        self.pipe.send (Message::Hint (Hint::VolumeNext)).or_else (|_| {
            Err ( OrchError::Error ("Conductor has quit already".to_string ()) )
        })
    }


    pub fn vol_end (&self) -> Result<(), OrchError> {
        self.pipe.send (Message::Hint (Hint::VolumeEnd)).or_else (|_| {
            Err ( OrchError::Error ("Conductor has quit already".to_string ()) )
        })
    }


    pub fn vol_reserve (&self, size: usize) -> Result<(), OrchError> {
        self.pipe.send (Message::Hint (Hint::VolumeSize (size))).or_else (|_| {
            Err ( OrchError::Error ("Conductor has quit already".to_string ()) )
        })
    }


    pub fn the_end (&self) -> Result<(), OrchError> {
        self.pipe.send (Message::Hint (Hint::TheEnd)).or_else (|_| {
            Err ( OrchError::Error ("Conductor has quit already".to_string ()) )
        })
    }


    pub fn listen (&self) -> Result<Music, OrchError> {
        match self.cond.1.recv () {
            Ok (music) => Ok (music),
            Err (_) => Err (OrchError::Error (String::from ("orchestra vanished")))
        }
    }
}



pub enum OrchError {
    Error (String),
    IoError (io::Error)
}



impl From<io::Error> for OrchError {
    fn from (err: io::Error) -> OrchError { OrchError::IoError (err) }
}
