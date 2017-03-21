extern crate skimmer;

use self::skimmer::symbol::{ Combo, CopySymbol };


use model::{ Rope, Tagged, TaggedValue, Schema };
use model::renderer::{ EncodedString, Renderer, Node };

use orchestra::conductor::{ Coord, Gesture, NodeList, Signal, StringPointer, VolumeStyle };

use txt::{ Twine, Unicode };

use std::io;
use std::sync::Arc;
use std::sync::mpsc::{ sync_channel, Receiver, SyncSender };
use std::thread::{ Builder, JoinHandle };


pub type BytesLength = usize;
pub type PerformerId = u8;


#[derive (Debug)]
pub enum Play {
    Legato (Coord, TaggedValue),

    Note (Coord, Rope, BytesLength),

    Chord (Coord, Rope, BytesLength),

    Rendered
}



impl Play {
    pub fn get_coord (&self) -> &Coord {
        match *self {
            Play::Legato (ref c, _) => c,
            Play::Note (ref c, _, _) => c,
            Play::Chord (ref c, _, _) => c,
            _ => panic! ("no coords in here")
        }
    }

    pub fn unpack_legato_coord_n_val (self) -> (Coord, TaggedValue) {
        match self {
            Play::Legato (c, t) => (c, t),
            _ => panic! ("it is not legato")
        }
    }

    pub fn unpack_rope (self) -> Rope {
        match self {
            Play::Note (_, r, _) => r,
            Play::Chord (_, r, _) => r,
            _ => panic! ("no ropes in here")
        }
    }

    pub fn get_rope (&self) -> &Rope {
        match *self {
            Play::Note (_, ref r, _) => r,
            Play::Chord (_, ref r, _) => r,
            _ => panic! ("no ropes in here")
        }
    }

    pub fn bytes_len (&self) -> usize {
        match *self {
            Play::Note (_, _, l) => l,
            Play::Chord (_, _, l) => l,
            _ => 0
        }
    }
}




pub struct Performer<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    id: PerformerId,
    signals_in: Receiver<Signal>,
    cin: Receiver<Gesture>,
    out: SyncSender<(PerformerId, Play)>,

    renderer: Arc<Renderer<Char, DoubleChar>>,
    schema: Arc<Box<Schema<Char, DoubleChar>>>
}



impl<Char, DoubleChar> Performer<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    pub fn run (
        id: PerformerId,
        out: SyncSender<(PerformerId, Play)>,
        renderer: Arc<Renderer<Char, DoubleChar>>,
        schema: Arc<Box<Schema<Char, DoubleChar>>>
    )
        -> io::Result<(SyncSender<Gesture>, SyncSender<Signal>, JoinHandle<()>)>
    {
        let (to_me, cin): (SyncSender<Gesture>, Receiver<Gesture>) = sync_channel (2);
        let (to_me_signals, signals_in): (SyncSender<Signal>, Receiver<Signal>) = sync_channel (2);

        let handle = Builder::new ().name (format! ("performer_{}", id)).spawn (move || {
            ( Performer {
                id: id,
                signals_in: signals_in,
                cin: cin,
                out: out,

                renderer: renderer,
                schema: schema
            } ).execute ();
        }) ?;

        Ok ( (to_me, to_me_signals, handle) )
    }

    pub fn execute (self) {
        let schema: &Schema<Char, DoubleChar> = self.schema.as_ref ().as_ref ();
        let renderer: &Renderer<Char, DoubleChar> = self.renderer.as_ref ();

        let mut volume_tags: Vec<Option<Arc<Vec<(Twine, Twine)>>>> = Vec::new ();

        'main_loop: loop {
            if let Ok (signal) = self.signals_in.try_recv () {
                match signal {
                    Signal::Terminate => break 'main_loop,
                    Signal::Volumes (num) => { volume_tags = Vec::with_capacity (num); }
                    Signal::VolumeTags (tags) => volume_tags.push (tags)
                }
            }

            if let Ok (gesture) = self.cin.recv () {
                match gesture {
                    Gesture::LookForSignal => (),
                    Gesture::Render (node_list, string_pointer) => self.render (renderer, node_list, string_pointer),
                    Gesture::Value (coord, value) => self.play_note (schema, renderer, coord, value, &volume_tags),
                    Gesture::Chord (coord, value, children) => self.play_chord (schema, renderer, coord, value, &volume_tags, children),
                    Gesture::Style (coord, style) => self.play_style (schema, renderer, coord, style, &volume_tags)
                };
            } else { break 'main_loop }
        };
    }


    fn play_note (
        &self,
        schema: &Schema<Char, DoubleChar>,
        renderer: &Renderer<Char, DoubleChar>,
        coord: Coord,
        value: TaggedValue,
        tags: &Vec<Option<Arc<Vec<(Twine, Twine)>>>>
    ) {
        if let Some (model) = schema.look_up_model (value.get_tag ().as_ref ()) {
            if model.is_collection () {
                let res = self.out.send ((self.id, Play::Legato (coord, value)));
                if res.is_err () { unimplemented! () };

            } else {
                if !model.is_encodable () { unimplemented! () }

                let encoded = match *unsafe { tags.get_unchecked (coord.vol) } {
                    Some ( ref arc ) => model.encode (renderer, value, &mut arc.as_ref ().iter ().chain (schema.get_tag_handles ().iter ())),
                    None => model.encode (renderer, value, &mut schema.get_tag_handles ().iter ())
                };

                if let Ok (rope) = encoded {
                    let len = if coord.lvl == 0 { rope.bytes_len (renderer) } else { 0 };
                    let res = self.out.send ((self.id, Play::Note (coord, rope, len)));
                    if res.is_err () { unimplemented! () };
                } else { unimplemented! () }
            }
        } else { unimplemented! () }
    }


    fn play_chord (
        &self,
        schema: &Schema<Char, DoubleChar>,
        renderer: &Renderer<Char, DoubleChar>,
        coord: Coord,
        value: TaggedValue,
        tags: &Vec<Option<Arc<Vec<(Twine, Twine)>>>>,
        mut children: Vec<Rope>
    ) {
        if let Some (model) = schema.look_up_model (value.get_tag ().as_ref ()) {
            if !model.is_collection () { unimplemented! (); }

            let rope = match *unsafe { tags.get_unchecked (coord.vol) } {
                Some ( ref arc ) => model.compose (renderer, value, &mut arc.as_ref ().iter ().chain (schema.get_tag_handles ().iter ()), children.as_mut_slice ()),
                None => model.compose (renderer, value, &mut schema.get_tag_handles ().iter (), children.as_mut_slice ()),
            };

            let len = if coord.lvl == 0 { rope.bytes_len (renderer) } else { 0 };
            let res = self.out.send ((self.id, Play::Chord (coord, rope, len)));

            if res.is_err () { unimplemented! () }
        } else { unimplemented! () }
    }


    fn play_style (
        &self,
        schema: &Schema<Char, DoubleChar>,
        renderer: &Renderer<Char, DoubleChar>,
        coord: Coord,
        style: VolumeStyle,
        tags: &Vec<Option<Arc<Vec<(Twine, Twine)>>>>
    ) {
        let rope = match style {
            VolumeStyle::Yaml => {
                let (maj, min) = schema.get_yaml_version ();

                let encoding = schema.get_encoding ();

                let node = if maj == 1 && min == 2 {
                    match encoding.str_to_bytes ("%YAML 1.2") {
                        Ok (s) => Node::StringNewline (EncodedString::from (s)),
                        Err (s) => Node::StringNewline (EncodedString::from (s))
                    }
                } else if maj == 1 && min == 1 {
                    match encoding.str_to_bytes ("%YAML 1.1") {
                        Ok (s) => Node::StringNewline (EncodedString::from (s)),
                        Err (s) => Node::StringNewline (EncodedString::from (s))
                    }
                } else {
                    Node::StringNewline (EncodedString::from (encoding.string_to_bytes (format! ("%YAML {}.{}", maj, min))))
                };

                Rope::from (node)
            }

            VolumeStyle::Tags => {
                let encoding = schema.get_encoding ();

                match *unsafe { tags.get_unchecked (coord.vol) } {
                    Some ( ref arc ) => {
                        let tags: &Vec<(Twine, Twine)> = arc.as_ref ();
                        let mut nodes: Vec<Node> = Vec::with_capacity (tags.len ());

                        for &(ref shortcut, ref handle) in tags {
                            nodes.push (Node::StringNewline (EncodedString::from (encoding.string_to_bytes (format! ("%TAG {} {}", shortcut.as_ref (), handle.as_ref ())))));
                        }

                        Rope::from (nodes)
                    }
                    None => Rope::Empty
                }
            }

            VolumeStyle::TopBorder => Rope::from (Node::TripleHyphenNewline),
            VolumeStyle::BotBorder => Rope::from (Node::TripleDotNewline)
        };

        let len = rope.bytes_len (renderer);
        let res = self.out.send ((self.id, Play::Note (coord, rope, len)));
        if res.is_err () { unimplemented! () };
    }


    fn render (
        &self,
        renderer: &Renderer<Char, DoubleChar>,
        node_list: NodeList,
        string_pointer: StringPointer
    ) {
        unsafe {
            let nodes = node_list.as_list ();
            let mut ptr = string_pointer.as_ptr ();

            for node in nodes {
                ptr = renderer.render_onto_ptr (ptr, node);
            }
        }

        let res = self.out.send ((self.id, Play::Rendered));

        if res.is_err () { unimplemented! () }
    }
}
