extern crate skimmer;

use self::skimmer::symbol::{ CopySymbol, Combo };


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

use txt::{ CharSet, Encoding, Twine };

use std::default::Default;



pub struct Core<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    encoding: Encoding,
    styles: CommonStyles,
    tag_handles: [(Twine, Twine); 3],
    mod_map: Map<Char, DoubleChar>,
    mod_set: Set<Char, DoubleChar>,
    mod_pairs: Pairs<Char, DoubleChar>,
    mod_seq: Seq<Char, DoubleChar>,
    mod_omap: Omap<Char, DoubleChar>,
    mod_null: Null<Char, DoubleChar>,
    mod_bool: Bool<Char, DoubleChar>,
    mod_int: Int<Char, DoubleChar>,
    mod_float: Float<Char, DoubleChar>,
    mod_str: Str<Char, DoubleChar>,
    mod_merge: Merge<Char, DoubleChar>,
    mod_value: Value<Char, DoubleChar>,
    mod_yaml: Yaml<Char, DoubleChar>,
    mod_timestamp: Timestamp<Char, DoubleChar>,
    mod_binary: Binary<Char, DoubleChar>,
    mod_literal: Literal<Char, DoubleChar>,
    mod_incognitum: Incognitum<Char, DoubleChar>
}



impl<Char, DoubleChar> Schema<Char, DoubleChar> for Core<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    fn init (&mut self, _: &CharSet<Char, DoubleChar>) { }

    fn get_model_literal (&self) -> &Literal<Char, DoubleChar> { &self.mod_literal }

    fn get_model_null (&self) -> &Null<Char, DoubleChar> { &self.mod_null }

    fn get_encoding (&self) -> Encoding { self.encoding }

    fn get_common_styles (&self) -> CommonStyles { self.styles }

    fn get_yaml_version (&self) -> (u8, u8) { (1, 2) }

    fn get_tag_handles (&self) -> &[(Twine, Twine)] { &self.tag_handles }

    #[inline (always)]
    fn get_tag_model_map (&self) -> &Twine { <Map<Char, DoubleChar>>::get_tag () }

    #[inline (always)]
    fn get_tag_model_seq (&self) -> &Twine { <Seq<Char, DoubleChar>>::get_tag () }


    fn look_up_model<'a, 'b> (&'a self, tag: &'b str) -> Option<&'a Model<Char=Char, DoubleChar=DoubleChar>> {
             if tag == <Map<Char, DoubleChar>>::get_tag () { Some (&self.mod_map) }
        else if tag == <Set<Char, DoubleChar>>::get_tag () { Some (&self.mod_set) }
        else if tag == <Pairs<Char, DoubleChar>>::get_tag () { Some (&self.mod_pairs) }
        else if tag == <Seq<Char, DoubleChar>>::get_tag () { Some (&self.mod_seq) }
        else if tag == <Omap<Char, DoubleChar>>::get_tag () { Some (&self.mod_omap) }
        else if tag == <Null<Char, DoubleChar>>::get_tag () { Some (&self.mod_null) }
        else if tag == <Bool<Char, DoubleChar>>::get_tag () { Some (&self.mod_bool) }
        else if tag == <Int<Char, DoubleChar>>::get_tag () { Some (&self.mod_int) }
        else if tag == <Float<Char, DoubleChar>>::get_tag () { Some (&self.mod_float) }
        else if tag == <Str<Char, DoubleChar>>::get_tag () { Some (&self.mod_str) }
        else if tag == <Merge<Char, DoubleChar>>::get_tag () { Some (&self.mod_merge) }
        else if tag == <Value<Char, DoubleChar>>::get_tag () { Some (&self.mod_value) }
        else if tag == <Yaml<Char, DoubleChar>>::get_tag () { Some (&self.mod_yaml) }
        else if tag == <Timestamp<Char, DoubleChar>>::get_tag () { Some (&self.mod_timestamp) }
        else if tag == <Binary<Char, DoubleChar>>::get_tag () { Some (&self.mod_binary) }
        else if tag == <Literal<Char, DoubleChar>>::get_tag () { Some (&self.mod_literal) }
        else if tag == <Incognitum<Char, DoubleChar>>::get_tag () { Some (&self.mod_incognitum) }
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


    fn look_up_model_callback (&self, predicate: &mut (FnMut (&Model<Char=Char, DoubleChar=DoubleChar>) -> bool)) -> Option<&Model<Char=Char, DoubleChar=DoubleChar>> {
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

    fn get_metamodel (&self) -> Option<&Model<Char=Char, DoubleChar=DoubleChar>> { Some (&self.mod_incognitum) }
}



impl<Char, DoubleChar> Core<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    pub fn new (cset: &CharSet<Char, DoubleChar>) -> Core<Char, DoubleChar> { Core {
        // encoding: Encoding::default (),
        encoding: cset.encoding,

        styles: CommonStyles::default (),

        tag_handles: [
            (Twine::from ("!!"), Twine::from ("tag:yaml.org,2002:")),
            (Twine::from ("!"), Twine::from ("tag:yaml.org,2002:str tag:yaml.org,2002:seq tag:yaml.org,2002:map")),
            (Twine::from (""), Twine::from (""))
        ],

        // models: None
        mod_map: Map::new (&cset),
        mod_set: Set::new (&cset),
        mod_pairs: Pairs::new (&cset),
        mod_seq: Seq::new (&cset),
        mod_omap: Omap::new (&cset),
        mod_null: Null::new (&cset),
        mod_bool: Bool::new (&cset),
        mod_int: Int::new (&cset),
        mod_float: Float::new (&cset),
        mod_str: Str::new (&cset),
        mod_merge: Merge::new (&cset),
        mod_value: Value::new (&cset),
        mod_yaml: Yaml::new (&cset),
        mod_timestamp: Timestamp::new (&cset),
        mod_binary: Binary::new (&cset),
        mod_literal: Literal::new (&cset),
        mod_incognitum: Incognitum::new (&cset)
    } }
}
