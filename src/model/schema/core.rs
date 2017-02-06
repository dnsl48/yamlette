use model::schema::Schema;

use model::{ Factory, Model };
use model::style::CommonStyles;

use model::yaml::map::MapFactory;
use model::yaml::omap::OmapFactory;
use model::yaml::pairs::PairsFactory;
use model::yaml::set::SetFactory;
use model::yaml::seq::SeqFactory;
use model::yaml::bool::BoolFactory;
use model::yaml::null::NullFactory;
use model::yaml::int::IntFactory;
use model::yaml::float::FloatFactory;
use model::yaml::str::StrFactory;
use model::yaml::value::ValueFactory;
use model::yaml::merge::MergeFactory;
use model::yaml::yaml::YamlFactory;
use model::yaml::timestamp::TimestampFactory;
use model::yaml::binary::BinaryFactory;

use model::yamlette::literal::LiteralFactory;
use model::yamlette::incognitum::IncognitumFactory;

use txt::{ CharSet, Encoding, Twine };

use std::default::Default;



pub struct Core {
    encoding: Encoding,
    styles: CommonStyles,
    tag_handles: [(Twine, Twine); 3],
    models: Option<[Box<Model> ; 17]>
}



impl Schema for Core {
    fn init (&mut self, cset: &CharSet) {
        self.encoding = cset.encoding;

        self.models = Some ([
            MapFactory.build_model (cset),
            SetFactory.build_model (cset),
            PairsFactory.build_model (cset),

            SeqFactory.build_model (cset),
            OmapFactory.build_model (cset),

            NullFactory.build_model (cset),
            BoolFactory.build_model (cset),

            IntFactory.build_model (cset),
            FloatFactory.build_model (cset),

            StrFactory.build_model (cset),

            MergeFactory.build_model (cset),
            ValueFactory.build_model (cset),
            YamlFactory.build_model (cset),

            TimestampFactory.build_model (cset),
            BinaryFactory.build_model (cset),

            LiteralFactory.build_model (cset),
            IncognitumFactory.build_model (cset)
        ]);
    }

    fn get_encoding (&self) -> Encoding { self.encoding }

    fn get_common_styles (&self) -> CommonStyles { self.styles }

    fn get_yaml_version (&self) -> (u8, u8) { (1, 2) }

    fn get_tag_handles (&self) -> &[(Twine, Twine)] { &self.tag_handles }

    fn look_up_model<'a, 'b> (&'a self, tag: &'b str) -> Option<&'a Model> {
        if let Some (ref models) = self.models {
            for model in models {
                if model.get_tag ().as_ref () == tag { return Some (model.as_ref ()) }
            }
        }

        None
    }

    fn look_up_model_callback (&self, predicate: &mut (FnMut (&Model) -> bool)) -> Option<&Model> {
        if let Some (ref models) = self.models {
            for model in models {
                let model = model.as_ref ();
                if predicate (model) { return Some (model) }
            }
        }

        None
    }

    fn get_metamodel (&self) -> Option<&Model> {
        if let Some (ref models) = self.models {
            Some (models[16].as_ref ())
        } else { None }
    }
}



impl Core {
    pub fn new () -> Core { Core {
        encoding: Encoding::default (),

        styles: CommonStyles::default (),

        tag_handles: [
            (Twine::from ("!!"), Twine::from ("tag:yaml.org,2002:")),
            (Twine::from ("!"), Twine::from ("tag:yaml.org,2002:str tag:yaml.org,2002:seq tag:yaml.org,2002:map")),
            (Twine::from (""), Twine::from (""))
        ],

        models: None
    } }
}
