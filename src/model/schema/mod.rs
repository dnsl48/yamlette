extern crate skimmer;

use self::skimmer::symbol::{ CopySymbol, Combo };


pub mod core;


use txt::{ CharSet, Encoding, Twine };
use model::{ Model, TaggedValue };
use model::style::CommonStyles;

use model::yaml::null::Null;
use model::yamlette::literal::Literal;



pub trait Schema<Char, DoubleChar>: Send + Sync
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    fn init (&mut self, &CharSet<Char, DoubleChar>);

    fn get_encoding (&self) -> Encoding;

    fn get_common_styles (&self) -> CommonStyles;

    fn get_yaml_version (&self) -> (u8, u8);

    fn get_tag_handles (&self) -> &[(Twine, Twine)];

    fn look_up_model<'a, 'b> (&'a self, &'b str) -> Option<&'a Model<Char=Char, DoubleChar=DoubleChar>>;

    fn try_decodable_models (&self, &[u8]) -> Option<TaggedValue>;

    fn try_decodable_models_11 (&self, &[u8]) -> Option<TaggedValue>;

    // fn look_up_decodable_model_callback (&self, (FnMut (&Model<Char=Char, DoubleChar=DoubleChar>) -> bool)) -> Option<&Model<Char=Char, DoubleChar=DoubleChar>>;

    fn look_up_model_callback (&self, &mut (FnMut (&Model<Char=Char, DoubleChar=DoubleChar>) -> bool)) -> Option<&Model<Char=Char, DoubleChar=DoubleChar>>;

    fn get_metamodel (&self) -> Option<&Model<Char=Char, DoubleChar=DoubleChar>>;

    fn get_model_literal (&self) -> &Literal<Char, DoubleChar>;

    fn get_model_null (&self) -> &Null<Char, DoubleChar>;

    fn get_tag_model_map (&self) -> &Twine;

    fn get_tag_model_seq (&self) -> &Twine;
}
