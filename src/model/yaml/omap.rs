extern crate skimmer;


use model::{ Model, Renderer, Rope, Tagged, TaggedValue };
use model::style::CommonStyles;
use model::yaml::pairs::{ compose, PairsValue };

use std::any::Any;
use std::borrow::Cow;




pub static TAG: &'static str = "tag:yaml.org,2002:omap";




#[derive (Clone, Copy)]
pub struct Omap;



impl Omap {
    pub fn get_tag () -> Cow<'static, str> { Cow::from (TAG) }
}



impl Model for Omap {
    fn get_tag (&self) -> Cow<'static, str> { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

    fn is_sequence (&self) -> bool { true }

    fn compose (&self, renderer: &Renderer, value: TaggedValue, tags: &mut Iterator<Item=&(Cow<'static, str>, Cow<'static, str>)>, children: &mut [Rope]) -> Rope {
        let value: PairsValue = match <TaggedValue as Into<Result<OmapValue, TaggedValue>>>::into (value) {
            Ok (value) => value.into (),
            Err (_) => panic! ("Not an OmapValue")
        };

        compose (self, renderer, TaggedValue::from (value), tags, children)
    }
}




#[derive (Debug)]
pub struct OmapValue {
    styles: CommonStyles,
    alias: Option<Cow<'static, str>>
}



impl OmapValue {
    pub fn new (styles: CommonStyles, alias: Option<Cow<'static, str>>) -> OmapValue { OmapValue { styles: styles, alias: alias } }

    pub fn take_alias (&mut self) -> Option<Cow<'static, str>> { self.alias.take () }
}



impl Tagged for OmapValue {
    fn get_tag (&self) -> Cow<'static, str> { Cow::from (TAG) }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



impl Into<PairsValue> for OmapValue {
    fn into (self) -> PairsValue { PairsValue::new (self.styles, self.alias) }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    // use txt::get_charset_utf8;



    #[test]
    fn tag () {
        let omap = Omap; // ::new (&get_charset_utf8 ());

        assert_eq! (omap.get_tag (), TAG);
    }
}
