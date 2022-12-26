extern crate skimmer;

use self::skimmer::data::Datum;

pub mod extractor;
pub mod volume;
pub mod word;

use crate::model::schema::Schema;
use crate::sage::{Idea, Sage};

use self::volume::Volume;

use std::sync::mpsc::Receiver;

pub struct Book {
    pub volumes: Vec<Volume>,
}

impl Book {
    pub fn new() -> Book {
        Book::with_capacity(1)
    }

    pub fn get_written<S, D>(&mut self, author: &Sage<S, D>)
    where
        S: Schema + 'static,
        D: Datum + Sync + Send + 'static,
    {
        let ideas: &Receiver<Idea> = author;
        for idea in ideas {
            if self.stamp(idea) {
                break;
            }
        }
    }

    pub fn with_capacity(size: usize) -> Book {
        Book {
            volumes: Vec::with_capacity(size),
        }
    }

    pub fn stamp(&mut self, idea: Idea) -> bool {
        match idea {
            Idea::Done => return true,
            Idea::Dawn => self.volumes.push(Volume::new()),
            Idea::Dusk => {
                if let Some(vol) = self.volumes.last_mut() {
                    vol.complete()
                }
            }
            idea @ _ => {
                if let Some(vol) = self.volumes.last_mut() {
                    vol.stamp(idea)
                }
            }
        };

        false
    }
}
