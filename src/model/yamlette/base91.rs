extern crate skimmer;
extern crate base91;

use model::yaml::binary::BinaryValue;
use model::{model_issue_rope, EncodedString, Model, Node, Renderer, Rope, TaggedValue};

use std::any::Any;
use std::borrow::Cow;
use std::iter::Iterator;

pub static TAG: &'static str = "tag:yamlette.org,1:base91";

#[derive(Clone, Copy)]
pub struct Base91;

impl Base91 {
    pub fn get_tag() -> Cow<'static, str> {
        Cow::from(TAG)
    }
}

impl Model for Base91 {
    fn get_tag(&self) -> Cow<'static, str> {
        Cow::from(TAG)
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut Any {
        self
    }

    fn is_decodable(&self) -> bool {
        true
    }

    fn is_encodable(&self) -> bool {
        true
    }

    fn encode(
        &self,
        _renderer: &Renderer,
        value: TaggedValue,
        tags: &mut Iterator<Item = &(Cow<'static, str>, Cow<'static, str>)>,
    ) -> Result<Rope, TaggedValue> {
        let mut value: BinaryValue =
            match <TaggedValue as Into<Result<BinaryValue, TaggedValue>>>::into(value) {
                Ok(value) => value,
                Err(value) => return Err(value),
            };

        let issue_tag = value.issue_tag();
        let alias = value.take_alias();
        let value = value.to_vec();
        let production = base91::slice_encode(&value[..]);
        let node = Node::String(EncodedString::from(production));

        Ok(model_issue_rope(self, node, issue_tag, alias, tags))
    }

    fn decode(&self, _explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let slice = if value.len() > 1 && value[0] == b'\'' && value[value.len() - 1] == b'\'' {
            &value[1 .. value.len() - 2]
        } else {
            value
        };

        let production = base91::slice_decode(slice);

        Ok(TaggedValue::from(BinaryValue::from(production)))
    }
}


#[cfg(all(test, not(feature = "dev")))]
mod tests {
    use super::*;

    use model::{Renderer, Tagged};

    use std::iter;

    #[test]
    fn tag() {
        let model = Base91;

        assert_eq!(model.get_tag(), TAG);
    }

    #[test]
    fn encode() {
        let renderer = Renderer; // ::new (&get_charset_utf8 ());
        let model = Base91; // ::new (&get_charset_utf8 ());

        let pairs = pairs();

        for idx in 0..pairs.len() {
            let p = pairs[idx];

            if let Ok(rope) = model.encode(
                &renderer,
                TaggedValue::from(BinaryValue::from(p.0.to_string().into_bytes())),
                &mut iter::empty(),
            ) {
                // println!("rope: {:?}", rope);
                // let encoded = rope.render (&renderer);
                // let expected = p.1.as_bytes ();

                let encoded = unsafe { String::from_utf8_unchecked(rope.render(&renderer)) };
                let expected = p.1;

                assert_eq!(encoded, expected);
            } else {
                assert!(false, "Unexpected result")
            }
        }
    }

    #[test]
    fn decode() {
        let model = Base91; // ::new (&get_charset_utf8 ());

        let pairs = pairs();

        for idx in 0..pairs.len() {
            let p = pairs[idx];

            if let Ok(tagged) = model.decode(true, &p.1.to_string().into_bytes()) {
                // assert_eq!(tagged.get_tag(), Cow::from(TAG));

                let expected = p.0.as_bytes();

                if let Some(model) = tagged.as_any().downcast_ref::<BinaryValue>() {
                    let val: &Vec<u8> = model.as_ref();
                    assert_eq!(*val, expected);
                } else {
                    assert!(false)
                }
            } else {
                assert!(false, "Unexpected result")
            }
        }

        if let Ok(tagged) = model.decode(true, &"".to_string().into_bytes()) {
            // assert_eq!(tagged.get_tag(), Cow::from(TAG));

            let vec: &Vec<u8> = tagged
                .as_any()
                .downcast_ref::<BinaryValue>()
                .unwrap()
                .as_ref();
            assert_eq!(0, vec.len());
        } else {
            assert!(false, "Unexpected result")
        }

        // TODO: warning?
        // let decoded = bin.decode (&"=".to_string ().into_bytes ());
        // assert! (decoded.is_err ());

        // let decoded = bin.decode (&"c3VyZS4".to_string ().into_bytes ());
        // assert! (decoded.is_err ());
    }

    fn pairs() -> [(&'static str, &'static str); 11] {
        [
            ("sure.", r##"f8zg5gA"##),
            ("asure.", r##"v2e3f,BB"##),
            ("easure.", r##"_D7gt@"@C"##),
            ("leasure.", r##"XPH<2]6eOI"##),
            ("pleasure.", r##""imfN[;N<RX"##),
            ("any carnal pleas", r##"po9K%*jNn$m!mBMm_DNK"##),
            ("any carnal pleasu", r##"po9K%*jNn$m!mBMm_D7gd"##),
            ("any carnal pleasur", r##"po9K%*jNn$m!mBMm_D7gt@A"##),
            ("any carnal pleasure", r##"po9K%*jNn$m!mBMm_D7gt@UC"##),
            ("any carnal pleasure.", r##"po9K%*jNn$m!mBMm_D7gt@"@C"##),
            ("Man is distinguished, not only by his reason, but by this singular passion from other animals, which is a lust of the mind, that by a perseverance of delight in the continued and indefatigable generation of knowledge, exceeds the short vehemence of any carnal pleasure.", r##"8D$J`/wC4!c.hQ;mT8,<p/&Y/H@$]xlL3oDg<W.0$FW6GFMo_D8=8=}AMf][|LfVd/<P1o/1Z2(.I+LR6tQQ0o1a/2/WtN3$3t[x&k)zgZ5=p;LRe.{B[pqa(I.WRT%yxtB92oZB,2,Wzv;Rr#N.cju"JFXiZBMf<WMC&$@+e95p)z01_*UCxT0t88Km=UQJ;WH[#F]4pE>i3o(g7=$e7R2u>xjLxoefB.6Yy#~uex8jEU_1e,MIr%!&=EHnLBn2h>M+;Rl3qxcL5)Wfc,HT$F]4pEsofrFK;W&eh#=#},|iKB,2,W]@fVlx,a<m;i=CY<=Hb%}+},F"##)
        ]
    }
}
