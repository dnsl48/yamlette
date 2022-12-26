extern crate skimmer;

pub mod core;

use crate::model::style::CommonStyles;
use crate::model::{Model, TaggedValue};

use crate::model::yaml::null::Null;
use crate::model::yamlette::literal::Literal;

use std::borrow::Cow;

pub trait Schema: Send + Sync {
    fn get_common_styles(&self) -> CommonStyles;

    fn get_yaml_version(&self) -> (u8, u8);

    fn get_tag_handles(&self) -> &[(Cow<'static, str>, Cow<'static, str>)];

    fn look_up_model<'a, 'b>(&'a self, tag: &'b str) -> Option<&'a dyn Model>;

    fn try_decodable_models(&self, value: &[u8]) -> Option<TaggedValue>;

    fn try_decodable_models_11(&self, value: &[u8]) -> Option<TaggedValue>;

    fn look_up_model_callback(
        &self,
        predicate: &mut dyn FnMut(&dyn Model) -> bool,
    ) -> Option<&dyn Model>;

    fn get_metamodel(&self) -> Option<&dyn Model>;

    fn get_model_literal(&self) -> Literal;

    fn get_model_null(&self) -> Null;

    fn get_tag_model_map(&self) -> Cow<'static, str>;

    fn get_tag_model_seq(&self) -> Cow<'static, str>;
}
