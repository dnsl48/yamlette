extern crate skimmer;


use model::schema::Schema;

use model::{ Model, TaggedValue };
use model::style::CommonStyles;

use model::yaml::map::Map;
use model::yaml::omap::Omap;
use model::yaml::pairs::Pairs;
use model::yaml::set::Set;
use model::yaml::seq::Seq;
use model::yaml::bool::Bool;
use model::yaml::null::Null;
use model::yaml::int::Int;
use model::yaml::float::Float;
use model::yaml::str::Str;
use model::yaml::value::Value;
use model::yaml::merge::Merge;
use model::yaml::yaml::Yaml;
use model::yaml::timestamp::Timestamp;
use model::yaml::binary::Binary;

use model::yamlette::literal::Literal;
use model::yamlette::incognitum::Incognitum;

use txt::Twine;

use std::default::Default;



pub struct Core {
    styles: CommonStyles,
    tag_handles: [(Twine, Twine); 3],

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
    mod_incognitum: Incognitum
}



impl Schema for Core {
    #[inline (always)]
    fn get_model_literal (&self) -> Literal { self.mod_literal }

    #[inline (always)]
    fn get_model_null (&self) -> Null { self.mod_null }

    #[inline (always)]
    fn get_common_styles (&self) -> CommonStyles { self.styles }

    #[inline (always)]
    fn get_yaml_version (&self) -> (u8, u8) { (1, 2) }

    fn get_tag_handles (&self) -> &[(Twine, Twine)] { &self.tag_handles }

    #[inline (always)]
    fn get_tag_model_map (&self) -> &Twine { Map::get_tag () }

    #[inline (always)]
    fn get_tag_model_seq (&self) -> &Twine { Seq::get_tag () }


    fn look_up_model<'a, 'b> (&'a self, tag: &'b str) -> Option<&'a Model> {
             if tag == Map::get_tag () { Some (&self.mod_map) }
        else if tag == Set::get_tag () { Some (&self.mod_set) }
        else if tag == Pairs::get_tag () { Some (&self.mod_pairs) }
        else if tag == Seq::get_tag () { Some (&self.mod_seq) }
        else if tag == Omap::get_tag () { Some (&self.mod_omap) }
        else if tag == Null::get_tag () { Some (&self.mod_null) }
        else if tag == Bool::get_tag () { Some (&self.mod_bool) }
        else if tag == Int::get_tag () { Some (&self.mod_int) }
        else if tag == Float::get_tag () { Some (&self.mod_float) }
        else if tag == Str::get_tag () { Some (&self.mod_str) }
        else if tag == Merge::get_tag () { Some (&self.mod_merge) }
        else if tag == Value::get_tag () { Some (&self.mod_value) }
        else if tag == Yaml::get_tag () { Some (&self.mod_yaml) }
        else if tag == Timestamp::get_tag () { Some (&self.mod_timestamp) }
        else if tag == Binary::get_tag () { Some (&self.mod_binary) }
        else if tag == Literal::get_tag () { Some (&self.mod_literal) }
        else if tag == Incognitum::get_tag () { Some (&self.mod_incognitum) }
        else { None }
    }


    fn try_decodable_models (&self, value: &[u8]) -> Option<TaggedValue> {
             if let Ok (value) = self.mod_null.decode (false, value) { Some (value) }
        else if let Ok (value) = self.mod_bool.decode (false, value) { Some (value) }
        else if let Ok (value) = self.mod_int.decode (false, value) { Some (value) }
        else if let Ok (value) = self.mod_float.decode (false, value) { Some (value) }
        else if let Ok (value) = self.mod_str.decode (false, value) { Some (value) }
        else if let Ok (value) = self.mod_incognitum.decode (false, value) { Some (value) }
        else { None }
    }


    fn try_decodable_models_11 (&self, value: &[u8]) -> Option<TaggedValue> {
             if let Ok (value) = self.mod_null.decode11 (false, value) { Some (value) }
        else if let Ok (value) = self.mod_bool.decode11 (false, value) { Some (value) }
        else if let Ok (value) = self.mod_int.decode11 (false, value) { Some (value) }
        else if let Ok (value) = self.mod_float.decode11 (false, value) { Some (value) }
        else if let Ok (value) = self.mod_str.decode11 (false, value) { Some (value) }
        else if let Ok (value) = self.mod_incognitum.decode11 (false, value) { Some (value) }
        else { None }
    }


    fn look_up_model_callback (&self, predicate: &mut (FnMut (&Model) -> bool)) -> Option<&Model> {
             if predicate (&self.mod_map) { Some (&self.mod_map) }
        else if predicate (&self.mod_set) { Some (&self.mod_set) }
        else if predicate (&self.mod_pairs) { Some (&self.mod_pairs) }
        else if predicate (&self.mod_seq) { Some (&self.mod_seq) }
        else if predicate (&self.mod_omap) { Some (&self.mod_omap) }
        else if predicate (&self.mod_null) { Some (&self.mod_null) }
        else if predicate (&self.mod_bool) { Some (&self.mod_bool) }
        else if predicate (&self.mod_int) { Some (&self.mod_int) }
        else if predicate (&self.mod_float) { Some (&self.mod_float) }
        else if predicate (&self.mod_str) { Some (&self.mod_str) }
        else if predicate (&self.mod_merge) { Some (&self.mod_merge) }
        else if predicate (&self.mod_value) { Some (&self.mod_value) }
        else if predicate (&self.mod_yaml) { Some (&self.mod_yaml) }
        else if predicate (&self.mod_timestamp) { Some (&self.mod_timestamp) }
        else if predicate (&self.mod_binary) { Some (&self.mod_binary) }
        else if predicate (&self.mod_literal) { Some (&self.mod_literal) }
        else if predicate (&self.mod_incognitum) { Some (&self.mod_incognitum) }
        else { None }
    }

    fn get_metamodel (&self) -> Option<&Model> { Some (&self.mod_incognitum) }
}



impl Core {
    pub fn new () -> Core { Core {
        // encoding: Encoding::default (),
        // encoding: cset.encoding,

        styles: CommonStyles::default (),

        tag_handles: [
            (Twine::from ("!!"), Twine::from ("tag:yaml.org,2002:")),
            (Twine::from ("!"), Twine::from ("tag:yaml.org,2002:str tag:yaml.org,2002:seq tag:yaml.org,2002:map")),
            (Twine::from (""), Twine::from (""))
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
        mod_incognitum: Incognitum
    } }
}
