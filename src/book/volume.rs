use std::collections::HashMap;

use crate::sage::Idea;

use crate::book::word::Word;

use crate::model::yaml::map;
use crate::model::yaml::seq;

use std::borrow::Cow;

pub struct Volume {
    pub complete: bool,
    pub gist: Vec<(Option<String>, usize, Word)>,

    buff: Option<HashMap<usize, Idea>>,
}

impl Volume {
    pub fn new() -> Volume {
        Volume {
            complete: false,
            gist: Vec::with_capacity(0),
            buff: Some(HashMap::with_capacity(256)),
        }
    }

    pub fn complete(&mut self) {
        if self.complete {
            return;
        }

        let mut buff = self.buff.take().unwrap();

        self.gist.reserve_exact(buff.len());

        let mut border: usize = 0;

        for ix in buff.keys() {
            border = *ix;
            break;
        }

        /* 1 is always Dawn, so starting from 2 */
        for ix in 2..border {
            if buff.contains_key(&ix) {
                border = ix;
                break;
            }
        }

        let mut ix = border;

        for _ in 0..buff.len() {
            loop {
                if let Some(idea) = buff.remove(&ix) {
                    self.process(idea);
                    ix += 1;
                    break;
                } else {
                    ix += 1;
                    continue;
                }
            }
        }

        self.complete = true;
    }

    fn process(&mut self, idea: Idea) {
        match idea {
            Idea::Error(id, value) => self
                .gist
                .push((None, id.level, Word::Err(Cow::from(value)))),
            Idea::ReadError(id, _, string) => {
                self.gist
                    .push((None, id.level, Word::Err(Cow::from(string))))
            }
            Idea::ReadWarning(id, _, string) => {
                self.gist
                    .push((None, id.level, Word::Wrn(Cow::from(string))))
            }

            Idea::NodeLiteral(id, alias, value) => {
                self.gist.push((alias, id.level, Word::Str(value)))
            }
            Idea::NodeScalar(id, alias, value) => {
                self.gist
                    .push((alias, id.level, Word::extract_scalar(value)))
            }

            Idea::NodeSequence(id, alias, tag) => {
                self.gist.push((alias, id.level, Word::Seq(Cow::from(tag))))
            }
            Idea::NodeMetaSeq(id, alias, None) => {
                self.gist
                    .push((alias, id.level, Word::Seq(Cow::from(seq::TAG))))
            }
            Idea::NodeMetaSeq(id, alias, Some(tag)) => {
                self.gist.push((alias, id.level, Word::Seq(Cow::from(tag))))
            }

            Idea::NodeDictionary(id, alias, _, firstborn_id) => {
                self.gist
                    .push((alias, id.level, Word::Map(Cow::from(map::TAG))));

                if firstborn_id.is_some() {
                    // TODO: check whether it's ALWAYS the previous node?
                    let ln = self.gist.len();
                    let mut firstborn = self.gist.swap_remove(ln - 2);
                    firstborn.1 += 1; // level up
                    self.gist.push(firstborn);
                }
            }

            Idea::NodeMetaMap(id, alias, tag, firstborn_id) => {
                if let Some(tag) = tag {
                    self.gist.push((alias, id.level, Word::Map(Cow::from(tag))));
                } else {
                    self.gist
                        .push((alias, id.level, Word::Map(Cow::from(map::TAG))));
                }

                if firstborn_id.is_some() {
                    // TODO: check whether it's ALWAYS the previous node?
                    let ln = self.gist.len();
                    let mut firstborn = self.gist.swap_remove(ln - 2);
                    firstborn.1 += 1; // level up
                    self.gist.push(firstborn);
                }
            }

            Idea::Alias(id, value) => {
                let mut narr: Option<Word> = None;

                for (ix, &(ref alias, _, _)) in self.gist.iter().enumerate().rev() {
                    if let Some(ref alias) = *alias {
                        if *alias == value {
                            narr = Some(Word::Alias(ix));
                            break;
                        }
                    }
                }

                if narr.is_some() {
                    self.gist.push((None, id.level, narr.take().unwrap()));
                } else {
                    self.gist.push((None, id.level, Word::UnboundAlias(value)));
                }
            }

            Idea::Done | Idea::Dawn | Idea::Dusk => unreachable!(),
        };
    }

    pub fn stamp(&mut self, idea: Idea) {
        if self.complete {
            return;
        }

        let ix = match idea {
            Idea::Alias(ref id, _) => id.index,
            Idea::Error(ref id, _) => id.index,
            Idea::NodeMetaMap(ref id, _, _, _) => id.index,
            Idea::NodeMetaSeq(ref id, _, _) => id.index,
            Idea::NodeDictionary(ref id, _, _, _) => id.index,
            Idea::NodeSequence(ref id, _, _) => id.index,
            Idea::NodeScalar(ref id, _, _) => id.index,
            Idea::NodeLiteral(ref id, _, _) => id.index,
            Idea::ReadError(ref id, _, _) => id.index,
            Idea::ReadWarning(ref id, _, _) => id.index,

            _ => 0,
        };

        if ix > 0 {
            self.buff.as_mut().unwrap().insert(ix, idea);
        }
    }

    pub fn unalias(&self, idx: usize) -> &Word {
        let &(_, _, ref word) = &self.gist[idx];

        match *word {
            Word::Alias(idx) => self.unalias(idx),
            _ => word,
        }
    }
}
