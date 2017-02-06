use txt::{ CharSet, Encoding, Twine };

use model::{ Factory, Model, Renderer, Rope, Tagged, TaggedValue };
use model::style::CommonStyles;
use model::yaml::pairs::{ compose, PairsValue };

use std::any::Any;




pub const TAG: &'static str = "tag:yaml.org,2002:omap";
static TWINE_TAG: Twine = Twine::Static (TAG);




pub struct Omap {
    encoding: Encoding
}



impl Omap {
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }

    pub fn new (cset: &CharSet) -> Omap { Omap { encoding: cset.encoding } }
}



impl Model for Omap {
    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

    fn get_encoding (&self) -> Encoding { self.encoding }

    fn is_sequence (&self) -> bool { true }

    fn compose (&self, renderer: &Renderer, value: TaggedValue, tags: &mut Iterator<Item=&(Twine, Twine)>, children: &mut [Rope]) -> Rope {
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
    fn get_tag (&self) -> &Twine { Omap::get_tag () }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



impl Into<PairsValue> for OmapValue {
    fn into (self) -> PairsValue { PairsValue::new (self.styles, self.alias) }
}




pub struct OmapFactory;



impl Factory for OmapFactory {
    fn get_tag (&self) -> &Twine { Omap::get_tag () }

    fn build_model (&self, cset: &CharSet) -> Box<Model> { Box::new (Omap::new (cset)) }
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use model::Factory;

    use txt::get_charset_utf8;



    #[test]
    fn tag () {
        let omap = OmapFactory.build_model (&get_charset_utf8 ());

        assert_eq! (omap.get_tag (), TAG);
    }
}
