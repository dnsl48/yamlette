extern crate skimmer;


pub mod core;


use txt::Twine;
use model::{ Model, TaggedValue };
use model::style::CommonStyles;

use model::yaml::null::Null;
use model::yamlette::literal::Literal;



pub trait Schema: Send + Sync {
    fn get_common_styles (&self) -> CommonStyles;

    fn get_yaml_version (&self) -> (u8, u8);

    fn get_tag_handles (&self) -> &[(Twine, Twine)];

    fn look_up_model<'a, 'b> (&'a self, &'b str) -> Option<&'a Model>;

    fn try_decodable_models (&self, &[u8]) -> Option<TaggedValue>;

    fn try_decodable_models_11 (&self, &[u8]) -> Option<TaggedValue>;

    fn look_up_model_callback (&self, &mut (FnMut (&Model) -> bool)) -> Option<&Model>;

    fn get_metamodel (&self) -> Option<&Model>;

    fn get_model_literal (&self) -> Literal;

    fn get_model_null (&self) -> Null;

    fn get_tag_model_map (&self) -> &Twine;

    fn get_tag_model_seq (&self) -> &Twine;
}
