extern crate skimmer;

use self::skimmer::{Datum, Marker};

use crate::model::Schema;
use crate::reader::{Block, BlockType, Node, NodeKind};
use crate::sage::ant::{self, Ant, Message, Request, Response, Signal};
use crate::sage::{Idea, SageError, YamlVersion};

use std::borrow::Cow;
use std::io;
use std::sync::mpsc::{
    channel, sync_channel, Receiver, Sender, SyncSender, TryRecvError, TrySendError,
};
use std::sync::Arc;
use std::thread::{Builder, JoinHandle};

pub enum Clue {
    Version(YamlVersion),
    Terminate,

    Response(Response),
}

pub struct Conveyor<D> {
    pipe: Receiver<Block<D>>,
    cin: Receiver<(u8, Clue)>,
    ex_cin: Receiver<Clue>,
    out: Sender<Idea>,

    // data: Data,
    ants: [(SyncSender<Message<D>>, JoinHandle<()>); 3],
    msgs: usize,

    buff: Option<Clue>,

    /* Defaults */
    yaml_version: YamlVersion,

    tag_handles: Vec<Arc<(Cow<'static, str>, Cow<'static, str>)>>, // _schema: PhantomData<S>
}

macro_rules! _conveyor_signal {
    ( $slf:ident, $signal:expr ) => {{
        for idx in 0..$slf.ants.len() {
            $slf.ants[idx]
                .0
                .send(Message::Signal($signal))
                .or_else(|_| {
                    $slf.terminate();
                    Err(SageError::Error(Cow::from("One of ants has passed away")))
                })
                .and_then(|_| Ok(()))?;
        }

        let result: Result<(), SageError> = Ok(());

        result
    }};
}

impl<D> Conveyor<D>
where
    D: Datum + Sync + Send + 'static,
{
    pub fn run<S: Schema + 'static>(
        pipe: Receiver<Block<D>>,
        schema: S,
    ) -> io::Result<(
        JoinHandle<Result<(), SageError>>,
        SyncSender<Clue>,
        Receiver<Idea>,
    )> {
        let (ex_to_me, ex_cin) = sync_channel(2);
        let (idea_sdr, idea_rvr) = channel();

        let handle = Builder::new()
            .name("sage_conveyor".to_string())
            .spawn(move || {
                let (to_me, cin) = sync_channel(3);

                let mut atag_handles: Vec<Arc<(Cow<'static, str>, Cow<'static, str>)>>;

                {
                    let tag_handles = schema.get_tag_handles();
                    atag_handles = Vec::with_capacity(tag_handles.len());
                    for th in tag_handles.iter().rev() {
                        atag_handles.push(Arc::new((th.0.clone(), th.1.clone())));
                    }
                }

                let schema = Arc::new(schema);

                Ant::run(
                    0,
                    to_me.clone(),
                    schema.clone(),
                    YamlVersion::V1x2,
                    atag_handles.clone(),
                )
                .or_else(|err| Err(SageError::IoError(err)))
                .and_then(|ant1| {
                    Ant::run(
                        1,
                        to_me.clone(),
                        schema.clone(),
                        YamlVersion::V1x2,
                        atag_handles.clone(),
                    )
                    .or_else(|err| {
                        ant1.0.try_send(Message::Signal(Signal::Terminate)).ok();
                        Err(SageError::IoError(err))
                    })
                    .and_then(|ant2| {
                        Ant::run(
                            2,
                            to_me.clone(),
                            schema.clone(),
                            YamlVersion::V1x2,
                            atag_handles.clone(),
                        )
                        .or_else(|err| {
                            ant1.0.try_send(Message::Signal(Signal::Terminate)).ok();
                            ant2.0.try_send(Message::Signal(Signal::Terminate)).ok();
                            Err(SageError::IoError(err))
                        })
                        .and_then(|ant3| {
                            (Conveyor {
                                pipe: pipe,
                                cin: cin,
                                ex_cin: ex_cin,
                                out: idea_sdr,

                                // data: Data::with_capacity (4),
                                ants: [ant1, ant2, ant3],
                                msgs: 0,

                                buff: None,

                                yaml_version: YamlVersion::V1x2,

                                tag_handles: atag_handles,
                                // _schema: PhantomData,
                            })
                            .execute()
                        })
                    })
                })
            })?;

        Ok((handle, ex_to_me, idea_rvr))
    }

    pub fn execute(mut self) -> Result<(), SageError> {
        let mut dawn = false;
        let mut dusk = false;

        let mut flush_ants = false;
        let mut finish_loop = false;

        let mut job_is_done = false;
        let mut job_is_done_sent = false;

        let mut datum: Option<D> = None;

        let mut buf_literal_block: Option<(usize, Vec<Result<Marker, (u8, usize)>>)> = None;

        'top: loop {
            if let Some(msg) = if self.buff.is_some() {
                Some(self.buff.take().unwrap())
            } else if flush_ants && self.msgs > 0 {
                Some(self.cin.recv().unwrap().1) // TODO: do not panic, do SageError
            } else {
                match self.ex_cin.try_recv() {
                    Err(TryRecvError::Empty) => match self.cin.try_recv() {
                        Err(TryRecvError::Disconnected) => {
                            return Err(SageError::Error(Cow::from("abandoned sage")))
                        }
                        Ok((_, msg)) => Some(msg),
                        Err(TryRecvError::Empty) => None,
                    },
                    Ok(msg) => Some(msg),
                    Err(TryRecvError::Disconnected) => {
                        return Err(SageError::Error(Cow::from("abandoned sage")))
                    }
                }
            } {
                match msg {
                    Clue::Terminate => break 'top,

                    Clue::Version(ver) => match ver {
                        YamlVersion::V1x1 => self.set_version((1, 1))?,
                        YamlVersion::V1x2 => self.set_version((1, 2))?,
                    },

                    Clue::Response(result) => {
                        self.msgs -= 1;

                        match result {
                            Response::TagHandle(_, tag, handle) => {
                                self.reg_tag_handle(tag, handle)?
                            }

                            Response::Alias(id, alias) => self.think(Idea::Alias(id, alias))?,

                            Response::Error(id, message) => self.think(Idea::Error(id, message))?,

                            Response::Node(id, anchor, node) => match node {
                                ant::Node::MetaMap(tag, firstborn_id) => {
                                    self.think(Idea::NodeMetaMap(id, anchor, tag, firstborn_id))?
                                }
                                ant::Node::MetaSeq(tag) => {
                                    self.think(Idea::NodeMetaSeq(id, anchor, tag))?
                                }
                                ant::Node::Dictionary(tag, firstborn_id) => {
                                    self.think(Idea::NodeDictionary(id, anchor, tag, firstborn_id))?
                                }
                                ant::Node::Sequence(tag) => {
                                    self.think(Idea::NodeSequence(id, anchor, tag))?
                                }
                                ant::Node::Scalar(value) => {
                                    self.think(Idea::NodeScalar(id, anchor, value))?
                                }
                                ant::Node::Literal(value) => {
                                    self.think(Idea::NodeLiteral(id, anchor, value))?
                                }
                            },
                        }
                    }
                }

                continue 'top;
            }

            if flush_ants {
                if self.msgs > 0 {
                    continue;
                }

                flush_ants = false;

                if let Some(d) = datum {
                    _conveyor_signal!(self, Signal::Datum(d.clone()))?;
                }
                datum = None;

                if dawn {
                    dawn = false;
                    self.think(Idea::Dawn)?;
                }

                if dusk {
                    dusk = false;
                    self.think(Idea::Dusk)?;
                    _conveyor_signal!(self, Signal::Reset)?;
                    _conveyor_signal!(self, Signal::Version(self.yaml_version))?;

                    for i in 0..self.tag_handles.len() {
                        _conveyor_signal!(self, Signal::TagHandle(self.tag_handles[i].clone()))?;
                    }
                }

                if job_is_done {
                    job_is_done = false;
                    job_is_done_sent = true;
                    self.think(Idea::Done)?;
                }

                if finish_loop {
                    break 'top;
                }
            }

            if let Ok(block) = self.pipe.recv() {
                job_is_done_sent = false;

                match block.cargo {
                    BlockType::StreamEnd => {
                        job_is_done = true;
                        flush_ants = true;
                    }

                    BlockType::Datum(arc) => {
                        datum = Some(arc);
                        flush_ants = true;
                    }

                    BlockType::DirectiveYaml(version) => self.set_version(version)?,

                    BlockType::DirectiveTag((tag, handle)) => {
                        self.convey_request(Request::ReadDirectiveTag(block.id, tag, handle))?
                    }

                    BlockType::DocStart if self.msgs > 0 => {
                        flush_ants = true;
                        dawn = true;
                    }

                    BlockType::DocStart => self.think(Idea::Dawn)?,

                    BlockType::DocEnd if self.msgs > 0 => {
                        flush_ants = true;
                        dusk = true;
                    }

                    BlockType::DocEnd => self.think(Idea::Dusk)?,

                    BlockType::Error(message, position) => {
                        self.think(Idea::ReadError(block.id, position, message))?
                    }

                    BlockType::Warning(message, position) => {
                        self.think(Idea::ReadWarning(block.id, position, message))?
                    }

                    BlockType::Node(Node {
                        anchor: _,
                        tag: _,
                        content: NodeKind::LiteralBlockOpen,
                    }) => {
                        buf_literal_block = Some((block.id.index, Vec::with_capacity(32)));
                    }

                    BlockType::Literal(..) if buf_literal_block.is_some() => {
                        if let BlockType::Literal(chunk) = block.cargo {
                            if let Some((idx, ref mut vec)) = buf_literal_block {
                                if idx != block.id.parent {
                                    panic!("Unexpected literal!")
                                }
                                vec.push(Ok(chunk));
                            }
                        }
                    }

                    BlockType::Byte(..) if buf_literal_block.is_some() => {
                        if let BlockType::Byte(byte, amount) = block.cargo {
                            if let Some((idx, ref mut vec)) = buf_literal_block {
                                if idx != block.id.parent {
                                    panic!("Unexpected literal!")
                                }
                                vec.push(Err((byte, amount)));
                            }
                        }
                    }

                    BlockType::Node(Node {
                        anchor,
                        tag,
                        content: NodeKind::LiteralBlockClose,
                    }) => {
                        let (idx, vec) = buf_literal_block.take().unwrap();
                        if idx != block.id.index {
                            panic!("Unexpected literal block!")
                        }
                        self.convey_request(Request::ReadLiteralBlock(block.id, anchor, tag, vec))?;
                    }

                    BlockType::Alias(..)
                    | BlockType::BlockMap(..)
                    | BlockType::Literal(..)
                    | BlockType::Byte(..)
                    | BlockType::Node(..) => self.convey_request(Request::ReadBlock(block))?,
                }
            } else {
                if self.msgs > 0 {
                    flush_ants = true;
                    finish_loop = true;
                    job_is_done = true;
                } else {
                    if !job_is_done_sent {
                        self.think(Idea::Done)?;
                    }
                    break 'top;
                }
            }
        }

        Ok(())
    }

    fn set_version(&mut self, version: (u8, u8)) -> Result<(), SageError> {
        if version.0 != 1 {
            return Err(SageError::Error(Cow::from(format!(
                "Unsupported yaml version {}.{}",
                version.0, version.1
            ))));
        }

        let ver = if version.1 == 1 {
            YamlVersion::V1x1
        } else {
            YamlVersion::V1x2
        };

        _conveyor_signal!(self, Signal::Version(ver))?;

        Ok(())
    }

    fn reg_tag_handle(
        &mut self,
        shorthand: Cow<'static, str>,
        prefix: Cow<'static, str>,
    ) -> Result<(), SageError> {
        let arc = Arc::new((shorthand, prefix));

        _conveyor_signal!(self, Signal::TagHandle(arc.clone()))
    }

    fn think(&mut self, message: Idea) -> Result<(), SageError> {
        self.out.send(message).or_else(|_| {
            self.terminate();
            Err(SageError::Error(Cow::from("Sage is alone; nobody listens")))
        })
    }

    fn terminate(&mut self) {
        for idx in 0..self.ants.len() {
            self.ants[idx]
                .0
                .try_send(Message::Signal(Signal::Terminate))
                .ok();
        }
    }

    fn convey_request(&mut self, request: Request<D>) -> Result<(), SageError> {
        let mut message = Message::Request(request);

        'top: loop {
            for idx in 0..self.ants.len() {
                let result = self.ants[idx].0.try_send(message);

                if result.is_err() {
                    match result {
                        Err(TrySendError::Disconnected(_)) => {
                            self.terminate();
                            return Err(SageError::Error(Cow::from("One of ants passed away")));
                        }

                        Err(TrySendError::Full(msg)) => {
                            message = msg;
                            continue;
                        }

                        Ok(_) => unreachable!(),
                    }
                } else {
                    self.msgs += 1;
                }

                break 'top;
            }

            if self.buff.is_some() {
                unreachable!();
            } else if self.msgs > 0 {
                self.buff = match self.cin.recv() {
                    Ok((i, m)) => {
                        self.ants[i as usize]
                            .0
                            .send(message)
                            .or_else(|_| {
                                self.terminate();
                                return Err(SageError::Error(Cow::from(
                                    "One of ants has passed away",
                                )));
                            })
                            .and_then(|_| {
                                self.msgs += 1;
                                Ok(())
                            })
                            .ok();
                        Some(m)
                    }
                    Err(_) => return Err(SageError::Error(Cow::from("The ants have passed away"))),
                };

                break;
            } else if self.msgs == 0 {
                self.ants[0]
                    .0
                    .send(message)
                    .or_else(|_| {
                        self.terminate();
                        return Err(SageError::Error(Cow::from("One of ants has passed away")));
                    })
                    .and_then(|_| {
                        self.msgs += 1;
                        Ok(())
                    })
                    .ok();
                break;
            }
        }

        Ok(())
    }
}
