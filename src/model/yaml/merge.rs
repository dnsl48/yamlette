extern crate skimmer;

use self::skimmer::symbol::{ Combo, CopySymbol };


use txt::{ CharSet, Encoding, Twine };

use model::{ EncodedString, Model, Node, Rope, Renderer, Tagged, TaggedValue };

use std::any::Any;
use std::iter::Iterator;



pub const TAG: &'static str = "tag:yaml.org,2002:merge";
static TWINE_TAG: Twine = Twine::Static (TAG);




pub struct Merge<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    encoding: Encoding,

    marker: DoubleChar,

    s_quote: Char,
    d_quote: Char,

    line_feed: Char,
    carriage_return: Char,
    space: Char,
    tab_h: Char
}



impl<Char, DoubleChar> Merge<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    pub fn get_tag () -> &'static Twine { &TWINE_TAG }

    pub fn new (cset: &CharSet<Char, DoubleChar>) -> Merge<Char, DoubleChar> {
        Merge {
            encoding: cset.encoding,

            marker: DoubleChar::combine (&[ cset.less_than, cset.less_than ]),

            s_quote: cset.apostrophe,
            d_quote: cset.quotation,

            line_feed: cset.line_feed,
            carriage_return: cset.carriage_return,
            space: cset.space,
            tab_h: cset.tab_h
        }
    }
}



impl<Char, DoubleChar> Model for Merge<Char, DoubleChar>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static
{
    type Char = Char;
    type DoubleChar = DoubleChar;

    fn get_tag (&self) -> &Twine { Self::get_tag () }

    fn as_any (&self) -> &Any { self }

    fn as_mut_any (&mut self) -> &mut Any { self }

    fn get_encoding (&self) -> Encoding { self.encoding }


    fn is_decodable (&self) -> bool { true }

    fn is_encodable (&self) -> bool { true }


    fn encode (&self, _renderer: &Renderer<Char, DoubleChar>, value: TaggedValue, _tags: &mut Iterator<Item=&(Twine, Twine)>) -> Result<Rope, TaggedValue> {
        match <TaggedValue as Into<Result<MergeValue, TaggedValue>>>::into (value) {
            Ok (_) => Ok ( Rope::from (Node::String (EncodedString::from (self.marker.new_vec ()))) ),
            Err (value) => Err (value)
        }
    }


    fn decode (&self, explicit: bool, value: &[u8]) -> Result<TaggedValue, ()> {
        let mut ptr = 0;

        let vlen = value.len ();

        let mut quote_state = 0;

        if explicit {
            if self.s_quote.contained_at (value, 0) {
                ptr += self.s_quote.len ();
                quote_state = 1;
            } else if self.d_quote.contained_at (value, 0) {
                ptr += self.d_quote.len ();
                quote_state = 2;
            }
        }

        if self.marker.contained_at (value, ptr) {
            ptr += self.marker.len ();

            if quote_state > 0 {
                if quote_state == 1 {
                    if self.s_quote.contained_at (value, ptr) {
                        ptr += self.s_quote.len ();
                    } else {
                        return Err ( () )
                    }
                } else if quote_state == 2 {
                    if self.d_quote.contained_at (value, ptr) {
                        ptr += self.d_quote.len ();
                    } else {
                        return Err ( () )
                    }
                }
            }

            loop {
                if ptr >= vlen { break; }

                if self.space.contained_at (value, ptr) {
                    ptr += self.space.len ();
                    continue;
                }

                if self.tab_h.contained_at (value, ptr) {
                    ptr += self.tab_h.len ();
                    continue;
                }

                if self.line_feed.contained_at (value, ptr) {
                    ptr += self.line_feed.len ();
                    continue;
                }

                if self.carriage_return.contained_at (value, ptr) {
                    ptr += self.carriage_return.len ();
                    continue;
                }

                return Err ( () )
            }

            Ok ( TaggedValue::from (MergeValue) )
        }
        else { Err ( () ) }
    }
}




#[derive (Debug)]
pub struct MergeValue;



impl Tagged for MergeValue {
    fn get_tag (&self) -> &Twine { &TWINE_TAG }

    fn as_any (&self) -> &Any { self as &Any }

    fn as_mut_any (&mut self) -> &mut Any { self as &mut Any }
}



impl AsRef<str> for MergeValue {
    fn as_ref (&self) -> &'static str { "<<" }
}



/*
pub struct MergeFactory;

impl Factory for MergeFactory {
    fn get_tag (&self) -> &Twine { &TWINE_TAG }

    fn build_model<Char, DoubleChar> (&self, cset: &CharSet<Char, DoubleChar>) -> Box<Model<Char=Char, DoubleChar=DoubleChar>>
      where
        Char: CopySymbol + 'static,
        DoubleChar: CopySymbol + Combo + 'static
    { Box::new (Merge::new (cset)) }
}
*/



#[cfg (all (test, not (feature = "dev")))]
mod tests {
    use super::*;

    use model::{ Tagged, Renderer };
    use txt::get_charset_utf8;

    use std::iter;



    #[test]
    fn tag () {
        let merge = Merge::new (&get_charset_utf8 ());

        assert_eq! (merge.get_tag (), TAG);
    }


    #[test]
    fn encode () {
        let renderer = Renderer::new (&get_charset_utf8 ());
        let merge = Merge::new (&get_charset_utf8 ());

        let rope = merge.encode (&renderer, TaggedValue::from (MergeValue), &mut iter::empty ()).ok ().unwrap ();
        assert_eq! (rope.render (&renderer), vec! [b'<', b'<']);
    }


    #[test]
    fn decode () {
        let merge = Merge::new (&get_charset_utf8 ());

        if let Ok (tagged) = merge.decode (true, "<<".as_bytes ()) {
            assert_eq! (tagged.get_tag (), &TWINE_TAG);

            if let None = tagged.as_any ().downcast_ref::<MergeValue> () { assert! (false) }
        } else { assert! (false) }

        assert! (merge.decode (true, "<<~".as_bytes ()).is_err ());
    }
}
