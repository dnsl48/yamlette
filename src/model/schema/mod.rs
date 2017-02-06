pub mod core;


use txt::{ CharSet, Encoding, Twine };
use model::Model;
use model::style::CommonStyles;



pub trait Schema: Send + Sync {
    fn init (&mut self, &CharSet);

    fn get_encoding (&self) -> Encoding;

    fn get_common_styles (&self) -> CommonStyles;

    fn get_yaml_version (&self) -> (u8, u8);

    fn get_tag_handles (&self) -> &[(Twine, Twine)];

    fn look_up_model<'a, 'b> (&'a self, &'b str) -> Option<&'a Model>;

    fn look_up_model_callback (&self, &mut (FnMut (&Model) -> bool)) -> Option<&Model>;

    fn get_metamodel (&self) -> Option<&Model>;
}
