use crate::book::extractor::traits::FromPointer;
use crate::book::volume::Volume;
use crate::book::word::Word;

#[derive(Copy, Clone)]
pub struct Pointer<'a> {
    vol: &'a Volume,
    pos: usize,
}

impl<'a, T> PartialEq<T> for Pointer<'a>
where
    T: PartialEq + FromPointer<'a>,
{
    fn eq(&self, other: &T) -> bool {
        if let Some(v) = <T as FromPointer>::from_pointer(*self) {
            *other == v
        } else {
            false
        }
    }
}

impl<'a> Pointer<'a> {
    pub fn new(vol: &'a Volume) -> Option<Self> {
        if vol.gist.len() > 0 {
            Some(Pointer { vol: vol, pos: 0 })
        } else {
            None
        }
    }

    pub fn into<T>(self) -> Option<T>
    where
        &'a Word: Into<Result<T, &'a Word>>,
    {
        let result: Result<T, _> = self.unalias().to_word().into();
        result.ok()
    }

    pub fn unalias(self) -> Pointer<'a> {
        match self.vol.gist[self.pos] {
            (_, _, Word::Alias(sz)) => Pointer::unalias(Pointer {
                vol: self.vol,
                pos: sz,
            }),
            _ => self,
        }
    }

    pub fn to_word(self) -> &'a Word {
        let (_, _, ref word) = self.vol.gist[self.pos];
        word
    }

    pub fn next_sibling(self) -> Option<Pointer<'a>> {
        let (_, level, _) = self.vol.gist[self.pos];

        for i in (self.pos + 1)..self.vol.gist.len() {
            let (_, sub, _) = self.vol.gist[i];
            if sub == level {
                return Some(Pointer {
                    vol: self.vol,
                    pos: i,
                });
            }
            if sub < level {
                break;
            };
        }

        None
    }

    pub fn count_siblings(self) -> usize {
        let (_, level, _) = self.vol.gist[self.pos];

        let mut cnt = 1;
        for i in (self.pos + 1)..self.vol.gist.len() {
            let (_, sub, _) = self.vol.gist[i];
            if sub == level {
                cnt += 1;
            }
            if sub < level {
                break;
            };
        }

        cnt
    }

    pub fn into_seq(self) -> Option<Pointer<'a>> {
        let ptr = self.unalias();

        let (_, level, ref word) = self.vol.gist[ptr.pos];

        match *word {
            Word::Seq(_) => {
                if let Some(&(_, sublevel, _)) = self.vol.gist.get(ptr.pos + 1) {
                    if sublevel == level + 1 {
                        Some(Pointer {
                            vol: ptr.vol,
                            pos: ptr.pos + 1,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn into_map(self) -> Option<Pointer<'a>> {
        let ptr = self.unalias();

        let (_, level, ref word) = self.vol.gist[ptr.pos];

        match *word {
            Word::Map(_) => {
                if let Some(&(_, sublevel, _)) = self.vol.gist.get(ptr.pos + 1) {
                    if sublevel == level + 1 {
                        Some(Pointer {
                            vol: ptr.vol,
                            pos: ptr.pos + 1,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
