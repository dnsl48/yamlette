extern crate skimmer;

use self::skimmer::symbol::{ CopySymbol, Combo };


use txt::{ CharSet, Encoding, Twine };

use model::{ Model, Renderer, Rope, Tagged, TaggedValue };
use model::style::CommonStyles;
use model::yaml::pairs::{ compose, PairsValue };

use std::any::Any;
use std::marker::PhantomData;




pub const TAG: &'static str = "tag:yaml.org,2002:omap";
static TWINE_TAG: Twine = Twine::Static (TAG);




pub struct Omap<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    encoding: Encoding,
    _char: PhantomData<Char>,
    _dchr: PhantomData<DoubleChar>
}



impl<Char, DoubleChar> Omap<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }

    pub fn new (cset: &CharSet<Char, DoubleChar>) -> Omap<Char, DoubleChar> { Omap {
        encoding: cset.encoding,
        _char: PhantomData,
        _dchr: PhantomData
    } }
}



impl<Char, DoubleChar> Model for Omap<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    type Char = Char;
    type DoubleChar = DoubleChar;

    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

    fn get_encoding (&self) -> Encoding { self.encoding }

    fn is_sequence (&self) -> bool { true }

    fn compose (&self, renderer: &Renderer<Char, DoubleChar>, value: TaggedValue, tags: &mut Iterator<Item=&(Twine, Twine)>, children: &mut [Rope]) -> Rope {
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
    alias: Option<Twine>
}



impl OmapValue {
    pub fn new (styles: CommonStyles, alias: Option<Twine>) -> OmapValue { OmapValue { styles: styles, alias: alias } }

    pub fn take_alias (&mut self) -> Option<Twine> { self.alias.take () }
}



impl Tagged for OmapValue {
    fn get_tag (&self) -> &Twine { &TWINE_TAG }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



impl Into<PairsValue> for OmapValue {
    fn into (self) -> PairsValue { PairsValue::new (self.styles, self.alias) }
}



/*
pub struct OmapFactory;

impl Factory for OmapFactory {
    fn get_tag (&self) -> &Twine { &TWINE_TAG }

    fn build_model<Char: CopySymbol + 'static, DoubleChar: CopySymbol + Combo + 'static> (&self, cset: &CharSet<Char, DoubleChar>) -> Box<Model<Char=Char, DoubleChar=DoubleChar>> { Box::new (Omap::new (cset)) }
}
*/



#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use txt::get_charset_utf8;



    #[test]
    fn tag () {
        let omap = Omap::new (&get_charset_utf8 ());

        assert_eq! (omap.get_tag (), TAG);
    }
}
