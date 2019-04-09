use model::renderer::{EncodedString, Node, Renderer};
use model::{Rope, Schema, Tagged, TaggedValue};

use orchestra::conductor::{Coord, Gesture, NodeList, Signal, StringPointer, VolumeStyle};

use txt::encoding::UTF8;
use txt::Unicode;

use std::borrow::Cow;
use std::io;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread::{Builder, JoinHandle};

pub type BytesLength = usize;
pub type PerformerId = u8;

#[derive(Debug)]
pub enum Play {
    Legato(Coord, TaggedValue),

    Note(Coord, Rope, BytesLength),

    Chord(Coord, Rope, BytesLength),

    Rendered,
}

impl Play {
    pub fn get_coord(&self) -> &Coord {
        match *self {
            Play::Legato(ref c, _) => c,
            Play::Note(ref c, _, _) => c,
            Play::Chord(ref c, _, _) => c,
            _ => panic!("no coords in here"),
        }
    }

    pub fn unpack_legato_coord_n_val(self) -> (Coord, TaggedValue) {
        match self {
            Play::Legato(c, t) => (c, t),
            _ => panic!("it is not legato"),
        }
    }

    pub fn unpack_rope(self) -> Rope {
        match self {
            Play::Note(_, r, _) => r,
            Play::Chord(_, r, _) => r,
            _ => panic!("no ropes in here"),
        }
    }

    pub fn get_rope(&self) -> &Rope {
        match *self {
            Play::Note(_, ref r, _) => r,
            Play::Chord(_, ref r, _) => r,
            _ => panic!("no ropes in here"),
        }
    }

    pub fn bytes_len(&self) -> usize {
        match *self {
            Play::Note(_, _, l) => l,
            Play::Chord(_, _, l) => l,
            _ => 0,
        }
    }
}

pub struct Performer<S> {
    id: PerformerId,
    signals_in: Receiver<Signal>,
    cin: Receiver<Gesture>,
    out: SyncSender<(PerformerId, Play)>,

    renderer: Renderer,
    schema: S,
}

impl<S> Performer<S>
where
    S: Schema + Clone + 'static,
{
    pub fn run(
        id: PerformerId,
        out: SyncSender<(PerformerId, Play)>,
        renderer: Renderer,
        schema: S,
    ) -> io::Result<(SyncSender<Gesture>, SyncSender<Signal>, JoinHandle<()>)> {
        let (to_me, cin): (SyncSender<Gesture>, Receiver<Gesture>) = sync_channel(2);
        let (to_me_signals, signals_in): (SyncSender<Signal>, Receiver<Signal>) = sync_channel(2);

        let handle = Builder::new()
            .name(format!("performer_{}", id))
            .spawn(move || {
                (Performer {
                    id: id,
                    signals_in: signals_in,
                    cin: cin,
                    out: out,

                    renderer: renderer,
                    schema: schema,
                })
                .execute();
            })?;

        Ok((to_me, to_me_signals, handle))
    }

    pub fn execute(self) {
        // let schema: &Schema = &self.schema;
        // let renderer: Renderer = self.renderer;

        let mut volume_tags: Vec<Option<Arc<Vec<(Cow<'static, str>, Cow<'static, str>)>>>> =
            Vec::new();

        'main_loop: loop {
            if let Ok(signal) = self.signals_in.try_recv() {
                match signal {
                    Signal::Terminate => break 'main_loop,
                    Signal::Volumes(num) => {
                        volume_tags = Vec::with_capacity(num);
                    }
                    Signal::VolumeTags(tags) => volume_tags.push(tags),
                }
            }

            if let Ok(gesture) = self.cin.recv() {
                match gesture {
                    Gesture::LookForSignal => (),
                    Gesture::Render(node_list, string_pointer) => {
                        self.render(node_list, string_pointer)
                    }
                    Gesture::Value(coord, value) => self.play_note(coord, value, &volume_tags),
                    Gesture::Chord(coord, value, children) => {
                        self.play_chord(coord, value, &volume_tags, children)
                    }
                    Gesture::Style(coord, style) => self.play_style(coord, style, &volume_tags),
                };
            } else {
                break 'main_loop;
            }
        }
    }

    fn play_note(
        &self,
        coord: Coord,
        value: TaggedValue,
        tags: &Vec<Option<Arc<Vec<(Cow<'static, str>, Cow<'static, str>)>>>>,
    ) {
        // TODO: error handling (unimplemented!)
        if let Some(model) = self.schema.look_up_model(value.get_tag().as_ref()) {
            if model.is_collection() {
                let res = self.out.send((self.id, Play::Legato(coord, value)));
                if res.is_err() {
                    unimplemented!()
                };
            } else {
                if !model.is_encodable() {
                    unimplemented!()
                }

                let encoded = match *unsafe { tags.get_unchecked(coord.vol) } {
                    Some(ref arc) => model.encode(
                        &self.renderer,
                        value,
                        &mut arc
                            .as_ref()
                            .iter()
                            .chain(self.schema.get_tag_handles().iter()),
                    ),
                    None => model.encode(
                        &self.renderer,
                        value,
                        &mut self.schema.get_tag_handles().iter(),
                    ),
                };

                if let Ok(rope) = encoded {
                    let len = if coord.lvl == 0 {
                        rope.bytes_len(&self.renderer)
                    } else {
                        0
                    };
                    let res = self.out.send((self.id, Play::Note(coord, rope, len)));
                    if res.is_err() {
                        unimplemented!()
                    };
                } else {
                    unimplemented!()
                }
            }
        } else {
            unimplemented!()
        }
    }

    fn play_chord(
        &self,
        coord: Coord,
        value: TaggedValue,
        tags: &Vec<Option<Arc<Vec<(Cow<'static, str>, Cow<'static, str>)>>>>,
        mut children: Vec<Rope>,
    ) {
        if let Some(model) = self.schema.look_up_model(value.get_tag().as_ref()) {
            if !model.is_collection() {
                unimplemented!();
            }

            let rope = match *unsafe { tags.get_unchecked(coord.vol) } {
                Some(ref arc) => model.compose(
                    &self.renderer,
                    value,
                    &mut arc
                        .as_ref()
                        .iter()
                        .chain(self.schema.get_tag_handles().iter()),
                    children.as_mut_slice(),
                ),
                None => model.compose(
                    &self.renderer,
                    value,
                    &mut self.schema.get_tag_handles().iter(),
                    children.as_mut_slice(),
                ),
            };

            let len = if coord.lvl == 0 {
                rope.bytes_len(&self.renderer)
            } else {
                0
            };
            let res = self.out.send((self.id, Play::Chord(coord, rope, len)));

            if res.is_err() {
                unimplemented!()
            }
        } else {
            unimplemented!()
        }
    }

    fn play_style(
        &self,
        coord: Coord,
        style: VolumeStyle,
        tags: &Vec<Option<Arc<Vec<(Cow<'static, str>, Cow<'static, str>)>>>>,
    ) {
        let rope = match style {
            VolumeStyle::Yaml => {
                let (maj, min) = self.schema.get_yaml_version();

                let encoding = UTF8; // schema.get_encoding ();

                let node = if maj == 1 && min == 2 {
                    match encoding.str_to_bytes("%YAML 1.2") {
                        Ok(s) => Node::StringNewline(EncodedString::from(s)),
                        Err(s) => Node::StringNewline(EncodedString::from(s)),
                    }
                } else if maj == 1 && min == 1 {
                    match encoding.str_to_bytes("%YAML 1.1") {
                        Ok(s) => Node::StringNewline(EncodedString::from(s)),
                        Err(s) => Node::StringNewline(EncodedString::from(s)),
                    }
                } else {
                    Node::StringNewline(EncodedString::from(
                        encoding.string_to_bytes(format!("%YAML {}.{}", maj, min)),
                    ))
                };

                Rope::from(node)
            }

            VolumeStyle::Tags => {
                let encoding = UTF8; // schema.get_encoding ();

                match *unsafe { tags.get_unchecked(coord.vol) } {
                    Some(ref arc) => {
                        let tags: &Vec<(Cow<'static, str>, Cow<'static, str>)> = arc.as_ref();
                        let mut nodes: Vec<Node> = Vec::with_capacity(tags.len());

                        for &(ref shortcut, ref handle) in tags {
                            nodes.push(Node::StringNewline(EncodedString::from(
                                encoding.string_to_bytes(format!(
                                    "%TAG {} {}",
                                    shortcut.as_ref(),
                                    handle.as_ref()
                                )),
                            )));
                        }

                        Rope::from(nodes)
                    }
                    None => Rope::Empty,
                }
            }

            VolumeStyle::TopBorder => Rope::from(Node::TripleHyphenNewline),
            VolumeStyle::BotBorder => Rope::from(Node::TripleDotNewline),
        };

        let len = rope.bytes_len(&self.renderer);
        let res = self.out.send((self.id, Play::Note(coord, rope, len)));
        if res.is_err() {
            unimplemented!()
        };
    }

    fn render(&self, node_list: NodeList, string_pointer: StringPointer) {
        unsafe {
            let nodes = node_list.as_list();
            let mut ptr = string_pointer.as_ptr();

            for node in nodes {
                ptr = self.renderer.render_onto_ptr(ptr, node);
            }
        }

        let res = self.out.send((self.id, Play::Rendered));

        if res.is_err() {
            unimplemented!()
        }
    }
}
