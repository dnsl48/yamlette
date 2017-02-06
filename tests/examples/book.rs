macro_rules! sage {
    ($src:expr) => {{
        let cset = get_charset_utf8 ();

        let (sender, receiver) = channel ();
        let mut reader = Reader::new (Tokenizer::new (cset.clone ()), sender);

        let sage = Sage::new (cset, receiver, get_schema ());

        reader.read (SliceReader::new ($src.as_bytes ())).unwrap_or_else (|err| { assert! (false, format! ("Unexpected result: {}, :{}", err, err.position)); });

        sage
    }}
}


macro_rules! sage11 {
    ($src:expr) => {{
        let cset = get_charset_utf8 ();

        let (sender, receiver) = channel ();
        let mut reader = Reader::new (Tokenizer::new (cset.clone ()), sender);

        let sage = Sage::new (cset, receiver, get_schema ()).and_then (|sg| {
            sg.set_yaml_version (YamlVersion::V1x1).ok ();
            Ok ( sg )
        });

        reader.read (SliceReader::new ($src.as_bytes ())).unwrap_or_else (|err| { assert! (false, format! ("Unexpected result: {}, :{}", err, err.position)); });

        sage
    }}
}


macro_rules! book {
    ($sage:expr) => {{
        let s = $sage.unwrap ();

        let mut book = Book::new ();

        for idea in &*s {
            book.stamp (idea);
        }

        book
    }}
}



#[cfg (all (test, not (feature = "dev")))]
mod stable {
    extern crate skimmer;
    extern crate yamlette;

    use self::skimmer::reader::SliceReader;
    use self::yamlette::book::Book;
    use self::yamlette::model::schema::core::Core;
    use self::yamlette::reader::Reader;
    use self::yamlette::sage::{ Sage, YamlVersion };
    use self::yamlette::tokenizer::Tokenizer;
    use self::yamlette::txt::get_charset_utf8;
    use std::sync::mpsc::channel;



    fn get_schema () -> Core { Core::new () }



    #[test]
    fn extra_00 () {
        let src = r"Mark McGwire";

        let sage = sage! (src);
        let book = book! (sage);

        yamlette_reckon! (book ; book ; [[
            (name:&str)
        ]]);

        assert_eq! (name, Some("Mark McGwire"));
    }


    #[test]
    fn extra_01 () {
        let src =
r"- [a, b, c]
- &b [d, e, f]
- [g, *b, i]
- *b
";

        let sage = sage! (src);
        let book = book! (sage);

        yamlette_reckon! (book ; book ; [[
        [
            [ (a:&str), (b:&str), (c:&str) ],
            [ (d:&str), (e:&str), (f:&str) ],
            [
                (g:&str),
                [
                    (hd:&str),
                    (he:&str),
                    (hf:&str)
                ],
                (i:&str)
            ],
            [ (bd:&str), (be:&str), (bf:&str) ]
        ]
        ]]);

        assert_eq! (a, Some ("a"));
        assert_eq! (b, Some ("b"));
        assert_eq! (c, Some ("c"));
        assert_eq! (d, Some ("d"));
        assert_eq! (e, Some ("e"));
        assert_eq! (f, Some ("f"));
        assert_eq! (g, Some ("g"));
        assert_eq! (hd, Some ("d"));
        assert_eq! (he, Some ("e"));
        assert_eq! (hf, Some ("f"));
        assert_eq! (i, Some ("i"));
        assert_eq! (bd, Some ("d"));
        assert_eq! (be, Some ("e"));
        assert_eq! (bf, Some ("f"));
    }


    #[test]
    fn extra_02 () {
        let src = r"{ unu: 1, du: 2.0, tri: 3, kvar: 4.0, kvin: five, ses: six }";

        let sage = sage! (src);
        let book = book! (sage);

        use std::collections::HashMap;
        yamlette_reckon! (book; book; [[
            (dict si:HashMap<String, i32>, sf:HashMap<String, f32>, ss:HashMap<&str, &str>)
        ]]);


        assert! (si.is_some ());
        let si = si.unwrap ();
        assert_eq! (si.len (), 4);
        assert_eq! (si.get ("unu"),  Some (&1));
        assert_eq! (si.get ("du"),   Some (&2));
        assert_eq! (si.get ("tri"),  Some (&3));
        assert_eq! (si.get ("kvar"), Some (&4));


        assert! (sf.is_some ());
        let sf = sf.unwrap ();
        assert_eq! (sf.len (), 4);
        assert_eq! (sf.get ("unu"),  Some (&1.0));
        assert_eq! (sf.get ("du"),   Some (&2.0));
        assert_eq! (sf.get ("tri"),  Some (&3.0));
        assert_eq! (sf.get ("kvar"), Some (&4.0));


        assert! (ss.is_some ());
        let ss = ss.unwrap ();
        assert_eq! (ss.len (), 2);
        assert_eq! (ss.get ("kvin"), Some (&"five"));
        assert_eq! (ss.get ("ses"), Some (&"six"));
    }


    #[test]
    fn extra_03 () {
        let src = r"{ unu: 1, du: 2.0, tri: 3, kvar: 4.0, kvin: five, ses: six }";

        let sage = sage! (src);
        let book = book! (sage);

        use std::collections::HashMap;
        yamlette_reckon! (book; book; [[
            (dict si:HashMap<String, i32>, sf:HashMap<String, f32>, ss:HashMap<&str, &str>)
        ]]);


        assert! (si.is_some ());
        let si = si.unwrap ();
        assert_eq! (si.len (), 4);
        assert_eq! (si.get ("unu"),  Some (&1));
        assert_eq! (si.get ("du"),   Some (&2));
        assert_eq! (si.get ("tri"),  Some (&3));
        assert_eq! (si.get ("kvar"), Some (&4));


        assert! (sf.is_some ());
        let sf = sf.unwrap ();
        assert_eq! (sf.len (), 4);
        assert_eq! (sf.get ("unu"),  Some (&1.0));
        assert_eq! (sf.get ("du"),   Some (&2.0));
        assert_eq! (sf.get ("tri"),  Some (&3.0));
        assert_eq! (sf.get ("kvar"), Some (&4.0));


        assert! (ss.is_some ());
        let ss = ss.unwrap ();
        assert_eq! (ss.len (), 2);
        assert_eq! (ss.get ("kvin"), Some (&"five"));
        assert_eq! (ss.get ("ses"), Some (&"six"));
    }


    #[test]
    fn extra_04 () {
        let src =
r"- Mark McGwire
- [unu, [dua, tri], kvar]
- []
- Sammy Sosa
key: val
[kvin, ses]: { sep: oka, nau: dek }
iso: morph";

        let sage = sage! (src);
        let book = book! (sage);

        yamlette_reckon! (book ; &book; [[
            [
                (mark:&str),
                [
                    (unu:&str),
                    [
                        (dua:&str),
                        (tri:&str)
                    ],
                    (kvar:&str)
                ],
                [],
                (sosa:&str)
            ],
            {
                (key:&str) > (val:&str),
                [(kvin:&str), (ses:&str)] > { (sep:&str) > (oka:&str), (nau:&str) > (dek:&str) },
                (iso:&str) > (morph:&str)
            }
        ]]);

        assert_eq! (mark, Some ("Mark McGwire"));
        assert_eq! (sosa, Some ("Sammy Sosa"));
        assert_eq! (unu, Some ("unu"));
        assert_eq! (kvar, Some ("kvar"));
        assert_eq! (dua, Some ("dua"));
        assert_eq! (tri, Some ("tri"));
        assert_eq! (key, Some ("key"));
        assert_eq! (val, Some ("val"));
        assert_eq! (iso, Some ("iso"));
        assert_eq! (morph, Some ("morph"));

        assert_eq! (kvin, Some ("kvin"));
        assert_eq! (ses, Some ("ses"));
        assert_eq! (sep, Some ("sep"));
        assert_eq! (oka, Some ("oka"));
        assert_eq! (nau, Some ("nau"));
        assert_eq! (dek, Some ("dek"));



        yamlette_reckon! (book ; &book ; [[
            (),
            {
                "iso" => (morph:&str),
                "key" => (val:&str)
            }
        ]]);

        assert_eq! (val, Some ("val"));
        assert_eq! (morph, Some ("morph"));


        let mut morph: &str = "";
        let mut val: Option<&str> = None;

        yamlette_reckon! (book ; &book ; [[
            (),
            (call &mut |ptr| {

                ptr.into_map ().and_then (|mut ptr| {
                    loop {
                        if ptr == "key" {
                            ptr.next_sibling ().and_then (|ptr| {
                                val = ptr.into ();
                                Some ( () )
                            });
                        } else if ptr == "iso" {
                            ptr.next_sibling ().and_then (|ptr| {
                                if let Some (v) = ptr.into () { morph = v; }
                                Some ( () )
                            });
                        }

                        ptr = if let Some (p) = if let Some (p) = ptr.next_sibling () { p.next_sibling () } else { None } { p } else { break };
                    }

                    Some ( () )
                });
            })
        ]]);

        assert_eq! (morph, "morph");
        assert_eq! (val, Some ("val"));
    }


    #[test]
    fn extra_05 () {
        let src =
r"- unu
- 2
- tri
- 4";

        let sage = sage! (src);
        let book = book! (sage);

        yamlette_reckon! (book; book; [[
            (list strs:Vec<&str>, nums:Vec<i32>)
        ]]);

        assert! (strs.is_some ());
        assert! (nums.is_some ());

        assert_eq! (strs.as_ref ().unwrap ().len (), 2);
        assert_eq! (nums.as_ref ().unwrap ().len (), 2);

        assert_eq! (strs.as_ref ().unwrap ()[0], "unu");
        assert_eq! (strs.as_ref ().unwrap ()[1], "tri");

        assert_eq! (nums.as_ref ().unwrap ()[0], 2);
        assert_eq! (nums.as_ref ().unwrap ()[1], 4);
    }


    #[test]
    fn extra_06 () {
        let src = r"{ unu: 1, dua: 2.0, tri: 3, kvar: 4.0, kvin: yoba, ses: roba }";

        let sage = sage! (src);
        let book = book! (sage);

        use std::collections::HashMap;
        yamlette_reckon! (book; book; [[
            (dict si:HashMap<String, i32>, sf:HashMap<String, f32>, ss:HashMap<&str, &str>)
        ]]);

        assert! (si.is_some ());
        assert_eq! (si.as_ref ().unwrap ().len (), 4);
        assert! (si.as_ref ().unwrap ().contains_key ("unu"));
        assert_eq! (si.as_ref ().unwrap ()["unu"], 1);
        assert! (si.as_ref ().unwrap ().contains_key ("dua"));
        assert_eq! (si.as_ref ().unwrap ()["dua"], 2);
        assert! (si.as_ref ().unwrap ().contains_key ("tri"));
        assert_eq! (si.as_ref ().unwrap ()["tri"], 3);
        assert! (si.as_ref ().unwrap ().contains_key ("kvar"));
        assert_eq! (si.as_ref ().unwrap ()["kvar"], 4);

        assert! (sf.is_some ());
        assert_eq! (sf.as_ref ().unwrap ().len (), 4);
        assert! (sf.as_ref ().unwrap ().contains_key ("unu"));
        assert_eq! (sf.as_ref ().unwrap ()["unu"], 1.0);
        assert! (sf.as_ref ().unwrap ().contains_key ("dua"));
        assert_eq! (sf.as_ref ().unwrap ()["dua"], 2.0);
        assert! (sf.as_ref ().unwrap ().contains_key ("tri"));
        assert_eq! (sf.as_ref ().unwrap ()["tri"], 3.0);
        assert! (sf.as_ref ().unwrap ().contains_key ("kvar"));
        assert_eq! (sf.as_ref ().unwrap ()["kvar"], 4.0);

        assert! (ss.is_some ());
        assert_eq! (ss.as_ref ().unwrap ().len (), 2);
        assert! (ss.as_ref ().unwrap ().contains_key ("kvin"));
        assert! (ss.as_ref ().unwrap ().contains_key ("ses"));
        assert_eq! (ss.as_ref ().unwrap ()["kvin"], "yoba");
        assert_eq! (ss.as_ref ().unwrap ()["ses"], "roba");
    }


    #[test]
    fn extra_07 () {
        let src = r"&a [a, b, c, *a]";

        let sage = sage! (src);
        let book = book! (sage);

        yamlette_reckon! (book; book; [[
            [(a:&str), (b:&str), (c:&str), [(a2:&str), (b2:&str)]]
        ]]);

        assert_eq! (a, Some ("a"));
        assert_eq! (b, Some ("b"));
        assert_eq! (c, Some ("c"));
        assert_eq! (a2, Some ("a"));
        assert_eq! (b2, Some ("b"));
    }


    #[test]
    fn extra_08 () {
        let src =
r"scene:
 - name: Living Room TV Time
   entities:
       light.living_room_front_left:
           state: on
           transition: 10
           brightness: 1
       light.living_room_front_right: 
           state: on
           transition: 10
           brightness: 1";

        let sage = sage11! (src);
        let book = book! (sage);

        yamlette_reckon! (book; book; [[
            {
                "scene" => [ {
                    "name" => (name:&str),
                    "entities" => (call &mut |ptr| {
                        ptr.unalias ().into_map ().and_then (|ptr| {
                            assert! (ptr == "light.living_room_front_left");

                            yamlette_reckon! ( ptr ; ptr.next_sibling () ; {
                                (state:bool),
                                (transition:u8),
                                (brightness:u8)
                            } );

                            assert_eq! (state, Some (true));
                            assert_eq! (transition, Some (10u8));
                            assert_eq! (brightness, Some (1u8));


                            let ptr = ptr.next_sibling ();
                            assert! (ptr.is_some ());

                            ptr.unwrap ().next_sibling ().and_then (|ptr| {
                                assert! (ptr == "light.living_room_front_right");

                                yamlette_reckon! ( ptr ; ptr.next_sibling () ; {
                                    (state:bool),
                                    (transition:u8),
                                    (brightness:u8)
                                } );

                                assert_eq! (state, Some (true));
                                assert_eq! (transition, Some (10u8));
                                assert_eq! (brightness, Some (1u8));

                                Some ( () )
                            })
                        });
                    })
                } ]
            }
        ]]);

        assert_eq! (name, Some ("Living Room TV Time"));
    }


    #[test]
    fn extra_09 () {
        use std::collections::HashMap;
        use self::yamlette::book::extractor::pointer::Pointer;
        use self::yamlette::book::extractor::traits::Dict;

        #[derive (PartialEq, Eq, Debug)]
        struct State {
            pub state: bool,
            pub transition: u8,
            pub brightness: u8
        }

        impl State {
            pub fn new (state: bool, transition: u8, brightness: u8) -> State {
                State {
                    state: state,
                    transition: transition,
                    brightness: brightness
                }
            }
        }

        struct StateMap(HashMap<String, State>);

        impl<'a> Dict<'a> for StateMap {
            fn dict_new () -> Self { StateMap(HashMap::new ()) }

            fn dict_reserve (&mut self, size: usize) { self.0.reserve (size) }

            fn dict_update (&mut self, key: Pointer<'a>, val: Pointer<'a>) {
                if let Some (key) = key.into::<String> () {
                    yamlette_reckon! ( ptr ; Some (val) ; {
                        (state:bool),
                        (transition:u8),
                        (brightness:u8)
                    } );

                    let val = State {
                        state: if let Some (s) = state { s } else { false },
                        transition: if let Some (t) = transition { t } else { 0u8 },
                        brightness: if let Some (b) = brightness { b } else { 0u8 }
                    };

                    self.0.insert (key, val);
                }
            }
        }


        let src =
r"scene:
 - name: Living Room TV Time
   entities:
       light.living_room_front_left:
           state: on
           transition: 10
           brightness: 1
       light.living_room_front_right: 
           state: on
           transition: 10
           brightness: 1
       light.living_room_back_right: 
           state: on
           transition: 10
           brightness: 1
       light.living_room_back_left: 
           state: on
           transition: 4
           brightness: 1
       light.living_room_slider: 
           state: on
           transition: 4
           brightness: 1
       light.living_room_couch_1:
           state: on
           transition: 40
           brightness: 255
       light.living_room_couch_2:
           state: on
           transition: 40
           brightness: 255
       light.couch_tv_light:
           state: on
           transition: 40
           brightness: 100";


        let sage = sage11! (src);
        let book = book! (sage);

        yamlette_reckon! (book; book; [[{ "scene" => [ {
            "name" => (name:&str),
            "entities" => (dict states:StateMap)
        } ] }]]);

        assert_eq! (name, Some ("Living Room TV Time"));
        assert! (states.is_some ());

        let states: HashMap<String, State> = states.unwrap ().0;

        assert_eq! (states.len (), 8);

        assert! (states.contains_key ("light.living_room_front_left"));
        assert! (states.contains_key ("light.living_room_front_right"));
        assert! (states.contains_key ("light.living_room_back_right"));
        assert! (states.contains_key ("light.living_room_back_left"));
        assert! (states.contains_key ("light.living_room_slider"));
        assert! (states.contains_key ("light.living_room_couch_1"));
        assert! (states.contains_key ("light.living_room_couch_2"));
        assert! (states.contains_key ("light.couch_tv_light"));

        assert_eq! (states["light.living_room_front_left"], State::new (true, 10, 1));
        assert_eq! (states["light.living_room_front_right"], State::new (true, 10, 1));
        assert_eq! (states["light.living_room_back_right"], State::new (true, 10, 1));
        assert_eq! (states["light.living_room_back_left"], State::new (true, 4, 1));
        assert_eq! (states["light.living_room_slider"], State::new (true, 4, 1));
        assert_eq! (states["light.living_room_couch_1"], State::new (true, 40, 255));
        assert_eq! (states["light.living_room_couch_2"], State::new (true, 40, 255));
        assert_eq! (states["light.couch_tv_light"], State::new (true, 40, 100));
    }


    #[test]
    fn extra_10 () {
        use std::collections::HashMap;
        use self::yamlette::book::extractor::pointer::Pointer;
        use self::yamlette::book::extractor::traits::FromPointer;

        #[derive (PartialEq, Eq, Debug)]
        struct State {
            pub state: bool,
            pub transition: u8,
            pub brightness: u8
        }

        impl State {
            pub fn new (state: bool, transition: u8, brightness: u8) -> State {
                State {
                    state: state,
                    transition: transition,
                    brightness: brightness
                }
            }
        }

        impl<'a> FromPointer<'a> for State {
            fn from_pointer (pointer: Pointer<'a>) -> Option<Self> {
                yamlette_reckon! ( ptr ; Some (pointer) ; {
                    (state:bool),
                    (transition:u8),
                    (brightness:u8)
                } );

                Some (State {
                    state: if let Some (s) = state { s } else { false },
                    transition: if let Some (t) = transition { t } else { 0u8 },
                    brightness: if let Some (b) = brightness { b } else { 0u8 }
                })
            }
        }

/*
        struct StateMap(HashMap<String, State>);

        impl<'a> Dict<'a> for StateMap {
            fn dict_new () -> Self { StateMap(HashMap::new ()) }

            fn dict_reserve (&mut self, size: usize) { self.0.reserve (size) }

            fn dict_update (&mut self, key: Pointer<'a>, val: Pointer<'a>) {
                if let Some (key) = key.into::<String> () {
                    yamlette_reckon! ( ptr ; Some (val) ; {
                        (state:bool),
                        (transition:u8),
                        (brightness:u8)
                    } );

                    let val = State {
                        state: if let Some (s) = state { s } else { false },
                        transition: if let Some (t) = transition { t } else { 0u8 },
                        brightness: if let Some (b) = brightness { b } else { 0u8 }
                    };

                    self.0.insert (key, val);
                }
            }
        }
*/


        let src =
r"scene:
 - name: Living Room TV Time
   entities:
       light.living_room_front_left:
           state: on
           transition: 10
           brightness: 1
       light.living_room_front_right: 
           state: on
           transition: 10
           brightness: 1
       light.living_room_back_right: 
           state: on
           transition: 10
           brightness: 1
       light.living_room_back_left: 
           state: on
           transition: 4
           brightness: 1
       light.living_room_slider: 
           state: on
           transition: 4
           brightness: 1
       light.living_room_couch_1:
           state: on
           transition: 40
           brightness: 255
       light.living_room_couch_2:
           state: on
           transition: 40
           brightness: 255
       light.couch_tv_light:
           state: on
           transition: 40
           brightness: 100";


        let sage = sage11! (src);
        let book = book! (sage);

        yamlette_reckon! (book; book; [[{ "scene" => [ {
            "name" => (name:&str),
            "entities" => (dict states:HashMap<String, State>)
        } ] }]]);

        assert_eq! (name, Some ("Living Room TV Time"));
        assert! (states.is_some ());

        let states: HashMap<String, State> = states.unwrap ();

        assert_eq! (states.len (), 8);

        assert! (states.contains_key ("light.living_room_front_left"));
        assert! (states.contains_key ("light.living_room_front_right"));
        assert! (states.contains_key ("light.living_room_back_right"));
        assert! (states.contains_key ("light.living_room_back_left"));
        assert! (states.contains_key ("light.living_room_slider"));
        assert! (states.contains_key ("light.living_room_couch_1"));
        assert! (states.contains_key ("light.living_room_couch_2"));
        assert! (states.contains_key ("light.couch_tv_light"));

        assert_eq! (states["light.living_room_front_left"], State::new (true, 10, 1));
        assert_eq! (states["light.living_room_front_right"], State::new (true, 10, 1));
        assert_eq! (states["light.living_room_back_right"], State::new (true, 10, 1));
        assert_eq! (states["light.living_room_back_left"], State::new (true, 4, 1));
        assert_eq! (states["light.living_room_slider"], State::new (true, 4, 1));
        assert_eq! (states["light.living_room_couch_1"], State::new (true, 40, 255));
        assert_eq! (states["light.living_room_couch_2"], State::new (true, 40, 255));
        assert_eq! (states["light.couch_tv_light"], State::new (true, 40, 100));
    }


    #[test]
    fn extra_11 () {
        let src = r"{ name: Mark McGwire }";

        let sage = sage! (src);
        let book = book! (sage);

        yamlette_reckon! (book ; book ; [[
            {(name:&str)}
        ]]);

        assert_eq! (name, Some("Mark McGwire"));


        yamlette_reckon! (book ; book ; [[
            { "name" => (name:&str) }
        ]]);

        assert_eq! (name, Some("Mark McGwire"));
    }


    #[test]
    fn extra_12 () {
        use self::yamlette::book::extractor::{ Pointer, FromPointer };

        #[derive (PartialEq, Eq)]
        enum Key {
            UNU,
            DUA,
            TRI
        }

        impl<'a> FromPointer<'a> for Key {
            fn from_pointer (pointer: Pointer<'a>) -> Option<Self> {
                if let Some (v) = pointer.into::<&'a str> () {
                    match v {
                        "unu" => Some (Key::UNU),
                        "dua" => Some (Key::DUA),
                        "tri" => Some (Key::TRI),
                        _ => None
                    }
                } else { None }
            }
        }


        let src = r"{ unu: 1, dua: 2, tri: 3, kvar: 4 }";

        let sage = sage! (src);
        let book = book! (sage);

        yamlette_reckon! ( book ; book ; [[ {
            Key::UNU => (unu:u8),
            Key::DUA => (dua:u8),
            Key::TRI => (tri:u8)
        } ]] );

        assert_eq! (unu, Some (1u8));
        assert_eq! (dua, Some (2u8));
        assert_eq! (tri, Some (3u8));
    }
}

/*
extern crate skimmer;
extern crate yamlette;


use self::skimmer::reader::SliceReader;
use self::yamlette::book::Book;
use self::yamlette::model::schema::core::get_schema;
use self::yamlette::reader::Reader;
use self::yamlette::sage::{ Sage, YamlVersion };
use self::yamlette::tokenizer::Tokenizer;
use self::yamlette::txt::get_charset_utf8;
use std::sync::mpsc::channel;



*/
