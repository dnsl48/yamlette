extern crate skimmer;

use model::yaml;
use model::yamlette::literal::LiteralValue;
use model::{Model, Renderer, Rope, TaggedValue};

use std::any::Any;
use std::borrow::Cow;
use std::env;
use std::iter::Iterator;

pub static TAG: &'static str = "tag:yamlette.org,1:env";

#[derive(Clone, Copy)]
pub struct Env;

impl Env {
    pub fn get_tag() -> Cow<'static, str> {
        Cow::from(TAG)
    }

    #[inline(always)]
    fn bytes_to_string(&self, bytes: &[u8]) -> Result<String, ()> {
        match String::from_utf8(Vec::from(bytes)) {
            Ok(s) => Ok(s),
            _ => Err(()),
        }
    }

    #[inline(always)]
    fn get_env_var(&self, key: &str) -> Result<String, ()> {
        match env::var(key) {
            Ok(value) => Ok(value),
            _ => Err(())
        }
    }
}

impl Model for Env {
    fn get_tag(&self) -> Cow<'static, str> {
        Self::get_tag()
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

    fn decode(&self, _: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let key = self.bytes_to_string(value)?;
        let val = self.get_env_var(&key)?;

        Ok(TaggedValue::from(LiteralValue::from(val)))
    }

    fn encode (
        &self,
        renderer: &Renderer,
        value: TaggedValue,
        tags: &mut Iterator<Item=&(Cow<'static, str>, Cow<'static, str>)>
    ) -> Result<Rope, TaggedValue> {
        match <TaggedValue as Into<Result<LiteralValue, TaggedValue>>>::into(value) {
            Err(value) => Err(value),
            Ok(value) => {
                match self.get_env_var(value.as_ref()) {
                    Err (_) => Err (TaggedValue::from(value)),
                    Ok (env_value) => {
                        let s = yaml::str::Str;
                        let strval = yaml::str::StrValue::from(env_value);
                        let tagval = TaggedValue::from(strval);
                        s.encode(renderer, tagval, tags)
                    }
                }
            }
        }
    }
}


#[cfg(all(test, not(feature = "dev")))]
mod tests {
    use super::*;

    use model::{Renderer, Tagged};
    use model::yamlette::literal;

    use std::env;
    use std::iter;

    #[test]
    fn tag() {
        let env = Env;

        assert_eq!(env.get_tag().as_ref(), TAG);
    }

    #[test]
    fn encode() {
        let renderer = Renderer;
        let env = Env;

        if let Ok(_) = env.encode(
            &renderer,
            TaggedValue::from(LiteralValue::from("YEMLETTE_TESTS_MODEL_ENV_KEY_01")),
            &mut iter::empty()
        ) {
            assert!(false)
        }

        env::set_var("YEMLETTE_TESTS_MODEL_ENV_KEY_01", "Value 01");
        env::set_var("YEMLETTE_TESTS_MODEL_ENV_KEY_02", "Value 02");

        if let Ok(rope) = env.encode(
            &renderer,
            TaggedValue::from(LiteralValue::from("YEMLETTE_TESTS_MODEL_ENV_KEY_01")),
            &mut iter::empty()
        ) {
            let vec = rope.render(&renderer);
            assert_eq!(vec, "Value 01".to_string().into_bytes().to_vec());
        } else {
            assert!(false)
        }

        if let Ok(rope) = env.encode(
            &renderer,
            TaggedValue::from(LiteralValue::from("YEMLETTE_TESTS_MODEL_ENV_KEY_02")),
            &mut iter::empty()
        ) {
            let vec = rope.render(&renderer);
            assert_eq!(vec, "Value 02".to_string().into_bytes().to_vec());
        } else {
            assert!(false)
        }
    }

    #[test]
    fn decode() {
        let env = Env;

        if let Ok(_) = env.decode(false, "YEMLETTE_TESTS_MODEL_ENV_KEY_03".as_bytes()) {
            assert!(false)
        }

        env::set_var("YEMLETTE_TESTS_MODEL_ENV_KEY_03", "Value 03");

        if let Ok(value) = env.decode(false, "YEMLETTE_TESTS_MODEL_ENV_KEY_03".as_bytes()) {
            assert_eq!(value.get_tag(), Cow::from(literal::TAG));

            let val: &str = value
                    .as_any()
                    .downcast_ref::<LiteralValue>()
                    .unwrap()
                    .as_ref();

            assert_eq!(val, "Value 03");
        } else {
            assert!(false)
        }
    }
}
