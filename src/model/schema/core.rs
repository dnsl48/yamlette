extern crate skimmer;

use crate::model::schema::Schema;

use crate::model::style::CommonStyles;
use crate::model::{Model, TaggedValue};

use crate::model::yaml::binary::Binary;
use crate::model::yaml::bool::Bool;
use crate::model::yaml::float::Float;
use crate::model::yaml::int::Int;
use crate::model::yaml::map::Map;
use crate::model::yaml::merge::Merge;
use crate::model::yaml::null::Null;
use crate::model::yaml::omap::Omap;
use crate::model::yaml::pairs::Pairs;
use crate::model::yaml::seq::Seq;
use crate::model::yaml::set::Set;
use crate::model::yaml::str::Str;
use crate::model::yaml::timestamp::Timestamp;
use crate::model::yaml::value::Value;
use crate::model::yaml::yaml::Yaml;

use crate::model::yamlette::incognitum::Incognitum;
use crate::model::yamlette::literal::Literal;

use std::borrow::Cow;
use std::clone::Clone;
use std::default::Default;

// #[derive (Clone)]
pub struct Core {
    styles: CommonStyles,
    tag_handles: [(Cow<'static, str>, Cow<'static, str>); 3],

    mod_map: Map,
    mod_set: Set,
    mod_pairs: Pairs,
    mod_seq: Seq,
    mod_omap: Omap,
    mod_null: Null,
    mod_bool: Bool,
    mod_int: Int,
    mod_float: Float,
    mod_str: Str,
    mod_merge: Merge,
    mod_value: Value,
    mod_yaml: Yaml,
    mod_timestamp: Timestamp,
    mod_binary: Binary,
    mod_literal: Literal,
    mod_incognitum: Incognitum,
}

impl Schema for Core {
    #[inline(always)]
    fn get_common_styles(&self) -> CommonStyles {
        self.styles
    }

    #[inline(always)]
    fn get_yaml_version(&self) -> (u8, u8) {
        (1, 2)
    }

    fn get_tag_handles(&self) -> &[(Cow<'static, str>, Cow<'static, str>)] {
        &self.tag_handles
    }

    fn look_up_model<'a, 'b>(&'a self, tag: &'b str) -> Option<&'a dyn Model> {
        if tag == Map::get_tag() {
            Some(&self.mod_map)
        } else if tag == Set::get_tag() {
            Some(&self.mod_set)
        } else if tag == Pairs::get_tag() {
            Some(&self.mod_pairs)
        } else if tag == Seq::get_tag() {
            Some(&self.mod_seq)
        } else if tag == Omap::get_tag() {
            Some(&self.mod_omap)
        } else if tag == Null::get_tag() {
            Some(&self.mod_null)
        } else if tag == Bool::get_tag() {
            Some(&self.mod_bool)
        } else if tag == Int::get_tag() {
            Some(&self.mod_int)
        } else if tag == Float::get_tag() {
            Some(&self.mod_float)
        } else if tag == Str::get_tag() {
            Some(&self.mod_str)
        } else if tag == Merge::get_tag() {
            Some(&self.mod_merge)
        } else if tag == Value::get_tag() {
            Some(&self.mod_value)
        } else if tag == Yaml::get_tag() {
            Some(&self.mod_yaml)
        } else if tag == Timestamp::get_tag() {
            Some(&self.mod_timestamp)
        } else if tag == Binary::get_tag() {
            Some(&self.mod_binary)
        } else if tag == Literal::get_tag() {
            Some(&self.mod_literal)
        } else if tag == Incognitum::get_tag() {
            Some(&self.mod_incognitum)
        } else {
            None
        }
    }

    fn try_decodable_models(&self, value: &[u8]) -> Option<TaggedValue> {
        if let Ok(value) = self.mod_null.decode(false, value) {
            Some(value)
        } else if let Ok(value) = self.mod_bool.decode(false, value) {
            Some(value)
        } else if let Ok(value) = self.mod_int.decode(false, value) {
            Some(value)
        } else if let Ok(value) = self.mod_float.decode(false, value) {
            Some(value)
        } else if let Ok(value) = self.mod_str.decode(false, value) {
            Some(value)
        } else if let Ok(value) = self.mod_incognitum.decode(false, value) {
            Some(value)
        } else {
            None
        }
    }

    fn try_decodable_models_11(&self, value: &[u8]) -> Option<TaggedValue> {
        if let Ok(value) = self.mod_null.decode11(false, value) {
            Some(value)
        } else if let Ok(value) = self.mod_bool.decode11(false, value) {
            Some(value)
        } else if let Ok(value) = self.mod_int.decode11(false, value) {
            Some(value)
        } else if let Ok(value) = self.mod_float.decode11(false, value) {
            Some(value)
        } else if let Ok(value) = self.mod_str.decode11(false, value) {
            Some(value)
        } else if let Ok(value) = self.mod_incognitum.decode11(false, value) {
            Some(value)
        } else {
            None
        }
    }

    fn look_up_model_callback(
        &self,
        predicate: &mut dyn FnMut(&dyn Model) -> bool,
    ) -> Option<&dyn Model> {
        if predicate(&self.mod_map) {
            Some(&self.mod_map)
        } else if predicate(&self.mod_set) {
            Some(&self.mod_set)
        } else if predicate(&self.mod_pairs) {
            Some(&self.mod_pairs)
        } else if predicate(&self.mod_seq) {
            Some(&self.mod_seq)
        } else if predicate(&self.mod_omap) {
            Some(&self.mod_omap)
        } else if predicate(&self.mod_null) {
            Some(&self.mod_null)
        } else if predicate(&self.mod_bool) {
            Some(&self.mod_bool)
        } else if predicate(&self.mod_int) {
            Some(&self.mod_int)
        } else if predicate(&self.mod_float) {
            Some(&self.mod_float)
        } else if predicate(&self.mod_str) {
            Some(&self.mod_str)
        } else if predicate(&self.mod_merge) {
            Some(&self.mod_merge)
        } else if predicate(&self.mod_value) {
            Some(&self.mod_value)
        } else if predicate(&self.mod_yaml) {
            Some(&self.mod_yaml)
        } else if predicate(&self.mod_timestamp) {
            Some(&self.mod_timestamp)
        } else if predicate(&self.mod_binary) {
            Some(&self.mod_binary)
        } else if predicate(&self.mod_literal) {
            Some(&self.mod_literal)
        } else if predicate(&self.mod_incognitum) {
            Some(&self.mod_incognitum)
        } else {
            None
        }
    }

    fn get_metamodel(&self) -> Option<&dyn Model> {
        Some(&self.mod_incognitum)
    }

    #[inline(always)]
    fn get_model_literal(&self) -> Literal {
        self.mod_literal
    }

    #[inline(always)]
    fn get_model_null(&self) -> Null {
        self.mod_null
    }

    #[inline(always)]
    fn get_tag_model_map(&self) -> Cow<'static, str> {
        Map::get_tag()
    }

    #[inline(always)]
    fn get_tag_model_seq(&self) -> Cow<'static, str> {
        Seq::get_tag()
    }
}

impl Core {
    pub fn new() -> Core {
        Core {
            // encoding: Encoding::default (),
            // encoding: cset.encoding,
            styles: CommonStyles::default(),

            tag_handles: [
                (Cow::from("!!"), Cow::from("tag:yaml.org,2002:")),
                (
                    Cow::from("!"),
                    Cow::from("tag:yaml.org,2002:str tag:yaml.org,2002:seq tag:yaml.org,2002:map"),
                ),
                (Cow::from(""), Cow::from("")),
            ],

            // models: None
            mod_map: Map,
            mod_set: Set,
            mod_pairs: Pairs,
            mod_seq: Seq,
            mod_omap: Omap,
            mod_null: Null,
            mod_bool: Bool,
            mod_int: Int,
            mod_float: Float,
            mod_str: Str,
            mod_merge: Merge,
            mod_value: Value,
            mod_yaml: Yaml,
            mod_timestamp: Timestamp,
            mod_binary: Binary,
            mod_literal: Literal,
            mod_incognitum: Incognitum,
        }
    }
}

impl Clone for Core {
    fn clone(&self) -> Core {
        Core::new()
    }
}
