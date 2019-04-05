extern crate fraction;
extern crate num;

use self::fraction::BigFraction;
use self::num::{BigInt, ToPrimitive};

use model::{Fraction, Tagged, TaggedValue};

use model::yaml::binary::BinaryValue;
use model::yaml::bool::BoolValue;
use model::yaml::float::FloatValue;
use model::yaml::int::IntValue;
use model::yaml::null::NullValue;
use model::yaml::str::StrValue;
use model::yamlette::literal::LiteralValue;

use std::borrow::Cow;
use std::cmp::PartialEq;

#[derive(Debug)]
pub enum Word {
    Bin(Vec<u8>),
    Bool(bool),
    Int(IntValue),
    Str(String),
    Float(FloatValue),
    Null,

    Alias(usize),

    Seq(Cow<'static, str>),
    Map(Cow<'static, str>),

    Scalar(TaggedValue),

    Err(Cow<'static, str>),
    Wrn(Cow<'static, str>),
    UnboundAlias(String),
}

impl PartialEq for Word {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl Word {
    pub fn extract_scalar(value: TaggedValue) -> Word {
        let bin: Result<BinaryValue, TaggedValue> = value.into();
        let value = match bin {
            Ok(v) => return Word::Bin(v.to_vec()),
            Err(v) => v,
        };

        let bol: Result<BoolValue, TaggedValue> = value.into();
        let value = match bol {
            Ok(v) => return Word::Bool(v.to_bool()),
            Err(v) => v,
        };

        let int: Result<IntValue, TaggedValue> = value.into();
        let value = match int {
            Ok(v) => return Word::Int(v),
            Err(v) => v,
        };

        let lit: Result<LiteralValue, TaggedValue> = value.into();
        let value = match lit {
            Ok(v) => return Word::Str(v.to_cow().into()),
            Err(v) => v,
        };

        let str: Result<StrValue, TaggedValue> = value.into();
        let value = match str {
            Ok(v) => return Word::Str(v.to_twine().into()),
            Err(v) => v,
        };

        let flo: Result<FloatValue, TaggedValue> = value.into();
        let value = match flo {
            Ok(v) => return Word::Float(v),
            Err(v) => v,
        };

        let nil: Result<NullValue, TaggedValue> = value.into();
        let value = match nil {
            Ok(_) => return Word::Null,
            Err(v) => v,
        };

        Word::Scalar(value)
    }
}

impl Into<Result<String, Word>> for Word {
    fn into(self) -> Result<String, Word> {
        match self {
            Word::Str(str) => return Ok(str),
            Word::Scalar(tagged_value) => {
                if let Some(_) = tagged_value.as_any().downcast_ref::<String>() {
                    let s: Box<String> = tagged_value.get_boxed().downcast().ok().unwrap();
                    return Ok(*s.clone());
                } else {
                    Err(Word::Scalar(tagged_value))
                }
            }
            _ => Err(self),
        }
    }
}

impl<'a> Into<Result<String, &'a Word>> for &'a Word {
    fn into(self) -> Result<String, &'a Word> {
        match *self {
            Word::Str(ref str) => Ok(str.to_string()),
            Word::Scalar(ref tagged_value) => {
                if let Some(str) = tagged_value.as_any().downcast_ref::<String>() {
                    Ok(str.to_string())
                } else {
                    Err(self)
                }
            }
            _ => Err(self),
        }
    }
}

impl<'a> Into<Result<&'a str, &'a Word>> for &'a Word {
    fn into(self) -> Result<&'a str, &'a Word> {
        match *self {
            Word::Str(ref str) => Ok(str),
            Word::Scalar(ref tagged_value) => {
                if let Some(str) = tagged_value.as_any().downcast_ref::<String>() {
                    Ok(str)
                } else {
                    Err(self)
                }
            }
            _ => Err(self),
        }
    }
}

impl Into<Result<char, Word>> for Word {
    fn into(self) -> Result<char, Word> {
        match self {
            Word::Str(ref str) => {
                if str.len() == 1 {
                    return Ok(str.chars().next().unwrap());
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(ref str) = tagged_value.as_any().downcast_ref::<String>() {
                    if str.len() == 1 {
                        return Ok(str.chars().next().unwrap());
                    }
                }
            }
            _ => (),
        };
        Err(self)
    }
}

impl<'a> Into<Result<char, &'a Word>> for &'a Word {
    fn into(self) -> Result<char, &'a Word> {
        match *self {
            Word::Str(ref str) => {
                if str.len() == 1 {
                    return Ok(str.chars().next().unwrap());
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(ref str) = tagged_value.as_any().downcast_ref::<String>() {
                    if str.len() == 1 {
                        return Ok(str.chars().next().unwrap());
                    }
                }
            }
            _ => (),
        };
        Err(self)
    }
}

impl Into<Result<bool, Word>> for Word {
    fn into(self) -> Result<bool, Word> {
        match self {
            Word::Bool(v) => return Ok(v),
            Word::Scalar(ref tagged_value) => {
                if let Some(ref b) = tagged_value.as_any().downcast_ref::<bool>() {
                    return Ok(**b);
                }
            }
            _ => (),
        };
        Err(self)
    }
}

impl<'a> Into<Result<bool, &'a Word>> for &'a Word {
    fn into(self) -> Result<bool, &'a Word> {
        match *self {
            Word::Bool(ref v) => return Ok(*v),
            Word::Scalar(ref tagged_value) => {
                if let Some(ref b) = tagged_value.as_any().downcast_ref::<bool>() {
                    return Ok(**b);
                }
            }
            _ => (),
        };
        Err(self)
    }
}

impl Into<Result<Fraction, Word>> for Word {
    fn into(self) -> Result<Fraction, Word> {
        match self {
            Word::Float(val) => return Ok(val.into()),
            Word::Scalar(tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<Fraction>() {
                    return Ok(val.clone());
                }

                return Err(Word::Scalar(tagged_value));
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<Fraction, &'a Word>> for &'a Word {
    fn into(self) -> Result<Fraction, &'a Word> {
        match *self {
            Word::Float(ref val) => {
                let res: &'a Fraction = val.into();
                return Ok(res.clone());
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<Fraction>() {
                    Ok(val.clone())
                } else {
                    Err(self)
                }
            }
            _ => Err(self),
        }
    }
}

impl<'a> Into<Result<&'a Fraction, &'a Word>> for &'a Word {
    fn into(self) -> Result<&'a Fraction, &'a Word> {
        match *self {
            Word::Float(ref val) => return Ok(val.into()),
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<Fraction>() {
                    Ok(val)
                } else {
                    Err(self)
                }
            }
            _ => Err(self),
        }
    }
}

impl Into<Result<BigFraction, Word>> for Word {
    fn into(self) -> Result<BigFraction, Word> {
        match self {
            Word::Float(val) => return Ok(val.into()),
            Word::Scalar(tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigFraction>() {
                    return Ok(val.clone());
                }

                return Err(Word::Scalar(tagged_value));
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<BigFraction, &'a Word>> for &'a Word {
    fn into(self) -> Result<BigFraction, &'a Word> {
        match *self {
            Word::Float(ref val) => Ok(val.clone().into()),
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigFraction>() {
                    Ok(val.clone())
                } else {
                    Err(self)
                }
            }
            _ => Err(self),
        }
    }
}

impl Into<Result<f32, Word>> for Word {
    fn into(self) -> Result<f32, Word> {
        match self {
            Word::Float(ref val) => match val.to_f32() {
                Some(v) => return Ok(v),
                None => (),
            },
            Word::Int(ref val) => match val.to_f32() {
                Some(v) => return Ok(v),
                None => (),
            },
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<f32>() {
                    return Ok(*val);
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<f32, &'a Word>> for &'a Word {
    fn into(self) -> Result<f32, &'a Word> {
        match *self {
            Word::Float(ref val) => match val.to_f32() {
                Some(v) => return Ok(v),
                None => (),
            },
            Word::Int(ref val) => match val.to_f32() {
                Some(v) => return Ok(v),
                None => (),
            },
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<f32>() {
                    return Ok(*val);
                }
            }
            _ => (),
        }

        Err(self)
    }
}

impl Into<Result<f64, Word>> for Word {
    fn into(self) -> Result<f64, Word> {
        match self {
            Word::Float(ref val) => match val.to_f64() {
                Some(v) => return Ok(v),
                None => (),
            },
            Word::Int(ref val) => match val.to_f64() {
                Some(v) => return Ok(v),
                None => (),
            },
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<f64>() {
                    return Ok(*val);
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<f64, &'a Word>> for &'a Word {
    fn into(self) -> Result<f64, &'a Word> {
        match *self {
            Word::Float(ref val) => match val.to_f64() {
                Some(v) => return Ok(v),
                None => (),
            },
            Word::Int(ref val) => match val.to_f64() {
                Some(v) => return Ok(v),
                None => (),
            },
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<f64>() {
                    return Ok(*val);
                }
            }
            _ => (),
        }

        Err(self)
    }
}

impl Into<Result<BigInt, Word>> for Word {
    fn into(self) -> Result<BigInt, Word> {
        match self {
            Word::Int(val) => return Ok(val.into()),
            /*
                        Word::Float (ref val) => {
                            let val: BigFraction = val.clone ().into ();
                            if let Some (den) = val.denom () {
                                if *den == BigUint::one () {
                                    let num = val.numer ().unwrap ();
                                    let sig = val.sign ().unwrap ();

                                    let sig = match *sig {
                                        fraction::Sign::Plus => num::bigint::Sign::Plus,
                                        fraction::Sign::Minus => num::bigint::Sign::Minus
                                    };

                                    return Ok (BigInt::from_biguint (sig, num.clone ()));
                                }
                            }
                        }
            */
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    return Ok(val.clone()); // TODO: unbox instead of cloning?
                }
            }
            _ => (),
        }

        Err(self)
    }
}

impl<'a> Into<Result<BigInt, &'a Word>> for &'a Word {
    fn into(self) -> Result<BigInt, &'a Word> {
        match *self {
            Word::Int(ref val) => Ok(val.clone().into()),
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    Ok(val.clone())
                } else {
                    Err(self)
                }
            }
            _ => Err(self),
        }
    }
}

impl<'a> Into<Result<&'a BigInt, &'a Word>> for &'a Word {
    fn into(self) -> Result<&'a BigInt, &'a Word> {
        match *self {
            Word::Int(ref val) => {
                let maybe: Option<&'a BigInt> = val.into();
                if let Some(v) = maybe {
                    Ok(v)
                } else {
                    Err(self)
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    Ok(val)
                } else {
                    Err(self)
                }
            }
            _ => Err(self),
        }
    }
}

impl Into<Result<u8, Word>> for Word {
    fn into(self) -> Result<u8, Word> {
        match self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_u8() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_u8() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_u8() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<u8, &'a Word>> for &'a Word {
    fn into(self) -> Result<u8, &'a Word> {
        match *self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_u8() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_u8() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_u8() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl Into<Result<i8, Word>> for Word {
    fn into(self) -> Result<i8, Word> {
        match self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_i8() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_i8() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_i8() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<i8, &'a Word>> for &'a Word {
    fn into(self) -> Result<i8, &'a Word> {
        match *self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_i8() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_i8() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_i8() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl Into<Result<u16, Word>> for Word {
    fn into(self) -> Result<u16, Word> {
        match self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_u16() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_u16() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_u16() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<u16, &'a Word>> for &'a Word {
    fn into(self) -> Result<u16, &'a Word> {
        match *self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_u16() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_u16() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_u16() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl Into<Result<i16, Word>> for Word {
    fn into(self) -> Result<i16, Word> {
        match self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_i16() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_i16() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_i16() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<i16, &'a Word>> for &'a Word {
    fn into(self) -> Result<i16, &'a Word> {
        match *self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_i16() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_i16() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_i16() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl Into<Result<u32, Word>> for Word {
    fn into(self) -> Result<u32, Word> {
        match self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_u32() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_u32() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_u32() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<u32, &'a Word>> for &'a Word {
    fn into(self) -> Result<u32, &'a Word> {
        match *self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_u32() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_u32() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_u32() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl Into<Result<i32, Word>> for Word {
    fn into(self) -> Result<i32, Word> {
        match self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_i32() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_i32() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_i32() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<i32, &'a Word>> for &'a Word {
    fn into(self) -> Result<i32, &'a Word> {
        match *self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_i32() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_i32() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_i32() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl Into<Result<u64, Word>> for Word {
    fn into(self) -> Result<u64, Word> {
        match self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_u64() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_u64() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_u64() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<u64, &'a Word>> for &'a Word {
    fn into(self) -> Result<u64, &'a Word> {
        match *self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_u64() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_u64() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_u64() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl Into<Result<i64, Word>> for Word {
    fn into(self) -> Result<i64, Word> {
        match self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_i64() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_i64() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_i64() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<i64, &'a Word>> for &'a Word {
    fn into(self) -> Result<i64, &'a Word> {
        match *self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_i64() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_i64() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_i64() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl Into<Result<usize, Word>> for Word {
    fn into(self) -> Result<usize, Word> {
        match self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_usize() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_usize() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_usize() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<usize, &'a Word>> for &'a Word {
    fn into(self) -> Result<usize, &'a Word> {
        match *self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_usize() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_usize() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_usize() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl Into<Result<isize, Word>> for Word {
    fn into(self) -> Result<isize, Word> {
        match self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_isize() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_isize() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_isize() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<isize, &'a Word>> for &'a Word {
    fn into(self) -> Result<isize, &'a Word> {
        match *self {
            Word::Int(ref val) => {
                if let Some(i) = val.to_isize() {
                    return Ok(i);
                }
            }
            Word::Float(ref val) => {
                if let Some(i) = val.to_isize() {
                    return Ok(i);
                }
            }
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<BigInt>() {
                    if let Some(i) = val.to_isize() {
                        return Ok(i);
                    }
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl Into<Result<Vec<u8>, Word>> for Word {
    fn into(self) -> Result<Vec<u8>, Word> {
        match self {
            Word::Bin(val) => return Ok(val),
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<Vec<u8>>() {
                    return Ok(val.clone()); // TODO: unbox instead of copying
                }
            }
            _ => (),
        };

        Err(self)
    }
}

impl<'a> Into<Result<Vec<u8>, &'a Word>> for &'a Word {
    fn into(self) -> Result<Vec<u8>, &'a Word> {
        match *self {
            Word::Bin(ref val) => return Ok(val.clone()),
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<Vec<u8>>() {
                    return Ok(val.clone());
                }
            }
            _ => (),
        }

        Err(self)
    }
}

impl<'a> Into<Result<&'a Vec<u8>, &'a Word>> for &'a Word {
    fn into(self) -> Result<&'a Vec<u8>, &'a Word> {
        match *self {
            Word::Bin(ref val) => return Ok(val),
            Word::Scalar(ref tagged_value) => {
                if let Some(val) = tagged_value.as_any().downcast_ref::<Vec<u8>>() {
                    return Ok(val);
                }
            }
            _ => (),
        }

        Err(self)
    }
}

impl Into<Result<(), Word>> for Word {
    fn into(self) -> Result<(), Word> {
        match self {
            Word::Null => return Ok(()),
            Word::Scalar(ref tagged_value) => {
                if let Some(_) = tagged_value.as_any().downcast_ref::<()>() {
                    return Ok(());
                }
            }
            _ => (),
        }

        Err(self)
    }
}

impl<'a> Into<Result<(), &'a Word>> for &'a Word {
    fn into(self) -> Result<(), &'a Word> {
        match *self {
            Word::Null => return Ok(()),
            Word::Scalar(ref tagged_value) => {
                if let Some(_) = tagged_value.as_any().downcast_ref::<()>() {
                    return Ok(());
                }
            }
            _ => (),
        }

        Err(self)
    }
}
