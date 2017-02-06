use model::{ Rope, TaggedValue, Schema };
use model::renderer::{ Renderer, Node };
use txt::{ CharSet, Twine };

use orchestra::OrchError;
use orchestra::performer::{ Performer, PerformerId, Play };

use std::io;
use std::mem;
use std::sync::Arc;
use std::sync::mpsc::{ sync_channel, Receiver, SyncSender, TryRecvError, TrySendError };
use std::thread::{ Builder, JoinHandle };




pub type Level = usize;




#[derive (Debug)]
pub enum Message {
    Hint (Hint),
    Value (Level, TaggedValue)
}



#[derive (Clone, Debug)]
pub enum Signal {
    Terminate,
    Volumes (usize),
    VolumeTags (Option<Arc<Vec<(Twine, Twine)>>>)
}




#[derive (Debug)]
pub enum Hint {
    TheEnd,

    Volumes (usize),

    VolumeEnd,
    VolumeNext,
    VolumeSize (usize),

    DirectiveYaml (bool),
    DirectiveTags (Vec<(Twine, Twine)>),

    BorderTop (bool),
    BorderBot (bool)
}




#[derive (Clone, Debug)]
pub struct Coord {
    pub vol: usize,  // volume number
    pub idx: usize,  // element index within the volume
    pub lvl: usize   // element level within the tree
}



impl Coord {
    pub fn new (vol: usize, idx: usize, lvl: usize) -> Coord { Coord { vol: vol, idx: idx, lvl: lvl } }
}


#[derive (Copy, Clone, Debug)]
pub enum VolumeStyle {
    Yaml,
    Tags,
    TopBorder,
    BotBorder
}


#[derive (Debug)]
pub enum Gesture {
    LookForSignal,
    Chord (Coord, TaggedValue, Vec<Rope>),
    Value (Coord, TaggedValue),
    Style (Coord, VolumeStyle),
    Render (NodeList, StringPointer)
}




#[derive (Debug)]
pub struct NodeList ( *const [Node] );
unsafe impl Send for NodeList {}
impl NodeList {
    pub unsafe fn as_list (&self) -> &[Node] { mem::transmute::<*const [Node], &[Node]> (self.0) }
}



#[derive (Debug)]
pub struct StringPointer ( *mut u8 );
unsafe impl Send for StringPointer {}
impl StringPointer {
    pub fn as_ptr (&self) -> *mut u8 { self.0 }
}




#[derive (Debug)]
pub struct Record {
    pub level: usize,
    state: u8,
    legatos_link: usize,
    play: Option<Play>
}


const RECORD_DONE: u8 = 1;
const RECORD_LEGATO: u8 = 2;
const RECORD_CHORD: u8 = 4;
const RECORD_ZIP_READY: u8 = 8;
const RECORD_ZIPPED: u8 = 16;
const RECORD_BORROWED: u8 = 32;


impl Record {
    pub fn new (level: usize) -> Record { Record { level: level, state: 0, legatos_link: 0, play: None } }

    pub fn set_play (&mut self, play: Play) {
        let mut state = self.state | RECORD_DONE;

        match play {
            Play::Legato (_, _) => { state |= RECORD_LEGATO; }
            Play::Chord (_, _, _) => { state |= RECORD_CHORD; }
            _ => ()
        };

        self.play = Some (play);
        self.state = state & !RECORD_BORROWED;
    }

    pub fn get_rope (&self) -> &Rope { self.play.as_ref ().unwrap ().get_rope () }

    pub fn mark_zip_ready (&mut self) { self.state |= RECORD_ZIP_READY; }

    pub fn take (&mut self) -> Play {
        if self.play.is_none () { panic! ("empty record") }
        if self.is_legato () && !self.is_chord () { panic! ("the record is legato and not chord yet") }

        self.state |= RECORD_ZIPPED;

        self.play.take ().unwrap ()
    }

    pub fn borrow (&mut self) -> Play {
        if self.play.is_none () { panic! ("empty record") }
        if !self.is_legato () { panic! ("the record is not legato") }

        self.state |= RECORD_BORROWED;

        self.play.take ().unwrap ()
    }

    pub fn is_done (&self) -> bool { self.state & RECORD_DONE == RECORD_DONE }

    pub fn is_legato (&self) -> bool { self.state & RECORD_LEGATO == RECORD_LEGATO }

    pub fn is_chord (&self) -> bool { self.state & RECORD_CHORD == RECORD_CHORD }

    pub fn is_zipped (&self) -> bool { self.state & RECORD_ZIPPED == RECORD_ZIPPED }

    pub fn is_notready_or_zipped_or_borrowed (&self) -> bool {
        self.state & (RECORD_DONE | RECORD_ZIP_READY | RECORD_ZIPPED | RECORD_BORROWED) != RECORD_DONE | RECORD_ZIP_READY
    }

    pub fn is_notready_or_notchordlegato_or_borrowed (&self) -> bool {
        let flag = self.state & (RECORD_DONE | RECORD_ZIP_READY | RECORD_BORROWED | RECORD_LEGATO | RECORD_CHORD);

        return flag != RECORD_DONE | RECORD_ZIP_READY && flag != RECORD_DONE | RECORD_ZIP_READY | RECORD_LEGATO | RECORD_CHORD;
    }
}




#[derive (Debug)]
struct Volume {
    styles: u8,
    styled: u8,
    init: bool,
    flat: bool,
    idx: usize,
    size: usize,
    real_records: usize,
    bytes_len: usize,
    zero_level_nodes: usize,
    legatos: Vec<(usize, bool)>,
    records: Vec<Record>,
    tags: Option<Arc<Vec<(Twine, Twine)>>>
}



const VOLUME_STYLE_DIR_YAML: u8 = 1;
const VOLUME_STYLE_DIR_TAGS: u8 = 2;
const VOLUME_STYLE_TOP_BORDER: u8 = 4;
const VOLUME_STYLE_BOT_BORDER: u8 = 8;
const VOLUME_STYLE_BOT_BORDER_EXPLICIT_NO: u8 = 16;



impl Volume {
    pub fn new (idx: usize) -> Volume { Volume {
        idx: idx,
        styles: 0,
        styled: 0,
        tags: None,
        size: 0,
        real_records: 0,
        bytes_len: 0,
        zero_level_nodes: 0,
        init: false,
        flat: false,
        legatos: Vec::new (),
        records: Vec::new ()
    } }


    pub fn initialized (&self) -> bool { self.init }


    pub fn init (&mut self, mut size: usize, prefer_bot_border: bool) {
        self.legatos = Vec::with_capacity (size);

        if self.styles & VOLUME_STYLE_DIR_YAML == VOLUME_STYLE_DIR_YAML {
            self.styles |= VOLUME_STYLE_TOP_BORDER;
        }

        if self.tags.is_some () {
            self.styles |= VOLUME_STYLE_DIR_TAGS | VOLUME_STYLE_TOP_BORDER;
        }

        if prefer_bot_border {
            self.styles |= VOLUME_STYLE_BOT_BORDER;
        }

        if self.styles & VOLUME_STYLE_BOT_BORDER_EXPLICIT_NO == VOLUME_STYLE_BOT_BORDER_EXPLICIT_NO {
            self.styles &= !VOLUME_STYLE_BOT_BORDER;
            self.styles &= !VOLUME_STYLE_BOT_BORDER_EXPLICIT_NO;

        } else if self.styles & VOLUME_STYLE_TOP_BORDER == VOLUME_STYLE_TOP_BORDER {
            self.styles |= VOLUME_STYLE_BOT_BORDER;
        }

        if self.styles & VOLUME_STYLE_DIR_YAML == VOLUME_STYLE_DIR_YAML { size += 1; }
        if self.styles & VOLUME_STYLE_DIR_TAGS == VOLUME_STYLE_DIR_TAGS { size += 1; }
        if self.styles & VOLUME_STYLE_TOP_BORDER == VOLUME_STYLE_TOP_BORDER { size += 1; }
        if self.styles & VOLUME_STYLE_BOT_BORDER == VOLUME_STYLE_BOT_BORDER { size += 1; }

        self.records = Vec::with_capacity (size);

        self.size = size;
        self.init = true;
    }


    pub fn style (&mut self) -> Option<Gesture> {
        if self.styles == self.styled { return None }

        let volume_idx = self.idx;

        if (self.styles & VOLUME_STYLE_DIR_YAML == VOLUME_STYLE_DIR_YAML) && (self.styled & VOLUME_STYLE_DIR_YAML != VOLUME_STYLE_DIR_YAML) {
            self.styled |= VOLUME_STYLE_DIR_YAML;

            self.zero_level_nodes += 1;
            let mut record = Record::new (0);
            record.mark_zip_ready ();
            self.records.push (record);

            let coord = Coord::new (volume_idx, self.records.len () - 1, 0);

            return Some (Gesture::Style (coord, VolumeStyle::Yaml))
        }


        if (self.styles & VOLUME_STYLE_DIR_TAGS == VOLUME_STYLE_DIR_TAGS) && (self.styled & VOLUME_STYLE_DIR_TAGS != VOLUME_STYLE_DIR_TAGS) {
            self.styled |= VOLUME_STYLE_DIR_TAGS;

            self.zero_level_nodes += 1;
            let mut record = Record::new (0);
            record.mark_zip_ready ();
            self.records.push (record);

            let coord = Coord::new (volume_idx, self.records.len () - 1, 0);

            return Some (Gesture::Style (coord, VolumeStyle::Tags))
        }


        if (self.styles & VOLUME_STYLE_TOP_BORDER == VOLUME_STYLE_TOP_BORDER) && (self.styled & VOLUME_STYLE_TOP_BORDER != VOLUME_STYLE_TOP_BORDER) {
            self.styled |= VOLUME_STYLE_TOP_BORDER;

            self.zero_level_nodes += 1;
            let mut record = Record::new (0);
            record.mark_zip_ready ();
            self.records.push (record);

            let coord = Coord::new (volume_idx, self.records.len () - 1, 0);

            return Some (Gesture::Style (coord, VolumeStyle::TopBorder))
        }

        if self.records.len () != self.size - 1 { return None }

        if (self.styles & VOLUME_STYLE_BOT_BORDER == VOLUME_STYLE_BOT_BORDER) && (self.styled & VOLUME_STYLE_BOT_BORDER != VOLUME_STYLE_BOT_BORDER) {
            self.styled |= VOLUME_STYLE_BOT_BORDER;

            self.push (Record::new (0));
            let coord = Coord::new (volume_idx, self.len () - 1, 0);

            return Some (Gesture::Style (coord, VolumeStyle::BotBorder))
        }

        unreachable! ()
    }


    pub fn len (&self) -> usize { self.records.len () }


    pub fn push (&mut self, record: Record) {
        if record.level == 0 { self.zero_level_nodes += 1; }
        self.records.push (record);
        self.mark_zip_ready ();
    }


    pub fn play (&mut self, play: Play) {
        let idx = play.get_coord ().idx;

        let record = unsafe { self.records.get_unchecked_mut (idx) };

        if record.level == 0 { self.bytes_len += play.bytes_len (); }

        let done = record.is_done ();
        if !done { self.real_records += 1; }

        if idx == self.size - 1 { record.mark_zip_ready (); }

        record.set_play (play);

        if record.is_legato () {
            if !done {
                record.legatos_link = self.legatos.len ();
                self.legatos.push ((idx, false));

            } else if record.is_chord () {
                unsafe {
                    self.legatos.get_unchecked_mut (record.legatos_link).1 = true;
                };
            }
        }
    }


    fn mark_zip_ready (&mut self) {
        if self.records.len () < 2 {
            if self.records.len () == self.size {
                for rec in self.records.iter_mut () { rec.mark_zip_ready () }
            }
            return
        }

        let mut ptr = self.records.len () - 1;

        let level = {
            if self.records.len () == self.size {
                0
            } else {
                let latest = unsafe { self.records.get_unchecked (ptr) };
                let penult = unsafe { self.records.get_unchecked (ptr - 1) };

                if penult.level < latest.level { return }

                latest.level
            }
        };

        loop {
            if ptr == 0 { break; }

            ptr -= 1;

            let record = unsafe { self.records.get_unchecked_mut (ptr) };

            if record.level > level {
                record.mark_zip_ready ();
            } else if record.level == level {
                record.mark_zip_ready ();
                break;
            } else { break; }
        }
    }


    pub fn flatten (&mut self) -> Option<Gesture> {
        if self.flat { return None }

        let mut ready = true;

        for &(record_idx, is_chord) in self.legatos.iter () {
            if is_chord { continue; }
            ready = false;

            let level = {
                let record = unsafe { self.records.get_unchecked (record_idx) };
                if record.is_notready_or_zipped_or_borrowed () { continue; }

                record.level
            };


            if record_idx == self.records.len ()-1 {
                let record = unsafe { self.records.get_unchecked_mut (record_idx) };
                let (coord, val) = record.borrow ().unpack_legato_coord_n_val ();
                let gesture = Gesture::Chord (coord, val, Vec::with_capacity (0));

                return Some (gesture)
            }


            let mut doin = false;
            let mut upto = 0;
            let mut ropes_estimated = 0;

            for idx in record_idx + 1 .. self.records.len () {
                let record = unsafe { self.records.get_unchecked (idx) };

                if record.level <= level {
                    doin = true;
                    upto = idx + 1;
                    break;
                }

                if record.is_notready_or_notchordlegato_or_borrowed () { break; }

                if !record.is_zipped () { ropes_estimated += 1; }

                if idx == self.size - 1 {
                    doin = true;
                    upto = idx + 1;
                    break;
                }
            }

            if !doin { continue }

            let mut ropes: Vec<Rope> = Vec::with_capacity (ropes_estimated);
            let (coord, val) = unsafe {
                let record = self.records.get_unchecked_mut (record_idx);
                record.borrow ().unpack_legato_coord_n_val ()
            };

            for idx in record_idx + 1 .. upto {
                if ropes.len () == ropes_estimated { break; }
                let record = unsafe { self.records.get_unchecked_mut (idx) };
                if record.is_zipped () { continue }
                ropes.push (record.take ().unpack_rope ());
            }

            return Some (Gesture::Chord (coord, val, ropes));
        }

        if ready && self.real_records == self.size {
            self.flat = true;

            mem::replace (&mut self.legatos, Vec::with_capacity (0));
        }

        None
    }
}



const PERFORMERS_NUMBER: usize = 3;

pub struct Conductor {
    pipe: Receiver<Message>,
    cin: Receiver<(PerformerId, Play)>,

    out: SyncSender<Vec<u8>>,

    performers: [(SyncSender<Gesture>, SyncSender<Signal>, JoinHandle<()>); PERFORMERS_NUMBER],
    msgs: usize,
    buff: Option<Play>,

    renderer: Arc<Renderer>
}


impl Conductor {
    pub fn run (
        cset: CharSet,
        pipe: Receiver<Message>,
        renderer: Renderer,
        mut schema: Box<Schema>
    )
        -> io::Result<(JoinHandle<Result<(), OrchError>>, Receiver<Vec<u8>>)>
    {
        let (out_sdr, out_rvr): (SyncSender<Vec<u8>>, Receiver<Vec<u8>>) = sync_channel (1);

        let handle = Builder::new ().name ("conductor".to_string ()).spawn (move || {
            let (to_conductor, cin): (SyncSender<(PerformerId, Play)>, Receiver<(PerformerId, Play)>) = sync_channel (PERFORMERS_NUMBER);

            schema.as_mut ().init (&cset);

            let schema = Arc::new (schema);
            let renderer = Arc::new (renderer);

            let permer_0 = Performer::run (0, to_conductor.clone (), renderer.clone (), schema.clone ()) ?;
            let permer_1 = Performer::run (1, to_conductor.clone (), renderer.clone (), schema.clone ()) ?;
            let permer_2 = Performer::run (2, to_conductor, renderer.clone (), schema) ?;

            (Conductor {
                pipe: pipe,
                cin: cin,
                out: out_sdr,

                performers: [permer_0, permer_1, permer_2],
                msgs: 0,

                renderer: renderer,

                buff: None
            }).execute ()

        }) ?;

        Ok ( (handle, out_rvr) )
    }


    fn execute (mut self) -> Result<(), OrchError> {
        let mut performer_buffers: [&[Node]; PERFORMERS_NUMBER] = [&[]; PERFORMERS_NUMBER];
        let mut music: Vec<u8> = Vec::with_capacity (0);
        let mut volumes: Vec<Volume> = Vec::with_capacity (0);

        let result = self._execute (&mut performer_buffers, &mut music, &mut volumes);

        self.terminate ();

        result
    }


    fn play_to_volume (play: Play, volumes: &mut Vec<Volume>) {
        let vol = play.get_coord ().vol;
        unsafe { volumes.get_unchecked_mut (vol).play (play); }
    }


    fn _execute (
        &mut self,
        performer_buffers: &mut [&[Node]; PERFORMERS_NUMBER],
        music: &mut Vec<u8>,
        volumes: &mut Vec<Volume>
    ) -> Result<(), OrchError> {
        let mut vols_num = 0;

        let mut wait = false;
        let mut flush = false;

        let mut do_the_music = false;
        let mut music_length: usize = 0;

        'main_loop: loop {
            if let Some (play) = self.listen_to_play (wait) ? {
                Self::play_to_volume (play, volumes);
                continue 'main_loop;
            }

            wait = false;

            if volumes.len () > 0 && !do_the_music {
                let mut flats = 0;

                for volume in volumes.iter_mut () {
                    if !volume.initialized () { continue }

                    if let Some (gesture) = volume.style () {
                        self.conduct_gesture (gesture) ?;
                        continue 'main_loop;
                    }

                    if volume.flat {
                        flats += 1;
                        continue;
                    }

                    if let Some (gesture) = volume.flatten () {
                        self.conduct_gesture (gesture) ?;
                        continue 'main_loop;

                    } else if volume.flat {
                        flats += 1;
                        continue;
                    }
                }

                if flush {
                    if flats != volumes.len () {
                        if self.msgs == 0 { unreachable! () }
                        wait = true;
                        continue 'main_loop;
                    }

                    for vol in volumes.iter () { music_length += vol.bytes_len; }

                    music.reserve_exact (music_length);
                    unsafe { music.set_len (music_length); }

                    do_the_music = true;
                }
            }

            if do_the_music {
                if self.msgs != 0 { panic! ("Someone still plays something") }

                self.do_the_music (music, performer_buffers, volumes) ?;

                let music = mem::replace (music, Vec::with_capacity (0));

                let result = self.out.send (music);
                if result.is_err () { return Err (OrchError::Error (String::from ("Could not send out the music"))) }

                *volumes = Vec::with_capacity (0);
                music_length = 0;
                do_the_music = false;
                flush = false;
            }


            if let Ok (message) = self.pipe.recv () {
                match message {
                    Message::Hint (hint) => match hint {
                        Hint::Volumes (size) => {
                            self.conduct_signal (Signal::Volumes (size), volumes) ?;
                            vols_num = size;
                            volumes.reserve_exact (size);
                        }
                        Hint::VolumeNext => { let idx = volumes.len (); volumes.push (Volume::new (idx)); }
                        Hint::VolumeSize (size) => {
                            let tags = {
                                let vlen = volumes.len ();
                                let volume = volumes.get_mut (vlen - 1).unwrap ();

                                let tags = if let Some (ref tags) = volume.tags {
                                    Some (tags.clone ())
                                } else { None };

                                volume.init (size, vlen < vols_num);
                                tags
                            };

                            self.conduct_signal (Signal::VolumeTags (tags), volumes) ?;
                        }
                        Hint::VolumeEnd => (),
                        Hint::TheEnd => { flush = true; }
                        Hint::DirectiveYaml (print) => {
                            if print {
                                volumes.last_mut ().unwrap ().styles |= VOLUME_STYLE_DIR_YAML;
                            } else {
                                volumes.last_mut ().unwrap ().styles &= !VOLUME_STYLE_DIR_YAML;
                            }
                        }
                        Hint::BorderTop (print) => {
                            if print {
                                volumes.last_mut ().unwrap ().styles |= VOLUME_STYLE_TOP_BORDER;
                            } else {
                                volumes.last_mut ().unwrap ().styles &= !VOLUME_STYLE_TOP_BORDER;
                            }
                        }
                        Hint::BorderBot (print) => {
                            if print {
                                volumes.last_mut ().unwrap ().styles |= VOLUME_STYLE_BOT_BORDER;
                            } else {
                                volumes.last_mut ().unwrap ().styles |= VOLUME_STYLE_BOT_BORDER_EXPLICIT_NO;
                            }
                        }
                        Hint::DirectiveTags (tags) => {
                            volumes.last_mut ().unwrap ().tags = Some (Arc::new (tags));
                        }
                    },

                    Message::Value (level, value) => {
                        let volume_idx = volumes.len () - 1;
                        let volume = volumes.last_mut ().unwrap ();

                        let coord = Coord::new (volume_idx, volume.len (), level);
                        volume.push (Record::new (level));

                        self.conduct_gesture (Gesture::Value (coord, value)) ?;
                    }
                }
            } else if volumes.len () > 0 {
                flush = true;
            } else { break; }
        }

        Ok ( () )
    }


    fn do_the_music<'a, 'b, 'c> (&'a mut self, music: &'b mut Vec<u8>, performer_buffers: &'b mut [&'c [Node]], vols: &'c Vec<Volume>) -> Result<(), OrchError> {
        let tho: usize = music.len () / PERFORMERS_NUMBER + 1;

        let mut perfs_busy = false;
        let mut perf_idx: usize = 0;

        let mut rope: &Rope;
        let mut rope_idx: usize;

        let mut str_ptr: *mut u8 = music.as_mut_ptr ();

        let renderer: &Renderer = &self.renderer;

        for vol in vols.iter () {
            let mut rec_ptr: usize = 0;
            let mut rec_cnt: usize = 0;

            'vol_loop: loop {
                if rec_cnt == vol.zero_level_nodes { break }
                if rec_ptr >= vol.size { break }

                let record = unsafe { vol.records.get_unchecked (rec_ptr) };

                rec_ptr += 1;

                if record.level != 0 { continue 'vol_loop; }

                rec_cnt += 1;

                rope = record.get_rope ();

                rope_idx = 0;
                'rope_loop: loop {
                    let (size, new_index, done) = rope.unrope (&mut performer_buffers[perf_idx], renderer, rope_idx, tho);
                    rope_idx = new_index;

                    if size == 0 {
                        if done { break } else { continue }
                    }

                    Self::conduct_render (&self.performers[perf_idx].0, &mut self.msgs, Gesture::Render (
                        NodeList (performer_buffers[perf_idx] as *const [Node]),
                        StringPointer (str_ptr)
                    )) ?;

                    str_ptr = unsafe { str_ptr.offset (size as isize) };

                    if !perfs_busy {
                        perf_idx +=1;
                        if perf_idx >= PERFORMERS_NUMBER {
                            perfs_busy = true;
                            perf_idx = Self::listen_to_render (&self.cin, &mut self.msgs) ? as usize;
                        }
                    } else { perf_idx = Self::listen_to_render (&self.cin, &mut self.msgs) ? as usize; }

                    if done { break; }
                }
            }
        }

        loop {
            if self.msgs > 0 {
                Self::listen_to_render (&self.cin, &mut self.msgs) ?
            } else {
                break
            };
        }

        for el in performer_buffers.iter_mut () { *el = &[]; }

        Ok ( () )
    }


    fn conduct_render (sender: &SyncSender<Gesture>, msgs: &mut usize, gesture: Gesture) -> Result<(), OrchError> {
        match sender.send (gesture) {
            Ok ( _ ) => { *msgs += 1; Ok ( () ) }
            Err ( _ ) => Err ( OrchError::Error (String::from ("The performer passed away")) )
        }
    }


    fn conduct_gesture (&mut self, mut gesture: Gesture) -> Result<(), OrchError> {
        'main_loop: loop {
            for idx in 0 .. self.performers.len () {
                let result = self.performers[idx].0.try_send (gesture);

                if result.is_err () {
                    match result {
                        Err (TrySendError::Disconnected (_)) => {
                            self.terminate ();
                            return Err ( OrchError::Error (format! ("One of performers ({}) passed away", idx)) );
                        }

                        Err (TrySendError::Full (g)) => {
                            gesture = g;
                            continue;
                        }

                        Ok ( _ ) => unreachable! ()
                    }
                } else { self.msgs += 1; }

                break 'main_loop;
            }

            if self.msgs > 0 {
                let bval = match self.cin.recv () {
                    Ok ((i, m)) => {
                        self.performers[i as usize].0.send (gesture).or_else (|_| {
                            self.terminate ();
                            return Err ( OrchError::Error (String::from ("One of performers has gone")) )
                        }).and_then (|_| { self.msgs += 1; Ok ( () ) }).ok ();

                        Some (m)
                    },
                    Err (_) => return Err ( OrchError::Error (format! ("The performers have gone").to_string ()) )
                };

                if self.buff.is_some () {
                    unreachable! ()
                } else {
                    self.buff = bval;
                }

                break;
            } else if self.msgs == 0 {
                self.performers[0].0.send (gesture).or_else (|_| {
                    self.terminate ();
                    return Err ( OrchError::Error (String::from ("One of performers has gone")) )
                }).and_then (|_| { self.msgs += 1; Ok ( () ) }).ok ();
                break;
            }
        }

        Ok ( () )
    }


    fn conduct_signal (&mut self, signal: Signal, volumes: &mut Vec<Volume>) -> Result<(), OrchError> {
        for idx in 0 .. self.performers.len () {
            let mut sig = signal.clone ();
            if self.msgs > 0 {
                loop {
                    match self.performers[idx].1.try_send (sig) {
                        Ok (_) => break,
                        Err (error) => match error {
                            TrySendError::Disconnected (_) => {
                                self.terminate ();
                                return Err ( OrchError::Error (String::from ("One of performers has gone")) )
                            }
                            TrySendError::Full (s) => {
                                sig = s;
                                if let Some (play) = self.listen_to_play (true) ? {
                                    Self::play_to_volume (play, volumes)
                                }
                            }
                        }
                    }
                }
            } else {
                self.performers[idx].1.send (signal.clone ()).or_else (|_| {
                    self.terminate ();
                    return Err ( OrchError::Error (String::from ("One of performers has gone")) )
                }).and_then (|_| { Ok ( () ) }).ok ();
            }

            self.performers[idx].0.try_send (Gesture::LookForSignal).ok ();
        }

        Ok ( () )
    }


    fn terminate (&mut self) {
        loop {
            for i in 0 .. self.performers.len () {
                self.performers[i].1.try_send (Signal::Terminate).ok ();
            }

            // we must wait for all performers to finish before terminating!
            // they might write onto a mut pointer right now! (see StringPointer)
            if let Err (_) = self.cin.recv () { break };
        }
    }


    fn listen_to_render (cin: &Receiver<(PerformerId, Play)>, msgs: &mut usize) -> Result<usize, OrchError> {
        match cin.recv () {
            Err (_) => Err ( (OrchError::Error ("abandoned conductor".to_string ())) ),
            Ok ((id, _)) => { *msgs -= 1; Ok (id as usize) }
        }
    }


    fn listen_to_play (&mut self, wait: bool) -> Result<Option<Play>, OrchError> {
        if self.buff.is_some () {
            self.msgs -= 1;
            Ok (Some ( self.buff.take ().unwrap () ))

        } else if wait && self.msgs > 0 {
            match self.cin.recv () {
                Err (_) => Err ( (OrchError::Error ("abandoned conductor".to_string ())) ),
                Ok ((_, play)) => { self.msgs -= 1; Ok (Some (play)) }
            }

        } else {
            match self.cin.try_recv () {
                Err (TryRecvError::Disconnected) => Err ( (OrchError::Error ("abandoned conductor".to_string ())) ),
                Ok ((_, play)) => { self.msgs -= 1; Ok (Some (play)) },
                Err (TryRecvError::Empty) => Ok (None)
            }
        }
    }
}
