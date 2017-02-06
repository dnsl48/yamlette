extern crate fraction;
extern crate num;

use self::fraction::BigFraction;
use self::num::BigInt;

use std::collections::HashMap;
use std::hash::Hash;

// use book::word::Word;
use book::extractor::pointer::Pointer;




pub trait FromPointer<'a>: Sized {
    fn from_pointer (Pointer<'a>) -> Option<Self>;
}



macro_rules! from_pointer_impl {
    ($t:ty) => {
        impl<'a> FromPointer<'a> for $t /* where &'a Word: Into<Result<$t, &'a Word>> */ {
            fn from_pointer (pointer: Pointer<'a>) -> Option<Self> { pointer.into::<$t> () }
        }
    };

    (link $t:ty) => {
        impl<'a> FromPointer<'a> for &'a $t /* where &'a Word: Into<Result<&'a $t, &'a Word>> */ {
            fn from_pointer (pointer: Pointer<'a>) -> Option<Self> { pointer.into::<&'a $t> () }
        }
    };
}


from_pointer_impl! (());

from_pointer_impl! (bool);

from_pointer_impl! (char);
from_pointer_impl! (link str);
from_pointer_impl! (String);

from_pointer_impl! (f32);
from_pointer_impl! (f64);

from_pointer_impl! (BigFraction);
from_pointer_impl! (link BigFraction);

from_pointer_impl! (u8);
from_pointer_impl! (i8);
from_pointer_impl! (u16);
from_pointer_impl! (i16);
from_pointer_impl! (u32);
from_pointer_impl! (i32);
from_pointer_impl! (u64);
from_pointer_impl! (i64);
from_pointer_impl! (usize);
from_pointer_impl! (isize);

from_pointer_impl! (BigInt);
from_pointer_impl! (link BigInt);

from_pointer_impl! (Vec<u8>);
from_pointer_impl! (link Vec<u8>);




pub trait List<'a>: Sized {
    fn list_new () -> Self;

    fn list_reserve (&mut self, usize);

    fn list_update (&mut self, val: Pointer<'a>);
}



impl<'a, V> List<'a> for Vec<V>
    where
        V: FromPointer<'a>
{
    fn list_new () -> Self { Vec::new () }

    fn list_reserve (&mut self, size: usize) { self.reserve_exact (size); }

    fn list_update (&mut self, val: Pointer<'a>) {
        if let Some (v) = <V as FromPointer>::from_pointer (val) { self.push (v); }
    }
}




pub trait Dict<'a>: Sized {
    fn dict_new () -> Self;

    fn dict_reserve (&mut self, usize);

    fn dict_update (&mut self, key: Pointer<'a>, val: Pointer<'a>);
}



impl<'a, K, V> Dict<'a> for HashMap<K, V>
    where
        K: FromPointer<'a> + Eq + Hash,
        V: FromPointer<'a>
{
    fn dict_new () -> Self { HashMap::new () }

    fn dict_reserve (&mut self, size: usize) { self.reserve (size) }

    fn dict_update (&mut self, key: Pointer<'a>, val: Pointer<'a>) {
        if let Some (k) = <K as FromPointer>::from_pointer (key) {
            if let Some (v) = <V as FromPointer>::from_pointer(val) {
                self.insert (k, v);
            }
        }
    }
}
