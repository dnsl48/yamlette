extern crate fraction;
extern crate num;

use self::fraction::{BigFraction, Fraction};
use self::num::{BigInt, BigUint};

use crate::model::style::{CommonStyles, Style};
use crate::model::{Tagged, TaggedValue};

use crate::model::yaml::binary;
use crate::model::yaml::bool::BoolValue;
use crate::model::yaml::float::FloatValue;
use crate::model::yaml::int::IntValue;
use crate::model::yaml::map::MapValue;
use crate::model::yaml::null::NullValue;
use crate::model::yaml::omap::OmapValue;
use crate::model::yaml::pairs::PairsValue;
use crate::model::yaml::seq::SeqValue;
use crate::model::yaml::set::SetValue;
use crate::model::yaml::str::StrValue;

use crate::orchestra::{OrchError, Orchestra};

use std::borrow::{Borrow, Cow};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::Hash;

pub struct EmptyList;
pub struct EmptyDict;
pub struct BinaryValue(pub Vec<u8>);
pub struct Omap<Data>(pub Data);
pub struct Pairs<Data>(pub Data);
pub struct Set<Data>(pub Data);

pub fn apply_styles(tagged: &mut dyn Tagged, styles: &mut [&mut dyn Style]) {
    for style in styles {
        style.tagged_styles_apply(tagged);
    }
}

pub trait Chord {
    fn chord_size(&self) -> usize;

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError>;
}

impl Chord for EmptyList {
    fn chord_size(&self) -> usize {
        1
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = SeqValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))
    }
}

impl Chord for EmptyDict {
    fn chord_size(&self) -> usize {
        1
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = MapValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))
    }
}

impl Chord for bool {
    fn chord_size(&self) -> usize {
        1
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = BoolValue::new(self, cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))
    }
}

impl Chord for char {
    fn chord_size(&self) -> usize {
        1
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = StrValue::new(Cow::from(self.to_string()), cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))
    }
}

impl Chord for &'static str {
    fn chord_size(&self) -> usize {
        1
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = StrValue::new(Cow::from(self), cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))
    }
}

impl Chord for String {
    fn chord_size(&self) -> usize {
        1
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = StrValue::new(Cow::from(self), cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))
    }
}

macro_rules! int_impl_for {
    ( $($t:ty),* ) => {
        $(
        impl Chord for $t {
            fn chord_size (&self) -> usize { 1 }

            fn play (self, orchestra: &Orchestra, level: usize, alias: Option<Cow<'static, str>>, cs: CommonStyles, vs: &mut [&mut dyn Style]) -> Result<(), OrchError> {
                let mut val = IntValue::from (self);

                val.init_common_styles (cs);
                val.set_alias (alias);
                apply_styles (&mut val, vs);

                orchestra.play (level, TaggedValue::from (val))
            }
        }
        )*
    };
}

int_impl_for!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize, BigUint, BigInt);

macro_rules! float_impl_for {
    ( $($t:ty),* ) => {
        $(
        impl Chord for $t {
            fn chord_size (&self) -> usize { 1 }

                fn play (self, orchestra: &Orchestra, level: usize, alias: Option<Cow<'static, str>>, cs: CommonStyles, vs: &mut [&mut dyn Style]) -> Result<(), OrchError> {
                let mut val = FloatValue::from (self);

                val.init_common_styles (cs);
                val.set_alias (alias);
                apply_styles (&mut val, vs);

                orchestra.play (level, TaggedValue::from (val))
            }
        }
        )*
    };
}

float_impl_for!(f32, f64, Fraction, BigFraction);

impl Chord for () {
    fn chord_size(&self) -> usize {
        1
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = NullValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))
    }
}

impl Chord for BinaryValue {
    fn chord_size(&self) -> usize {
        1
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = binary::BinaryValue::new(self.0, cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))
    }
}

impl<T> Chord for Vec<T>
where
    T: Chord,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for element in self {
            len += element.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = SeqValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for element in self {
            element.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}

impl<T> Chord for VecDeque<T>
where
    T: Chord,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for element in self {
            len += element.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = SeqValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for element in self {
            element.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}

impl<T> Chord for LinkedList<T>
where
    T: Chord,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for element in self {
            len += element.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = SeqValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for element in self {
            element.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}

impl<T> Chord for BinaryHeap<T>
where
    T: Chord + Ord,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for element in self {
            len += element.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = SeqValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for element in self {
            element.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}

impl<K, V> Chord for HashMap<K, V>
where
    K: Chord + Eq + Hash,
    V: Chord,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for (key, val) in self {
            len += key.chord_size();
            len += val.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = MapValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for (key, val) in self {
            key.play(orchestra, level + 1, None, cs, vs)?;
            val.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}

impl<K, V> Chord for BTreeMap<K, V>
where
    K: Chord,
    V: Chord,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for (key, val) in self {
            len += key.chord_size();
            len += val.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = MapValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for (key, val) in self {
            key.play(orchestra, level + 1, None, cs, vs)?;
            val.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}

impl<K, V> Chord for Omap<BTreeMap<K, V>>
where
    K: Chord,
    V: Chord,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for (key, val) in self.0.borrow().into_iter() {
            len += key.chord_size();
            len += val.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = OmapValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for (key, val) in self.0.into_iter() {
            key.play(orchestra, level + 1, None, cs, vs)?;
            val.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}

impl<K, V> Chord for Pairs<BTreeMap<K, V>>
where
    K: Chord,
    V: Chord,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for (key, val) in self.0.borrow().into_iter() {
            len += key.chord_size();
            len += val.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = PairsValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for (key, val) in self.0.into_iter() {
            key.play(orchestra, level + 1, None, cs, vs)?;
            val.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}

impl<K, V> Chord for Omap<Vec<(K, V)>>
where
    K: Chord,
    V: Chord,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for &(ref key, ref val) in self.0.iter() {
            len += key.chord_size();
            len += val.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = OmapValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for (key, val) in self.0.into_iter() {
            key.play(orchestra, level + 1, None, cs, vs)?;
            val.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}

impl<K, V> Chord for Pairs<Vec<(K, V)>>
where
    K: Chord,
    V: Chord,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for &(ref key, ref val) in self.0.iter() {
            len += key.chord_size();
            len += val.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = PairsValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for (key, val) in self.0.into_iter() {
            key.play(orchestra, level + 1, None, cs, vs)?;
            val.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}

impl<T> Chord for HashSet<T>
where
    T: Chord + Eq + Hash,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for element in self {
            len += element.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = SeqValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for element in self {
            element.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}

impl<T> Chord for BTreeSet<T>
where
    T: Chord,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for element in self {
            len += element.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = SeqValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for element in self {
            element.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}

impl<T> Chord for Set<HashSet<T>>
where
    T: Chord + Eq + Hash,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for element in self.0.iter() {
            len += element.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = SetValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for element in self.0 {
            element.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}

impl<T> Chord for Set<BTreeSet<T>>
where
    T: Chord,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for element in self.0.iter() {
            len += element.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = SetValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for element in self.0 {
            element.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}

impl<T> Chord for Set<Vec<T>>
where
    T: Chord,
{
    fn chord_size(&self) -> usize {
        let mut len = 1;
        for element in self.0.iter() {
            len += element.chord_size();
        }
        len
    }

    fn play(
        self,
        orchestra: &Orchestra,
        level: usize,
        alias: Option<Cow<'static, str>>,
        cs: CommonStyles,
        vs: &mut [&mut dyn Style],
    ) -> Result<(), OrchError> {
        let mut val = SetValue::new(cs, alias);
        apply_styles(&mut val, vs);

        orchestra.play(level, TaggedValue::from(val))?;

        for element in self.0 {
            element.play(orchestra, level + 1, None, cs, vs)?;
        }

        Ok(())
    }
}
