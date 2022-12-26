use crate::model::Tagged;

use crate::model::yaml::binary::BinaryValue;
use crate::model::yaml::bool::BoolValue;
use crate::model::yaml::float::FloatValue;
use crate::model::yaml::int::IntValue;
use crate::model::yaml::map::MapValue;
use crate::model::yaml::merge::MergeValue;
use crate::model::yaml::null::NullValue;
use crate::model::yaml::omap::OmapValue;
use crate::model::yaml::pairs::PairsValue;
use crate::model::yaml::seq::SeqValue;
use crate::model::yaml::set::SetValue;
use crate::model::yaml::str::StrValue;
use crate::model::yaml::timestamp::TimestampValue;
use crate::model::yaml::value::ValueValue;
use crate::model::yaml::yaml::YamlValue;

use crate::model::yamlette::incognitum::IncognitumValue;
use crate::model::yamlette::literal::LiteralValue;

use std::any::Any;
use std::borrow::Cow;

#[derive(Debug)]
pub enum TaggedValue {
    Binary(BinaryValue),
    Bool(BoolValue),
    Float(FloatValue),
    Int(IntValue),
    Map(MapValue),
    Merge(MergeValue),
    Null(NullValue),
    Omap(OmapValue),
    Pairs(PairsValue),
    Seq(SeqValue),
    Set(SetValue),
    Str(StrValue),
    Timestamp(TimestampValue),
    Value(ValueValue),
    Yaml(YamlValue),

    Literal(LiteralValue),
    Incognitum(IncognitumValue),

    Other(Cow<'static, str>, Box<dyn Any + Send>),
}

impl TaggedValue {
    pub fn new(tag: Cow<'static, str>, value: Box<dyn Any + Send>) -> Self {
        TaggedValue::Other(tag, value)
    }

    pub fn get_boxed(self) -> Box<dyn Any + Send> {
        match self {
            TaggedValue::Binary(v) => Box::new(v),
            TaggedValue::Bool(v) => Box::new(v),
            TaggedValue::Float(v) => Box::new(v),
            TaggedValue::Int(v) => Box::new(v),
            TaggedValue::Map(v) => Box::new(v),
            TaggedValue::Merge(v) => Box::new(v),
            TaggedValue::Null(v) => Box::new(v),
            TaggedValue::Omap(v) => Box::new(v),
            TaggedValue::Pairs(v) => Box::new(v),
            TaggedValue::Seq(v) => Box::new(v),
            TaggedValue::Set(v) => Box::new(v),
            TaggedValue::Str(v) => Box::new(v),
            TaggedValue::Timestamp(v) => Box::new(v),
            TaggedValue::Value(v) => Box::new(v),
            TaggedValue::Yaml(v) => Box::new(v),

            TaggedValue::Literal(v) => Box::new(v),
            TaggedValue::Incognitum(v) => Box::new(v),

            TaggedValue::Other(_, b) => b,
        }
    }
}

impl Tagged for TaggedValue {
    fn get_tag(&self) -> Cow<'static, str> {
        match *self {
            TaggedValue::Binary(ref v) => v.get_tag(),
            TaggedValue::Bool(ref v) => v.get_tag(),
            TaggedValue::Float(ref v) => v.get_tag(),
            TaggedValue::Int(ref v) => v.get_tag(),
            TaggedValue::Map(ref v) => v.get_tag(),
            TaggedValue::Merge(ref v) => v.get_tag(),
            TaggedValue::Null(ref v) => v.get_tag(),
            TaggedValue::Omap(ref v) => v.get_tag(),
            TaggedValue::Pairs(ref v) => v.get_tag(),
            TaggedValue::Seq(ref v) => v.get_tag(),
            TaggedValue::Set(ref v) => v.get_tag(),
            TaggedValue::Str(ref v) => v.get_tag(),
            TaggedValue::Timestamp(ref v) => v.get_tag(),
            TaggedValue::Value(ref v) => v.get_tag(),
            TaggedValue::Yaml(ref v) => v.get_tag(),

            TaggedValue::Literal(ref v) => v.get_tag(),
            TaggedValue::Incognitum(ref v) => Tagged::get_tag(v),

            TaggedValue::Other(ref tag, _) => tag.clone(),
        }
    }

    fn as_any(&self) -> &dyn Any {
        match *self {
            TaggedValue::Binary(ref v) => v.as_any(),
            TaggedValue::Bool(ref v) => v.as_any(),
            TaggedValue::Float(ref v) => v.as_any(),
            TaggedValue::Int(ref v) => v.as_any(),
            TaggedValue::Map(ref v) => v.as_any(),
            TaggedValue::Merge(ref v) => v.as_any(),
            TaggedValue::Null(ref v) => v.as_any(),
            TaggedValue::Omap(ref v) => v.as_any(),
            TaggedValue::Pairs(ref v) => v.as_any(),
            TaggedValue::Seq(ref v) => v.as_any(),
            TaggedValue::Set(ref v) => v.as_any(),
            TaggedValue::Str(ref v) => v.as_any(),
            TaggedValue::Timestamp(ref v) => v.as_any(),
            TaggedValue::Value(ref v) => v.as_any(),
            TaggedValue::Yaml(ref v) => v.as_any(),

            TaggedValue::Literal(ref v) => v.as_any(),
            TaggedValue::Incognitum(ref v) => v.as_any(),

            TaggedValue::Other(_, ref value) => value,
        }
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        match *self {
            TaggedValue::Binary(ref mut v) => v.as_mut_any(),
            TaggedValue::Bool(ref mut v) => v.as_mut_any(),
            TaggedValue::Float(ref mut v) => v.as_mut_any(),
            TaggedValue::Int(ref mut v) => v.as_mut_any(),
            TaggedValue::Map(ref mut v) => v.as_mut_any(),
            TaggedValue::Merge(ref mut v) => v.as_mut_any(),
            TaggedValue::Null(ref mut v) => v.as_mut_any(),
            TaggedValue::Omap(ref mut v) => v.as_mut_any(),
            TaggedValue::Pairs(ref mut v) => v.as_mut_any(),
            TaggedValue::Seq(ref mut v) => v.as_mut_any(),
            TaggedValue::Set(ref mut v) => v.as_mut_any(),
            TaggedValue::Str(ref mut v) => v.as_mut_any(),
            TaggedValue::Timestamp(ref mut v) => v.as_mut_any(),
            TaggedValue::Value(ref mut v) => v.as_mut_any(),
            TaggedValue::Yaml(ref mut v) => v.as_mut_any(),

            TaggedValue::Literal(ref mut v) => v.as_mut_any(),
            TaggedValue::Incognitum(ref mut v) => v.as_mut_any(),

            TaggedValue::Other(_, ref mut value) => value,
        }
    }
}

macro_rules! impl_from_into {
    ( $constructor:path => $value:ty ) => {
        impl From<$value> for TaggedValue {
            fn from(value: $value) -> Self {
                $constructor(value)
            }
        }

        impl Into<Result<$value, TaggedValue>> for TaggedValue {
            fn into(self) -> Result<$value, Self> {
                match self {
                    $constructor(v) => Ok(v),
                    _ => Err(self),
                }
            }
        }
    };
}

impl_from_into! ( TaggedValue::Binary     => BinaryValue );
impl_from_into! ( TaggedValue::Bool       => BoolValue );
impl_from_into! ( TaggedValue::Float      => FloatValue );
impl_from_into! ( TaggedValue::Int        => IntValue );
impl_from_into! ( TaggedValue::Map        => MapValue );
impl_from_into! ( TaggedValue::Merge      => MergeValue );
impl_from_into! ( TaggedValue::Null       => NullValue );
impl_from_into! ( TaggedValue::Omap       => OmapValue );
impl_from_into! ( TaggedValue::Pairs      => PairsValue );
impl_from_into! ( TaggedValue::Seq        => SeqValue );
impl_from_into! ( TaggedValue::Set        => SetValue );
impl_from_into! ( TaggedValue::Str        => StrValue );
impl_from_into! ( TaggedValue::Timestamp  => TimestampValue );
impl_from_into! ( TaggedValue::Value      => ValueValue );
impl_from_into! ( TaggedValue::Yaml       => YamlValue );
impl_from_into! ( TaggedValue::Literal    => LiteralValue );
impl_from_into! ( TaggedValue::Incognitum => IncognitumValue );
