macro_rules! sage {
    ($src:expr) => {{
        let cset = get_charset_utf8 ();

        let (sender, receiver) = channel ();
        let mut reader = Reader::new (Tokenizer::new (cset.clone ()));

        let sage = Sage::new (cset, receiver, get_schema ());

        reader.read (
            SliceReader::new ($src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        ).unwrap_or_else (|err| { assert! (false, format! ("Unexpected result: {}, :{}", err, err.position)); });

        sage
    }}
}



macro_rules! sage_with_error {
    ($src:expr, $err_desc:expr, $err_pos:expr) => {{
        let cset = get_charset_utf8 ();

        let (sender, receiver) = channel ();
        let mut reader = Reader::new (Tokenizer::new (cset.clone ()));

        let sage = Sage::new (cset, receiver, get_schema ());

        reader
            .read (
                SliceReader::new ($src.as_bytes ()),
                &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
            )
            .and_then (|_| { assert! (false, format! ("Must be an error in here; {}: {}", $err_pos, $err_desc)); Ok ( () ) })
            .or_else (|err| {
                assert_eq! ($err_desc, err.description);
                assert_eq! ($err_pos, err.position);
                Err (err)
            }).ok ();

        sage
    }}
}



macro_rules! sage_bytes {
    ($src:expr) => {{
        let cset = get_charset_utf8 ();

        let (sender, receiver) = channel ();
        let mut reader = Reader::new (Tokenizer::new (cset.clone ()));

        let sage = Sage::new (cset, receiver, get_schema ());

        reader.read (
            SliceReader::new ($src),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        ).unwrap_or_else (|err| { assert! (false, format! ("Unexpected result: {}, :{}", err, err.position)); });

        sage
    }}
}



macro_rules! read {
    ($sage:expr) => {{
        let s = $sage.unwrap ();

        let mut v: Vec<Idea> = Vec::with_capacity (256);

        for idea in &*s {
            v.push (idea);
        }

        match *v.first ().unwrap () {
            Idea::Dawn => (),
            _ => assert! (false, "The first was not a Dawn")
        };

        match *v.last ().unwrap () {
            Idea::Done => (),
            _ => assert! (false, "The last was not a Done")
        };

        v
    }}
}



macro_rules! read_without_dd_check {
    ($sage:expr) => {{
        let s = $sage.unwrap ();

        let mut v: Vec<Idea> = Vec::with_capacity (256);

        for idea in &*s {
            v.push (idea);
        }

        v
    }}
}



macro_rules! read_with_error {
    ($sage:expr) => {{
        let s = $sage.unwrap ();

        let mut v: Vec<Idea> = Vec::with_capacity (256);

        for idea in &*s {
            v.push (idea);
        }

        v
    }}
}


macro_rules! lookup {
    ($vec:expr, $id:expr) => {{
        let mut element: Option<&Idea> = None;

        for idea in $vec.iter () {
            let id = match *idea {
                Idea::Alias ( id, _ ) => id,
                Idea::Error ( id, _ ) => id,
                Idea::NodeDictionary ( id, _, _, _ ) => id,
                Idea::NodeSequence ( id, _, _ ) => id,
                Idea::NodeScalar ( id, _, _ ) => id,
                Idea::NodeLiteral ( id, _, _ ) => id,
                Idea::NodeMetaMap ( id, _, _, _ ) => id,
                Idea::NodeMetaSeq ( id, _, _ ) => id,
                Idea::ReadError ( id, _, _ ) => id,
                Idea::ReadWarning ( id, _, _ ) => id,

                Idea::Done |
                Idea::Dawn |
                Idea::Dusk => continue
            };

            if id.level != $id.0 || id.parent != $id.1 || id.index != $id.2 { continue }

            element = Some (idea);

            break;
        }

        element
    }}
}


macro_rules! assert_id {
    ( $id:expr, $idt:expr ) => {{
        assert! ($id.level == $idt.0, format! ("Level; actual != expected; {} != {}", $id.level, $idt.0));
        assert! ($id.parent == $idt.1, format! ("Parent; actual != expected; {} != {}", $id.parent, $idt.1));
        assert! ($id.index == $idt.2, format! ("Index; actual != expected; {} != {}", $id.index, $idt.2));
    }}
}


macro_rules! expect {
    ($vec:expr, $index:expr, dawn) => {{
        match $vec.get ($index - 1) {
            None => assert! (false, format! ("Unexisted node at ({})", $index)),
            Some ( idea ) => match *idea {
                Idea::Dawn => assert! (true),
                _ => assert! (false, format! ("Is not a dawn at ({}), it is {:?}", $index, idea))
            }
        }
    }};


    ($vec:expr, $index:expr, dusk) => {{
        match $vec.get ($index - 1) {
            None => assert! (false, format! ("Unexisted node at ({})", $index)),
            Some ( idea ) => match *idea {
                Idea::Dusk => assert! (true),
                _ => assert! (false, format! ("Is not a dusk at ({}), it is {:?}", $index, idea))
            }
        }
    }};



    ($vec:expr, error, $err_desc:expr, $err_pos:expr) => {{
        'top: loop {
            for item in $vec.iter () {
                match *item {
                    Idea::ReadError (_, ref pos, ref msg) => {
                        assert_eq! ($err_desc, <AsRef<str>>::as_ref (msg));
                        assert_eq! ($err_pos, *pos);
                        break 'top;
                    }

                    _ => continue
                };
            }
            assert! (false, "Error has not been found");
        }
    }};



    ($vec:expr, $id:expr, warning, $err_desc:expr, $err_pos:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::ReadWarning (_, ref pos, ref msg) => {
                    assert_eq! ($err_desc, <AsRef<str>>::as_ref (msg));
                    assert_eq! ($err_pos, *pos);
                },
                _ => assert! (false, format! ("Not a warning {:?}", idea))
            }
        };
    }};



    ($vec:expr, $id:expr, seq) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeSequence ( id , ref anchor, ref tag ) if tag == "tag:yaml.org,2002:seq" => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                },
                _ => assert! (false, format! ("Not a sequence {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, seq, !=$tag:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeSequence ( id , ref anchor, ref tag ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! (*tag, $tag);
                },
                _ => assert! (false, format! ("Not a sequence {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, seq, &=$anchor:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeSequence ( id , ref anchor, ref _tag ) => {
                    assert_id! (id, $id);
                    assert_eq! (*anchor, Some(String::from ($anchor)));
                },
                _ => assert! (false, format! ("Not a sequence {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, lazymap, $firstborn:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeDictionary ( id, ref anchor, ref tag, firstborn_option ) if tag == "tag:yaml.org,2002:map" => {
                    assert_id! (id, $id);
                    if firstborn_option.is_none () { assert! (false, "Lazy map must have a firstborn!") };
                    assert_id! (firstborn_option.unwrap (), $firstborn);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                },
                _ => assert! (false, format! ("Not a lazy map {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, lazymap, $firstborn:expr, &=$anchor:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeDictionary ( id, ref anchor, ref tag, firstborn_option ) if tag == "tag:yaml.org,2002:map" => {
                    assert_id! (id, $id);
                    if firstborn_option.is_none () { assert! (false, "Lazy map must have a firstborn!") };
                    assert_id! (firstborn_option.unwrap (), $firstborn);
                    if anchor.is_none () { assert! (false, format! ("Anchor is None, must be {}", $anchor)) };
                    assert_eq! (anchor.as_ref ().unwrap (), $anchor);
                },
                _ => assert! (false, format! ("Not a lazy map {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, map) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeDictionary ( id, ref anchor, ref tag, firstborn_option ) if tag == "tag:yaml.org,2002:map" => {
                    assert_id! (id, $id);
                    if firstborn_option.is_some () { assert! (false, "Non-lazy map cannot have a firstborn!") };
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                },
                _ => assert! (false, format! ("Not a lazy map {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, map, &=$anchor:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeDictionary ( id, ref anchor, ref tag, firstborn_option ) if tag == "tag:yaml.org,2002:map" => {
                    assert_id! (id, $id);
                    if firstborn_option.is_some () { assert! (false, "Non-lazy map cannot have a firstborn!") };
                    if anchor.is_none () { assert! (false, format! ("Anchor is None, must be {}", $anchor)) };
                    assert_eq! (anchor.as_ref ().unwrap (), $anchor);
                },
                _ => assert! (false, format! ("Not a lazy map {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, map, !=$tag:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeDictionary ( id, ref anchor, ref tag, firstborn_option ) => {
                    assert_id! (id, $id);
                    if firstborn_option.is_some () { assert! (false, "Non-lazy map cannot have a firstborn!") };
                    if anchor.is_some () { assert! (false, format! ("Anchor must be None, however it is {:?}", anchor.as_ref ().unwrap ())) };
                    assert_eq! (*tag, $tag);
                },
                _ => assert! (false, format! ("Not a lazy map {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, lazymetamap, $firstborn:expr, $tag:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeMetaMap ( id, ref anchor, ref tag_option, firstborn_option ) => {
                    assert_id! (id, $id);

                    if firstborn_option.is_none () { assert! (false, "Lazy map must have a firstborn!") };
                    assert_id! (firstborn_option.unwrap (), $firstborn);

                    match $tag {
                        Some (atag) => {
                            if tag_option.is_none () { assert! (false, format! ("Tag is not set whereas must be {}", atag)) };
                            assert_eq! (atag, tag_option.as_ref ().unwrap ());
                        }

                        None => {
                            if tag_option.is_some () { assert! (false, format! ("Tag must be None, however it is {}", tag_option.as_ref ().unwrap ())) };
                        }
                    };

                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                },
                _ => assert! (false, format! ("Not a lazy meta map {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, metaseq, $tag:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeMetaSeq ( id , ref anchor_option, ref tag_option ) => {
                    assert_id! (id, $id);
                    if anchor_option.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor_option)) };

                    match $tag {
                        Some (atag) => {
                            if tag_option.is_none () { assert! (false, format! ("Tag is not set whereas must be {}", atag)) };
                            assert_eq! (atag, tag_option.as_ref ().unwrap ());
                        }

                        None => {
                            if tag_option.is_some () { assert! (false, format! ("Tag must be None, however it is {}", tag_option.as_ref ().unwrap ())) };
                        }
                    };
                },
                _ => assert! (false, format! ("Not a meta sequence {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, alias, $value:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::Alias ( id, ref value ) => {
                    assert_id! (id, $id);
                    assert_eq! ($value, value);
                },
                _ => assert! (false, format! ("Not an alias {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, str, $value:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yaml.org,2002:str", tagged_value.get_tag ());
                    assert_eq! ($value, tagged_value.as_any ().downcast_ref::<StrValue> ().unwrap ().as_ref ());
                },
                _ => assert! (false, format! ("Not a str {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, str, $value:expr, &=$anchor:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);

                    match *anchor {
                        None => assert! (false, format! ("Anchor is None")),
                        Some (ref anchor) => assert_eq! ($anchor, anchor)
                    };

                    assert_eq! ("tag:yaml.org,2002:str", tagged_value.get_tag ());
                    assert_eq! ($value, tagged_value.as_any ().downcast_ref::<StrValue> ().unwrap ().as_ref ());
                },
                _ => assert! (false, format! ("Not a str {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, int, $value:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! (tagged_value.get_tag (), "tag:yaml.org,2002:int");
                    assert_eq! (BigInt::from ($value), tagged_value.as_any ().downcast_ref::<IntValue> ().unwrap ().clone ().into ());
                },
                _ => assert! (false, format! ("Not an int {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, timestamp, $year:expr, $month:expr, $day:expr, $hour:expr, $minute:expr, $second:expr, $fraction:expr, $tz_hour:expr, $tz_minute:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yaml.org,2002:timestamp", tagged_value.get_tag ());
                    let ts = tagged_value.as_any ().downcast_ref::<TimestampValue> ().unwrap ();

                    assert_eq! (ts.year, $year);
                    assert_eq! (ts.month, $month);
                    assert_eq! (ts.day, $day);
                    assert_eq! (ts.hour, $hour);
                    assert_eq! (ts.minute, $minute);
                    assert_eq! (ts.second, $second);
                    if let Some (f) = $fraction {
                        assert! (ts.fraction.is_some ());
                        assert_eq! (f.format_as_float (), ts.fraction.clone ().unwrap ().format_as_float ());
                    } else { assert! (ts.fraction.is_none ()) }
                    assert_eq! (ts.tz_hour, $tz_hour);
                    assert_eq! (ts.tz_minute, $tz_minute);
                },
                _ => assert! (false, format! ("Not a timestamp {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, float, $value:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yaml.org,2002:float", tagged_value.get_tag ());
                    assert_eq! (BigFraction::from ($value), tagged_value.as_any ().downcast_ref::<FloatValue> ().unwrap ().clone ().into ());
                },
                _ => assert! (false, format! ("Not a float {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, float::nan) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yaml.org,2002:float", tagged_value.get_tag ());
                    assert! (tagged_value.as_any ().downcast_ref::<FloatValue> ().unwrap ().is_nan ());
                },
                _ => assert! (false, format! ("Not a float {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, float::inf) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yaml.org,2002:float", tagged_value.get_tag ());
                    assert! (tagged_value.as_any ().downcast_ref::<FloatValue> ().unwrap ().is_infinite ());
                    assert! (!tagged_value.as_any ().downcast_ref::<FloatValue> ().unwrap ().is_negative ());
                },
                _ => assert! (false, format! ("Not a float {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, float::neg_inf) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yaml.org,2002:float", tagged_value.get_tag ());
                    assert! (tagged_value.as_any ().downcast_ref::<FloatValue> ().unwrap ().is_infinite ());
                    assert! (tagged_value.as_any ().downcast_ref::<FloatValue> ().unwrap ().is_negative ());
                },
                _ => assert! (false, format! ("Not a float {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, null) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yaml.org,2002:null", tagged_value.get_tag ());
                    assert! (tagged_value.as_any ().downcast_ref::<NullValue> ().is_some ());
                },
                _ => assert! (false, format! ("Not a null {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, merge) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yaml.org,2002:merge", tagged_value.get_tag ());
                    assert! (tagged_value.as_any ().downcast_ref::<MergeValue> ().is_some ());
                },
                _ => assert! (false, format! ("Not a merge {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, value) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yaml.org,2002:value", tagged_value.get_tag ());
                    assert! (tagged_value.as_any ().downcast_ref::<ValueValue> ().is_some ());
                },
                _ => assert! (false, format! ("Not a value {:?}", idea))
            }
        };
    }};



    ($vec:expr, $id:expr, yaml, tag) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yaml.org,2002:yaml", tagged_value.get_tag ());
                    assert! (tagged_value.as_any ().downcast_ref::<YamlValue> ().is_some ());

                    match tagged_value.as_any ().downcast_ref::<YamlValue> ().unwrap () {
                        &YamlValue::Tag => (),
                        ref val @ _ => { assert! (false, format! ("Not a tag {:?}", val)); }
                    };
                },
                _ => assert! (false, format! ("Not a yaml {:?}", idea))
            }
        };
    }};



    ($vec:expr, $id:expr, yaml, anchor) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yaml.org,2002:yaml", tagged_value.get_tag ());
                    assert! (tagged_value.as_any ().downcast_ref::<YamlValue> ().is_some ());

                    match tagged_value.as_any ().downcast_ref::<YamlValue> ().unwrap () {
                        &YamlValue::Anchor => (),
                        ref val @ _ => { assert! (false, format! ("Not an anchor {:?}", val)); }
                    };
                },
                _ => assert! (false, format! ("Not a yaml {:?}", idea))
            }
        };
    }};



    ($vec:expr, $id:expr, yaml, alias) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yaml.org,2002:yaml", tagged_value.get_tag ());
                    assert! (tagged_value.as_any ().downcast_ref::<YamlValue> ().is_some ());

                    match tagged_value.as_any ().downcast_ref::<YamlValue> ().unwrap () {
                        &YamlValue::Alias => (),
                        ref val @ _ => { assert! (false, format! ("Not an alias {:?}", val)); }
                    };
                },
                _ => assert! (false, format! ("Not a yaml {:?}", idea))
            }
        };
    }};



    ($vec:expr, $id:expr, bool, $value:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yaml.org,2002:bool", tagged_value.get_tag ());
                    assert_eq! ($value, *tagged_value.as_any ().downcast_ref::<BoolValue> ().unwrap ().as_ref ());
                },
                _ => assert! (false, format! ("Not a null {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, binary, $value:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yaml.org,2002:binary", tagged_value.get_tag ());
                    assert_eq! ($value, tagged_value.as_any ().downcast_ref::<BinaryValue> ().unwrap ().as_ref ().as_slice ());
                },
                _ => assert! (false, format! ("Not a null {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, incognitum, $tag:expr, $value:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_some () { assert! (false, format! ("Anchor is not None: {:?}", anchor)) };
                    assert_eq! ("tag:yamlette.org,1:incognitum", tagged_value.get_tag ());
                    let value = tagged_value.as_any ().downcast_ref::<IncognitumValue> ().unwrap ();
                    assert_eq! ($tag, value.get_tag ().as_ref ().unwrap ());
                    assert_eq! ($value, value.get_value ());
                },
                _ => assert! (false, format! ("Not a null {:?}", idea))
            }
        };
    }};


    ($vec:expr, $id:expr, incognitum, $tag:expr, $value:expr, &=$anchor:expr) => {{
        let element = lookup! ($vec, $id);

        match element {
            None => assert! (false, format! ("Cannot find element with address {:?}", $id)),
            Some ( idea ) => match *idea {
                Idea::NodeScalar ( id, ref anchor, ref tagged_value ) => {
                    assert_id! (id, $id);
                    if anchor.is_none () { assert! (false, format! ("Anchor is None")) };
                    assert_eq! ($anchor, anchor.as_ref ().unwrap ());
                    assert_eq! ("tag:yamlette.org,1:incognitum", tagged_value.get_tag ());
                    let value = tagged_value.as_any ().downcast_ref::<IncognitumValue> ().unwrap ();
                    assert_eq! ($tag, value.get_tag ().as_ref ().unwrap ());
                    assert_eq! ($value, value.get_value ());
                },
                _ => assert! (false, format! ("Not a null {:?}", idea))
            }
        };
    }};
}




#[cfg (all (test, not (feature = "dev")))]
mod stable {
    extern crate fraction;
    extern crate num;
    extern crate skimmer;
    extern crate yamlette;

    use self::fraction::{ Fraction, BigFraction };
    use self::num::BigInt;

    use self::skimmer::reader::SliceReader;

    use self::yamlette::model::schema::core::Core;
    use self::yamlette::sage::{ Idea, Sage };

    use self::yamlette::reader::Reader;

    use self::yamlette::tokenizer::Tokenizer;
    use self::yamlette::txt::{ Twine, get_charset_utf8 };

    use self::yamlette::model::Tagged;

    use self::yamlette::model::yaml::binary::BinaryValue;
    use self::yamlette::model::yaml::bool::BoolValue;
    use self::yamlette::model::yaml::null::NullValue;
    use self::yamlette::model::yaml::str::StrValue;
    use self::yamlette::model::yaml::int::IntValue;
    use self::yamlette::model::yaml::float::FloatValue;
    use self::yamlette::model::yaml::merge::MergeValue;
    use self::yamlette::model::yaml::timestamp::TimestampValue;
    use self::yamlette::model::yaml::value::ValueValue;
    use self::yamlette::model::yaml::yaml::YamlValue;
    use self::yamlette::model::yamlette::incognitum::IncognitumValue;

    use std::f64;
    use std::sync::mpsc::channel;



    fn get_schema () -> Core { Core::new () }



    #[test]
    fn example_02_01 () {
        let src =
r"- Mark McGwire
- Sammy Sosa
- Ken Griffey";

        let sage = sage! (src);
        let data = read! (sage);

        assert_eq! (7, data.len ());

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), str, r"Mark McGwire");
        expect! (data, (1, 2, 4), str, r"Sammy Sosa");
        expect! (data, (1, 2, 5), str, r"Ken Griffey");

        expect! (data, 6, dusk);
    }



    #[test]
    fn example_02_02 () {
        let src =
r"hr:  65    # Home runs
avg: 0.278 # Batting average
rbi: 147   # Runs Batted In";


        let sage = sage! (src);
        let data = read! (sage);

        assert_eq! (10, data.len ());

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"hr");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), int, 65);
        expect! (data, (1, 3, 5), str, r"avg");
        expect! (data, (1, 3, 6), float, 0.278);
        expect! (data, (1, 3, 7), str, r"rbi");
        expect! (data, (1, 3, 8), int, 147);

        expect! (data, 9, dusk);
    }



    #[test]
    fn example_02_03 () {
        let src =
r"american:
  - Boston Red Sox
  - Detroit Tigers
  - New York Yankees
national:
  - New York Mets
  - Chicago Cubs
  - Atlanta Braves";


        let sage = sage! (src);
        let data = read! (sage);

        assert_eq! (14, data.len ());

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"american");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), str, r"Boston Red Sox");
        expect! (data, (2, 4, 6), str, r"Detroit Tigers");
        expect! (data, (2, 4, 7), str, r"New York Yankees");
        expect! (data, (1, 3, 8), str, r"national");
        expect! (data, (1, 3, 9), seq);
        expect! (data, (2, 9, 10), str, r"New York Mets");
        expect! (data, (2, 9, 11), str, r"Chicago Cubs");
        expect! (data, (2, 9, 12), str, r"Atlanta Braves");

        expect! (data, 13, dusk);
    }



    #[test]
    fn example_02_04 () {
        let src =
r"-
  name: Mark McGwire
  hr:   65
  avg:  0.278
-
  name: Sammy Sosa
  hr:   63
  avg:  0.288";


        let sage = sage! (src);
        let data = read! (sage);

        assert_eq! (18, data.len ());

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), str, r"name");
        expect! (data, (1, 2, 4), lazymap, (1, 2, 3));
        expect! (data, (2, 4, 5), str, r"Mark McGwire");
        expect! (data, (2, 4, 6), str, r"hr");
        expect! (data, (2, 4, 7), int, 65);
        expect! (data, (2, 4, 8), str, r"avg");
        expect! (data, (2, 4, 9), float, 0.278);
        expect! (data, (1, 2, 10), str, r"name");
        expect! (data, (1, 2, 11), lazymap, (1, 2, 10));
        expect! (data, (2, 11, 12), str, r"Sammy Sosa");
        expect! (data, (2, 11, 13), str, r"hr");
        expect! (data, (2, 11, 14), int, 63);
        expect! (data, (2, 11, 15), str, r"avg");
        expect! (data, (2, 11, 16), float, 0.288);

        expect! (data, 17, dusk);
    }



    #[test]
    fn example_02_05 () {
        let src =
r"- [name        , hr, avg  ]
- [Mark McGwire, 65, 0.278]
- [Sammy Sosa  , 63, 0.288]";


        let sage = sage! (src);
        let data = read! (sage);

        assert_eq! (16, data.len ());

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), str, r"name");
        expect! (data, (2, 3, 5), str, r"hr");
        expect! (data, (2, 3, 6), str, r"avg");
        expect! (data, (1, 2, 7), seq);
        expect! (data, (2, 7, 8), str, r"Mark McGwire");
        expect! (data, (2, 7, 9), int, 65);
        expect! (data, (2, 7, 10), float, 0.278);
        expect! (data, (1, 2, 11), seq);
        expect! (data, (2, 11, 12), str, r"Sammy Sosa");
        expect! (data, (2, 11, 13), int, 63);
        expect! (data, (2, 11, 14), float, 0.288);

        expect! (data, 15, dusk);
    }



    #[test]
    fn example_02_06 () {
        let src =
r"Mark McGwire: {hr: 65, avg: 0.278}
Sammy Sosa: {
    hr: 63,
    avg: 0.288
  }";


        let sage = sage! (src);
        let data = read! (sage);

        assert_eq! (16, data.len ());

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"Mark McGwire");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), map);
        expect! (data, (2, 4, 5), str, r"hr");
        expect! (data, (2, 4, 6), int, 65);
        expect! (data, (2, 4, 7), str, r"avg");
        expect! (data, (2, 4, 8), float, 0.278);
        expect! (data, (1, 3, 9), str, r"Sammy Sosa");
        expect! (data, (1, 3, 10), map);
        expect! (data, (2, 10, 11), str, r"hr");
        expect! (data, (2, 10, 12), int, 63);
        expect! (data, (2, 10, 13), str, r"avg");
        expect! (data, (2, 10, 14), float, 0.288);

        expect! (data, 15, dusk);
    }



    #[test]
    fn example_02_07 () {
        let src =
r"# Ranking of 1998 home runs
---
- Mark McGwire
- Sammy Sosa
- Ken Griffey

# Team ranking
---
- Chicago Cubs
- St Louis Cardinals";


        let sage = sage! (src);
        let mut data1 = read! (sage);


        assert_eq! (12, data1.len ());

        let data2 = data1.split_off (6);

        assert_eq! (6, data1.len ());
        assert_eq! (6, data2.len ());

        expect! (data1, 1, dawn);

        expect! (data1, (0, 0, 2), seq);
        expect! (data1, (1, 2, 3), str, r"Mark McGwire");
        expect! (data1, (1, 2, 4), str, r"Sammy Sosa");
        expect! (data1, (1, 2, 5), str, r"Ken Griffey");

        expect! (data1, 6, dusk);


        expect! (data2, 1, dawn);

        expect! (data2, (0, 0, 2), seq);
        expect! (data2, (1, 2, 3), str, r"Chicago Cubs");
        expect! (data2, (1, 2, 4), str, r"St Louis Cardinals");

        expect! (data2, 5, dusk);
    }



    #[test]
    fn example_02_08 () {
        let src =
r"---
time: 20:03:20
player: Sammy Sosa
action: strike (miss)
...
---
time: 20:03:47
player: Sammy Sosa
action: grand slam
...
";


        let sage = sage! (src);
        let mut data1 = read! (sage);

        assert_eq! (19, data1.len ());

        let data2 = data1.split_off (9);

        assert_eq! (9, data1.len ());
        assert_eq! (10, data2.len ());


        expect! (data1, 1, dawn);

        expect! (data1, (0, 0, 2), str, r"time");
        expect! (data1, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data1, (1, 3, 4), str, r"20:03:20");
        expect! (data1, (1, 3, 10), str, r"player");
        expect! (data1, (1, 3, 11), str, r"Sammy Sosa");
        expect! (data1, (1, 3, 12), str, r"action");
        expect! (data1, (1, 3, 13), str, r"strike (miss)");

        expect! (data1, 9, dusk);


        expect! (data2, 1, dawn);

        expect! (data2, (0, 0, 2), str, r"time");
        expect! (data2, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data2, (1, 3, 4), str, r"20:03:47");
        expect! (data2, (1, 3, 10), str, r"player");
        expect! (data2, (1, 3, 11), str, r"Sammy Sosa");
        expect! (data2, (1, 3, 12), str, r"action");
        expect! (data2, (1, 3, 13), str, r"grand slam");

        expect! (data2, 9, dusk);
    }



    #[test]
    fn example_02_09 () {
        let src =
r"---
hr: # 1998 hr ranking
  - Mark McGwire
  - Sammy Sosa
rbi:
  # 1998 rbi ranking
  - Sammy Sosa
  - Ken Griffey";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"hr");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), str, r"Mark McGwire");
        expect! (data, (2, 4, 6), str, r"Sammy Sosa");
        expect! (data, (1, 3, 7), str, r"rbi");
        expect! (data, (1, 3, 8), seq);
        expect! (data, (2, 8, 9), str, r"Sammy Sosa");
        expect! (data, (2, 8, 10), str, r"Ken Griffey");

        expect! (data, 11, dusk);
    }



    #[test]
    fn example_02_10 () {
        let src =
r"---
hr:
  - Mark McGwire
  # Following node labeled SS
  - &SS Sammy Sosa
rbi:
  - *SS # Subsequent occurrence
  - Ken Griffey";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"hr");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), str, r"Mark McGwire");
        expect! (data, (2, 4, 6), str, r"Sammy Sosa", &=r"SS");
        expect! (data, (1, 3, 7), str, r"rbi");
        expect! (data, (1, 3, 8), seq);
        expect! (data, (2, 8, 9), alias, r"SS");
        expect! (data, (2, 8, 10), str, r"Ken Griffey");

        expect! (data, 11, dusk);
    }



    #[test]
    fn example_02_11 () {
        let src =
r"? - Detroit Tigers
  - Chicago cubs
:
  - 2001-07-23

? [ New York Yankees,
    Atlanta Braves ]
: [ 2001-07-02, 2001-08-12,
    2001-08-14 ]";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), map);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), str, r"Detroit Tigers");
        expect! (data, (2, 3, 5), str, r"Chicago cubs");
        expect! (data, (1, 2, 6), seq);
        expect! (data, (2, 6, 7), str, r"2001-07-23");
        expect! (data, (1, 2, 8), seq);
        expect! (data, (2, 8, 9), str, r"New York Yankees");
        expect! (data, (2, 8, 10), str, r"Atlanta Braves");
        expect! (data, (1, 2, 11), seq);
        expect! (data, (2, 11, 12), str, r"2001-07-02");
        expect! (data, (2, 11, 13), str, r"2001-08-12");
        expect! (data, (2, 11, 14), str, r"2001-08-14");

        expect! (data, 15, dusk);
    }



    #[test]
    fn example_02_12 () {
        let src =
r"---
# Products purchased
- item    : Super Hoop
  quantity: 1
- item    : Basketball
  quantity: 4
- item    : Big Shoes
  quantity: 1";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), str, r"item");
        expect! (data, (1, 2, 4), lazymap, (1, 2, 3));
        expect! (data, (2, 4, 5), str, r"Super Hoop");
        expect! (data, (2, 4, 6), str, r"quantity");
        expect! (data, (2, 4, 7), int, 1);
        expect! (data, (1, 2, 8), str, r"item");
        expect! (data, (1, 2, 9), lazymap, (1, 2, 8));
        expect! (data, (2, 9, 10), str, r"Basketball");
        expect! (data, (2, 9, 11), str, r"quantity");
        expect! (data, (2, 9, 12), int, 4);
        expect! (data, (1, 2, 13), str, r"item");
        expect! (data, (1, 2, 14), lazymap, (1, 2, 13));
        expect! (data, (2, 14, 15), str, r"Big Shoes");
        expect! (data, (2, 14, 16), str, r"quantity");
        expect! (data, (2, 14, 17), int, 1);

        expect! (data, 18, dusk);
    }



    #[test]
    fn example_02_13 () {
        let src =
r"# ASCII Art
--- |
  \//||\/||
  // ||  ||__";

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, &format! ("{}{}{}", r"\//||\/||", "\n", r"// ||  ||__"));
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_02_14 () {
        let src =
r"--- >
  Mark McGwire's
  year was crippled
  by a knee injury.";

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, &format! (
            "{}{}{}{}{}",
            r"Mark McGwire's",
            r" ",
            r"year was crippled",
            r" ",
            r"by a knee injury."
        ));
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_02_15 () {
        let src =
r">
 Sammy Sosa completed another
 fine season with great stats.

   63 Home Runs
   0.288 Batting Average

 What a year!";

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, &format! (
            r"{}{}{}{}{}{}{}{}{}{}",
            r"Sammy Sosa completed another",
            r" ",
            r"fine season with great stats.",
            "\n\n",
            r"  63 Home Runs",
            "\n",
            "  0.288 Batting Average",
            "\n",
            "\n",
            "What a year!",
        ));
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_02_16 () {
        let src =
r"name: Mark McGwire
accomplishment: >
  Mark set a major league
  home run record in 1998.
stats: |
  65 Home Runs
  0.278 Batting Average";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"name");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"Mark McGwire");
        expect! (data, (1, 3, 5), str, r"accomplishment");
        expect! (data, (1, 3, 6), str, &format! (
            r"{}{}{}{}",
            r"Mark set a major league",
            r" ",
            r"home run record in 1998.",
            "\n"
        ));
        expect! (data, (1, 3, 11), str, r"stats");
        expect! (data, (1, 3, 12), str, &format! (
            r"{}{}{}",
            r"65 Home Runs",
            "\n",
            r"0.278 Batting Average"
        ));
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_02_17 () {
        let src =
r##"unicode: "Sosa did fine.\u263A"
control: "\b1998\t1999\t2000\n"
hex esc: "\x0d\x0a is \r\n"

single: '"Howdy!" he cried.'
quoted: ' # Not a ''comment''.'
tie-fighter: '|\-*-/|'"##;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"unicode");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, "Sosa did fine.\u{263A}");
        expect! (data, (1, 3, 5), str, r"control");
        expect! (data, (1, 3, 6), str, "\x081998\t1999\t2000\n");
        expect! (data, (1, 3, 7), str, r"hex esc");
        expect! (data, (1, 3, 8), str, "\x0d\x0a is \r\n");
        expect! (data, (1, 3, 9), str, r"single");
        expect! (data, (1, 3, 10), str, r#""Howdy!" he cried."#);
        expect! (data, (1, 3, 11), str, r"quoted");
        expect! (data, (1, 3, 12), str, r#" # Not a 'comment'."#);
        expect! (data, (1, 3, 13), str, r"tie-fighter");
        expect! (data, (1, 3, 14), str, r"|\-*-/|");
        expect! (data, 15, dusk);
    }



    #[test]
    fn example_02_18 () {
        let src =
r#"plain:
  This unquoted scalar
  spans many lines.

quoted: "So does this
  quoted scalar.\n""#;

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"plain");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, &format! (
            r"{}{}{}",
            r"This unquoted scalar",
            r" ",
            r"spans many lines."
        ));
        expect! (data, (1, 3, 8), str, r"quoted");
        expect! (data, (1, 3, 9), str, "So does this quoted scalar.\n");
        expect! (data, 7, dusk);
    }



    #[test]
    fn example_02_19 () {
        let src =
r"canonical: 12345
decimal: +12345
octal: 0o14
hexadecimal: 0xC";

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"canonical");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), int, 12345);
        expect! (data, (1, 3, 5), str, r"decimal");
        expect! (data, (1, 3, 6), int, 12345);
        expect! (data, (1, 3, 7), str, r"octal");
        expect! (data, (1, 3, 8), int, 12);
        expect! (data, (1, 3, 9), str, r"hexadecimal");
        expect! (data, (1, 3, 10), int, 12);
        expect! (data, 11, dusk);
    }



    #[test]
    fn example_02_20 () {
        let src =
r"canonical: 1.23015e+3
exponential: 12.3015e+02
fixed: 1230.15
negative infinity: -.inf
not a number: .NaN";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"canonical");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), float, 1.23015e+3);
        expect! (data, (1, 3, 5), str, r"exponential");
        expect! (data, (1, 3, 6), float, 12.3015e+02);
        expect! (data, (1, 3, 7), str, r"fixed");
        expect! (data, (1, 3, 8), float, 1230.15);
        expect! (data, (1, 3, 9), str, r"negative infinity");
        expect! (data, (1, 3, 10), float, f64::NEG_INFINITY);
        expect! (data, (1, 3, 11), str, r"not a number");
        expect! (data, (1, 3, 12), float::nan);
        expect! (data, 13, dusk);
    }



    #[test]
    fn example_02_21 () {
        let src =
r"null:
booleans: [ true, false ]
string: '012345'";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), null);
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), null);
        expect! (data, (1, 3, 5), str, r"booleans");
        expect! (data, (1, 3, 6), seq);
        expect! (data, (2, 6, 7), bool, true);
        expect! (data, (2, 6, 8), bool, false);
        expect! (data, (1, 3, 9), str, r"string");
        expect! (data, (1, 3, 10), str, r"012345");
        expect! (data, 11, dusk);
    }



    #[test]
    fn example_02_22 () {
        let src =
r"canonical: 2001-12-15T02:59:43.1Z
iso8601: 2001-12-14t21:59:43.10-05:00
spaced: 2001-12-14 21:59:43.10 -5
date: 2002-12-14";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"canonical");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"2001-12-15T02:59:43.1Z");
        expect! (data, (1, 3, 10), str, r"iso8601");
        expect! (data, (1, 3, 11), str, r"2001-12-14t21:59:43.10-05:00");
        expect! (data, (1, 3, 19), str, r"spaced");
        expect! (data, (1, 3, 20), str, r"2001-12-14 21:59:43.10 -5");
        expect! (data, (1, 3, 26), str, r"date");
        expect! (data, (1, 3, 27), str, r"2002-12-14");
        expect! (data, 11, dusk);
    }



    #[test]
    fn example_02_23 () {
        // TODO: the same example with strip chomping
        let src =
r"---
not-date: !!str 2002-04-28

picture: !!binary |
 R0lGODlhDAAMAIQAAP//9/X
 17unp5WZmZgAAAOfn515eXv
 Pz7Y6OjuDg4J+fn5OTk6enp
 56enmleECcgggoBADs=

application specific tag: !something |
 The semantics of the tag
 above may be different for
 different documents.
";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"not-date");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"2002-04-28");
        expect! (data, (1, 3, 5), str, r"picture");

        expect! (data, (1, 3, 6), binary, <Vec<u8>>::from (&b"GIF89a\x0c\x00\x0c\x00\x84\x00\x00\xff\xff\xf7\xf5\xf5\xee\xe9\xe9\xe5fff\x00\x00\x00\xe7\xe7\xe7^^^\xf3\xf3\xed\x8e\x8e\x8e\xe0\xe0\xe0\x9f\x9f\x9f\x93\x93\x93\xa7\xa7\xa7\x9e\x9e\x9ei^\x10' \x82\n\x01\x00;"[..]));

        expect! (data, (1, 3, 15), str, r"application specific tag");

        expect! (data, (1, 3, 16), incognitum, r"!something", &format! (
            r"{}{}{}{}{}{}",
            r"The semantics of the tag",
            "\n",
            r"above may be different for",
            "\n",
            r"different documents.",
            "\n"
        ));

        expect! (data, 9, dusk);
    }



    #[test]
    fn example_02_24 () {
        // TODO: the same example with mapping instead of the sequence
        let src =
r"%TAG ! tag:clarkevans.com,2002:
--- !shape
  # Use the ! handle for presenting
  # tag:clarkevans.com,2002:circle
- !circle
  center: &ORIGIN {x: 73, y: 129}
  radius: 7
- !line
  start: *ORIGIN
  finish: { x: 89, y: 102 }
- !label
  start: *ORIGIN
  color: 0xFFEEBB
  text: Pretty vector drawing.";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), metaseq, Some (r"!shape"));
        expect! (data, (1, 3, 4), str, r"center");
        expect! (data, (1, 3, 5), lazymetamap, (1, 3, 4), Some (r"!circle"));
        expect! (data, (2, 5, 6), map, &=r"ORIGIN");
        expect! (data, (3, 6, 7), str, r"x");
        expect! (data, (3, 6, 8), int, 73);
        expect! (data, (3, 6, 9), str, r"y");
        expect! (data, (3, 6, 10), int, 129);
        expect! (data, (2, 5, 11), str, r"radius");
        expect! (data, (2, 5, 12), int, 7);
        expect! (data, (1, 3, 13), str, r"start");
        expect! (data, (1, 3, 14), lazymetamap, (1, 3, 13), Some (r"!line"));
        expect! (data, (2, 14, 15), alias, r"ORIGIN");
        expect! (data, (2, 14, 16), str, r"finish");
        expect! (data, (2, 14, 17), map);
        expect! (data, (3, 17, 18), str, r"x");
        expect! (data, (3, 17, 19), int, 89);
        expect! (data, (3, 17, 20), str, r"y");
        expect! (data, (3, 17, 21), int, 102);
        expect! (data, (1, 3, 22), str, r"start");
        expect! (data, (1, 3, 23), lazymetamap, (1, 3, 22), Some (r"!label"));
        expect! (data, (2, 23, 24), alias, r"ORIGIN");
        expect! (data, (2, 23, 25), str, r"color");
        expect! (data, (2, 23, 26), int, 0xFFEEBB);
        expect! (data, (2, 23, 27), str, r"text");
        expect! (data, (2, 23, 28), str, r"Pretty vector drawing.");
        expect! (data, 28, dusk);
    }



    #[test]
    fn example_02_25 () {
        let src =
r"# Sets are represented as a
# Mapping where each key is
# associated with a null value
--- !!set
? Mark McGwire
? Sammy Sosa
? Ken Griff";

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), map, !=r"tag:yaml.org,2002:set");
        expect! (data, (1, 2, 3), str, r"Mark McGwire");
        expect! (data, (1, 2, 4), null);
        expect! (data, (1, 2, 5), str, r"Sammy Sosa");
        expect! (data, (1, 2, 6), null);
        expect! (data, (1, 2, 7), str, r"Ken Griff");
        expect! (data, 8, dusk);
    }



    #[test]
    fn example_02_26 () {
        let src =
r"# Ordered maps are represented as
# A sequence of mappings, with
# each mapping having one key
--- !!omap
- Mark McGwire: 65
- Sammy Sosa: 63
- Ken Griffy: 58";

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq, !=r"tag:yaml.org,2002:omap");

        expect! (data, (1, 2, 3), str, r"Mark McGwire");
        expect! (data, (1, 2, 4), lazymap, (1, 2, 3));
        expect! (data, (2, 4, 5), int, 65);

        expect! (data, (1, 2, 6), str, r"Sammy Sosa");
        expect! (data, (1, 2, 7), lazymap, (1, 2, 6));
        expect! (data, (2, 7, 8), int, 63);

        expect! (data, (1, 2, 9), str, r"Ken Griffy");
        expect! (data, (1, 2, 10), lazymap, (1, 2, 9));
        expect! (data, (2, 10, 11), int, 58);

        expect! (data, 12, dusk);
    }



    #[test]
    fn example_02_27 () {
        let src =
r"--- !<tag:clarkevans.com,2002:invoice>
invoice: 34843
date   : 2001-01-23
bill-to: &id001
    given  : Chris
    family : Dumars
    address:
        lines: |
            458 Walkman Dr.
            Suite #292
        city    : Royal Oak
        state   : MI
        postal  : 48046
ship-to: *id001
product:
    - sku         : BL394D
      quantity    : 4
      description : Basketball
      price       : 450.00
    - sku         : BL4438H
      quantity    : 1
      description : Super Hoop
      price       : 2392.00
tax  : 251.42
total: 4443.52
comments:
    Late afternoon is best.
    Backup contact is Nancy
    Billsmer @ 338-4338.";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"invoice");
        expect! (data, (0, 0, 3), lazymetamap, (0, 0, 2), Some (r"!<tag:clarkevans.com,2002:invoice>"));
        expect! (data, (1, 3, 4), int, 34843);
        expect! (data, (1, 3, 5), str, r"date");
        expect! (data, (1, 3, 6), str, r"2001-01-23");
        expect! (data, (1, 3, 7), str, r"bill-to");
        expect! (data, (1, 3, 8), str, r"given");
        expect! (data, (1, 3, 9), lazymap, (1, 3, 8), &=r"id001");
        expect! (data, (2, 9, 10), str, r"Chris");
        expect! (data, (2, 9, 11), str, r"family");
        expect! (data, (2, 9, 12), str, r"Dumars");
        expect! (data, (2, 9, 13), str, r"address");
        expect! (data, (2, 9, 14), str, r"lines");
        expect! (data, (2, 9, 15), lazymap, (2, 9, 14));
        expect! (data, (3, 15, 16), str, "458 Walkman Dr.\nSuite #292\n");
        expect! (data, (3, 15, 21), str, r"city");
        expect! (data, (3, 15, 22), str, r"Royal Oak");
        expect! (data, (3, 15, 23), str, r"state");
        expect! (data, (3, 15, 24), str, r"MI");
        expect! (data, (3, 15, 25), str, r"postal");
        expect! (data, (3, 15, 26), int, 48046);
        expect! (data, (1, 3, 27), str, r"ship-to");
        expect! (data, (1, 3, 28), alias, r"id001");
        expect! (data, (1, 3, 29), str, r"product");
        expect! (data, (1, 3, 30), seq);

        expect! (data, (2, 30, 31), str, r"sku");
        expect! (data, (2, 30, 32), lazymap, (2, 30, 31));
        expect! (data, (3, 32, 33), str, r"BL394D");
        expect! (data, (3, 32, 34), str, r"quantity");
        expect! (data, (3, 32, 35), int, 4);
        expect! (data, (3, 32, 36), str, r"description");
        expect! (data, (3, 32, 37), str, r"Basketball");
        expect! (data, (3, 32, 38), str, r"price");
        expect! (data, (3, 32, 39), float, 450.00);

        expect! (data, (2, 30, 40), str, r"sku");
        expect! (data, (2, 30, 41), lazymap, (2, 30, 40));
        expect! (data, (3, 41, 42), str, r"BL4438H");
        expect! (data, (3, 41, 43), str, r"quantity");
        expect! (data, (3, 41, 44), int, 1);
        expect! (data, (3, 41, 45), str, r"description");
        expect! (data, (3, 41, 46), str, r"Super Hoop");
        expect! (data, (3, 41, 47), str, r"price");
        expect! (data, (3, 41, 48), float, 2392.00);

        expect! (data, (1, 3, 49), str, r"tax");
        expect! (data, (1, 3, 50), float, 251.42);

        expect! (data, (1, 3, 51), str, r"total");
        expect! (data, (1, 3, 52), float, 4443.52);

        expect! (data, (1, 3, 53), str, r"comments");
        expect! (data, (1, 3, 54), str, r"Late afternoon is best. Backup contact is Nancy Billsmer @ 338-4338.");

        expect! (data, 51, dusk);
    }



    #[test]
    fn example_02_28 () {
        let src =
r#"---
Time: 2001-11-23 15:01:42 -5
User: ed
Warning:
  This is an error message
  for the log file
---
Time: 2001-11-23 15:02:31 -5
User: ed
Warning:
  A slightly different error
  message.
---
Date: 2001-11-23 15:03:17 -5
User: ed
Fatal:
  Unknown variable "bar"
Stack:
  - file: TopClass.py
    line: 23
    code: |
      x = MoreObject("345\n")
  - file: MoreClass.py
    line: 58
    code: |-
      foo = bar



"#;


        let sage = sage! (src);
        let mut data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"Time");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"2001-11-23 15:01:42 -5");
        expect! (data, (1, 3, 10), str, r"User");
        expect! (data, (1, 3, 11), str, r"ed");
        expect! (data, (1, 3, 12), str, r"Warning");
        expect! (data, (1, 3, 13), str, r"This is an error message for the log file");

        expect! (data, 9, dusk);


        let mut data = data.split_off (9);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"Time");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"2001-11-23 15:02:31 -5");
        expect! (data, (1, 3, 10), str, r"User");
        expect! (data, (1, 3, 11), str, r"ed");
        expect! (data, (1, 3, 12), str, r"Warning");
        expect! (data, (1, 3, 13), str, r"A slightly different error message.");

        expect! (data, 9, dusk);


        let data = data.split_off (9);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"Date");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"2001-11-23 15:03:17 -5");
        expect! (data, (1, 3, 10), str, r"User");
        expect! (data, (1, 3, 11), str, r"ed");
        expect! (data, (1, 3, 12), str, r"Fatal");
        expect! (data, (1, 3, 13), str, r#"Unknown variable "bar""#);
        expect! (data, (1, 3, 14), str, r"Stack");
        expect! (data, (1, 3, 15), seq);
        expect! (data, (2, 15, 16), str, r"file");
        expect! (data, (2, 15, 17), lazymap, (2, 15, 16));
        expect! (data, (3, 17, 18), str, r"TopClass.py");
        expect! (data, (3, 17, 19), str, r"line");
        expect! (data, (3, 17, 20), int, 23);
        expect! (data, (3, 17, 21), str, r"code");
        expect! (data, (3, 17, 22), str, "x = MoreObject(\"345\\n\")\n");
        expect! (data, (2, 15, 25), str, r"file");
        expect! (data, (2, 15, 26), lazymap, (2, 15, 25));
        expect! (data, (3, 26, 27), str, r"MoreClass.py");
        expect! (data, (3, 26, 28), str, r"line");
        expect! (data, (3, 26, 29), int, 58);
        expect! (data, (3, 26, 30), str, r"code");
        expect! (data, (3, 26, 31), str, r"foo = bar");

        expect! (data, 25, dusk);
    }



    #[test]
    fn example_set_1 () {
        let src =
r"baseball players: !!set
  ? Mark McGwire
  ? Sammy Sosa
  ? Ken Griffey
# Flow style
baseball teams: !!set { Boston Red Sox, Detroit Tigers, New York Yankees }";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"baseball players");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), map, !=r"tag:yaml.org,2002:set");
        expect! (data, (2, 4, 5), str, r"Mark McGwire");
        expect! (data, (2, 4, 6), null);
        expect! (data, (2, 4, 7), str, r"Sammy Sosa");
        expect! (data, (2, 4, 8), null);
        expect! (data, (2, 4, 9), str, r"Ken Griffey");

        expect! (data, (1, 3, 10), str, r"baseball teams");
        expect! (data, (1, 3, 11), map, !=r"tag:yaml.org,2002:set");
        expect! (data, (2, 11, 12), str, r"Boston Red Sox");
        expect! (data, (2, 11, 13), null);
        expect! (data, (2, 11, 14), str, r"Detroit Tigers");
        expect! (data, (2, 11, 15), null);
        expect! (data, (2, 11, 16), str, r"New York Yankees");
        expect! (data, (2, 11, 17), null);

        expect! (data, 18, dusk);
    }



    #[test]
    fn example_value_1 () {
        let src =
r"---     # Old schema
link with:
  - library1.dll
  - library2.dll
---     # New schema
link with:
  - = : library1.dll
    version: 1.2
  - = : library2.dll
    version: 2.3";


        let sage = sage! (src);
        let mut data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"link with");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), str, r"library1.dll");
        expect! (data, (2, 4, 6), str, r"library2.dll");

        expect! (data, 7, dusk);


        let data = data.split_off (7);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"link with");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), seq);

        expect! (data, (2, 4, 5), str, r"=");
        expect! (data, (2, 4, 6), lazymap, (2, 4, 5));
        expect! (data, (3, 6, 7), str, r"library1.dll");
        expect! (data, (3, 6, 8), str, r"version");
        expect! (data, (3, 6, 9), float, 1.2);

        expect! (data, (2, 4, 10), str, r"=");
        expect! (data, (2, 4, 11), lazymap, (2, 4, 10));
        expect! (data, (3, 11, 12), str, r"library2.dll");
        expect! (data, (3, 11, 13), str, r"version");
        expect! (data, (3, 11, 14), float, 2.3);

        expect! (data, 15, dusk);
    }



    #[test]
    fn example_05_01 () {
        let src =
b"\xEF\xBB\xBF# Comment only.";

        let sage = sage_bytes! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, 2, dusk);
    }



    #[test]
    fn example_05_02 () {
        let src =
b"- Invalid use of BOM
\xEF\xBB\xBF
- Inside a document.";

        let sage = sage_bytes! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), str, r"Invalid use of BOM");
        expect! (data, (1, 2, 4), str, r"Inside a document.");
        expect! (data, 5, dusk);
    }



    #[test]
    fn example_05_03 () {
        let src =
r"sequence:
- one
- two
mapping:
  ? sky
  : blue
  sea : green";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"sequence");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), str, r"one");
        expect! (data, (2, 4, 6), str, r"two");
        expect! (data, (1, 3, 7), str, r"mapping");
        expect! (data, (1, 3, 8), map);
        expect! (data, (2, 8, 9), str, r"sky");
        expect! (data, (2, 8, 10), str, r"blue");
        expect! (data, (2, 8, 11), str, r"sea");
        expect! (data, (2, 8, 12), str, r"green");

        expect! (data, 13, dusk);
    }



    #[test]
    fn example_05_03_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "sequence"
  : !!seq [ !!str "one", !!str "two" ],
  ? !!str "mapping"
  : !!map {
    ? !!str "sky" : !!str "blue",
    ? !!str "sea" : !!str "green",
  },
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"sequence");
        expect! (data, (1, 3, 5), seq);
        expect! (data, (2, 5, 6), str, r"one");
        expect! (data, (2, 5, 7), str, r"two");
        expect! (data, (1, 3, 8), str, r"mapping");
        expect! (data, (1, 3, 9), map);
        expect! (data, (2, 9, 10), str, r"sky");
        expect! (data, (2, 9, 11), str, r"blue");
        expect! (data, (2, 9, 12), str, r"sea");
        expect! (data, (2, 9, 13), str, r"green");

        expect! (data, 13, dusk);
    }



    #[test]
    fn example_05_04 () {
        let src =
"sequence: [ one, two, ]
mapping: { sky: blue, sea: green }";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"sequence");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), str, r"one");
        expect! (data, (2, 4, 6), str, r"two");

        expect! (data, (1, 3, 7), str, r"mapping");
        expect! (data, (1, 3, 8), map);
        expect! (data, (2, 8, 9), str, r"sky");
        expect! (data, (2, 8, 10), str, r"blue");
        expect! (data, (2, 8, 11), str, r"sea");
        expect! (data, (2, 8, 12), str, r"green");

        expect! (data, 13, dusk);
    }



    #[test]
    fn example_05_05 () {
        let src = "# Comment only.";

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, 2, dusk);
    }



    #[test]
    fn example_05_06 () {
        let src =
"anchored: !local &anchor value
alias: *anchor";

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"anchored");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), incognitum, r"!local", "value", &=r"anchor");
        expect! (data, (1, 3, 5), str, r"alias");
        expect! (data, (1, 3, 6), alias, r"anchor");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_05_06_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "anchored"
  : !local &A1 "value",
  ? !!str "alias"
  : *A1,
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"anchored");
        expect! (data, (1, 3, 5), incognitum, r"!local", r#""value""#, &=r"A1");
        expect! (data, (1, 3, 6), str, r"alias");
        expect! (data, (1, 3, 7), alias, r"A1");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_05_07 () {
        let src =
"literal: |
  some
  text
folded: >
  some
  text
";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"literal");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, "some\ntext\n");
        expect! (data, (1, 3, 9), str, r"folded");
        expect! (data, (1, 3, 10), str, "some text\n");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_05_07_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "literal"
  : !!str "some\ntext\n",
  ? !!str "folded"
  : !!str "some text\n",
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"literal");
        expect! (data, (1, 3, 5), str, "some\ntext\n");
        expect! (data, (1, 3, 6), str, r"folded");
        expect! (data, (1, 3, 7), str, "some text\n");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_05_08 () {
        let src =
r#"single: 'text'
double: "text""#;

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"single");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"text");
        expect! (data, (1, 3, 5), str, r"double");
        expect! (data, (1, 3, 6), str, r"text");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_05_08_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "single"
  : !!str "text",
  ? !!str "double"
  : !!str "text",
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"single");
        expect! (data, (1, 3, 5), str, r"text");
        expect! (data, (1, 3, 6), str, r"double");
        expect! (data, (1, 3, 7), str, r"text");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_05_09 () {
        let src =
"%YAML 1.2
--- text";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, r"text");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_05_09_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "text""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, r"text");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_05_10 () {
        let src =
"commercial-at: @text
grave-accent: `text";

        let sage = sage_with_error! (src, r"@ character is reserved and may not be used to start a plain scalar", 15);
        let data = read_with_error! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"commercial-at");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, error, r"@ character is reserved and may not be used to start a plain scalar", 15);

        assert_eq! (5, data.len ());
    }



    #[test]
    fn example_05_11 () {
        let src =
"|
  Line break (no glyph)
  Line break (glyphed)
";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, "Line break (no glyph)\nLine break (glyphed)\n");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_05_11_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "line break (no glyph)\n\
      line break (glyphed)\n""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, "line break (no glyph)\nline break (glyphed)\n");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_05_12 () {
        let src =
r#"# Tabs and spaces
quoted: "Quoted 	"
block:	|
  void main() {
  	printf("Hello, world!\n");
  }"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"quoted");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"Quoted 	");
        expect! (data, (1, 3, 5), str, r"block");
        expect! (data, (1, 3, 6), str, "void main() {\n\tprintf(\"Hello, world!\\n\");\n}");
        expect! (data, 7, dusk);
    }



    #[test]
    fn example_05_12_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "quoted"
  : "Quoted \t",
  ? !!str "block"
  : "void main() {\n\
    \tprintf(\"Hello, world!\\n\");\n\
    }\n",
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"quoted");
        expect! (data, (1, 3, 5), str, "Quoted \t");
        expect! (data, (1, 3, 6), str, r"block");
        expect! (data, (1, 3, 7), str, "void main() {\n\tprintf(\"Hello, world!\\n\");\n}\n");
        expect! (data, 7, dusk);
    }



    #[test]
    fn example_05_13 () {
        let src =
r#""Fun with \\
\" \a \b \e \f \
\n \r \t \v \0 \
\  \_ \N \L \P \
\x41 \u0041 \U00000041""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, "Fun with \x5C \x22 \x07 \x08 \x1B \x0C \x0A \x0D \x09 \x0B \x00 \x20 \u{A0} \u{85} \u{2028} \u{2029} A A A");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_05_13_canonical () {
        let src =
r#"%YAML 1.2
---
"Fun with \x5C
\x22 \x07 \x08 \x1B \x0C
\x0A \x0D \x09 \x0B \x00
\x20 \xA0 \x85 \u2028 \u2029
A A A""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, "Fun with \x5C \x22 \x07 \x08 \x1B \x0C \x0A \x0D \x09 \x0B \x00 \x20 \u{A0} \u{85} \u{2028} \u{2029} A A A");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_05_14 () {
        let src =
r#"Bad escapes:
  "\c
  \xq-""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"Bad escapes");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, "\\c \\xq-");
        expect! (data, 5, dusk);
    }



    #[test]
    fn example_06_01 () {
        let src =
r#"  # Leading comment line spaces are
   # neither content nor indentation.
    
Not indented:
 By one space: |
    By four
      spaces
 Flow style: [    # Leading spaces
   By two,        # in flow style
  Also by two,    # are neither
  	Still by two   # content nor
    ]             # indentation."#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"Not indented");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"By one space");
        expect! (data, (1, 3, 5), lazymap, (1, 3, 4));
        expect! (data, (2, 5, 6), str, "By four\n  spaces\n");
        expect! (data, (2, 5, 11), str, r"Flow style");
        expect! (data, (2, 5, 12), seq);
        expect! (data, (3, 12, 13), str, r"By two");
        expect! (data, (3, 12, 14), str, r"Also by two");
        expect! (data, (3, 12, 15), str, r"Still by two");
        expect! (data, 12, dusk);
    }



    #[test]
    fn example_06_01_canonical () {
        let src =
r#"%YAML 1.2
- - -
!!map {
  ? !!str "Not indented"
  : !!map {
      ? !!str "By one space"
      : !!str "By four\n  spaces\n",
      ? !!str "Flow style"
      : !!seq [
          !!str "By two",
          !!str "Also by two",
          !!str "Still by two",
        ]
    }
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), seq);
        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), seq);
        expect! (data, (3, 5, 6), null);
        expect! (data, (0, 0, 7), map);
        expect! (data, (1, 7, 8), str, r"Not indented");
        expect! (data, (1, 7, 9), map);
        expect! (data, (2, 9, 10), str, r"By one space");
        expect! (data, (2, 9, 11), str, "By four\n  spaces\n");
        expect! (data, (2, 9, 12), str, "Flow style");
        expect! (data, (2, 9, 13), seq);
        expect! (data, (3, 13, 14), str, r"By two");
        expect! (data, (3, 13, 15), str, r"Also by two");
        expect! (data, (3, 13, 16), str, r"Still by two");
        expect! (data, 16, dusk);
    }



    #[test]
    fn example_06_02 () {
        let src =
r"? a
: -	b
  -  -	c
     - d
";


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), map);
        expect! (data, (1, 2, 3), str, r"a");
        expect! (data, (1, 2, 4), seq);
        expect! (data, (2, 4, 5), str, r"b");
        expect! (data, (2, 4, 6), seq);
        expect! (data, (3, 6, 7), str, r"c");
        expect! (data, (3, 6, 8), str, r"d");
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_06_02_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "a"
  : !!seq [
    !!str "b",
    !!seq [ !!str "c", !!str "d" ]
  ],
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"a");
        expect! (data, (1, 3, 5), seq);
        expect! (data, (2, 5, 6), str, r"b");
        expect! (data, (2, 5, 7), seq);
        expect! (data, (3, 7, 8), str, r"c");
        expect! (data, (3, 7, 9), str, r"d");
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_06_03 () {
        let src =
r#"- foo:	 bar
- - baz
  -	baz"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), str, r"foo");
        expect! (data, (1, 2, 4), lazymap, (1, 2, 3));
        expect! (data, (2, 4, 5), str, r"bar");
        expect! (data, (1, 2, 6), seq);
        expect! (data, (2, 6, 7), str, r"baz");
        expect! (data, (2, 6, 8), str, r"baz");
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_06_03_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!map {
    ? !!str "foo" : !!str "bar",
  },
  !!seq [ !!str "baz", !!str "baz" ],
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), seq);
        expect! (data, (1, 3, 4), map);
        expect! (data, (2, 4, 5), str, r"foo");
        expect! (data, (2, 4, 6), str, r"bar");
        expect! (data, (1, 3, 7), seq);
        expect! (data, (2, 7, 8), str, r"baz");
        expect! (data, (2, 7, 9), str, r"baz");
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_06_04 () {
        let src =
r#"plain: text
  lines
quoted: "text
  	lines"
block: |
  text
   	lines
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"plain");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"text lines");
        expect! (data, (1, 3, 8), str, r"quoted");
        expect! (data, (1, 3, 9), str, r"text lines");
        expect! (data, (1, 3, 10), str, r"block");
        expect! (data, (1, 3, 11), str, "text\n \tlines\n");
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_06_04_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "plain"
  : !!str "text lines",
  ? !!str "quoted"
  : !!str "text lines",
  ? !!str "block"
  : !!str "text\n 	lines\n",
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"plain");
        expect! (data, (1, 3, 5), str, r"text lines");
        expect! (data, (1, 3, 6), str, r"quoted");
        expect! (data, (1, 3, 7), str, r"text lines");
        expect! (data, (1, 3, 8), str, r"block");
        expect! (data, (1, 3, 9), str, "text\n 	lines\n");
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_06_05 () {
        let src =
r#"Folding:
  "Empty line
   	
  as a line feed"
Chomping: |
  Clipped empty lines
 "#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"Folding");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, "Empty line\nas a line feed");
        expect! (data, (1, 3, 5), str, r"Chomping");
        expect! (data, (1, 3, 6), str, "Clipped empty lines\n");
        expect! (data, 7, dusk);
    }



    #[test]
    fn example_06_05_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "Folding"
  : !!str "Empty line\nas a line feed",
  ? !!str "Chomping"
  : !!str "Clipped empty lines\n",
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"Folding");
        expect! (data, (1, 3, 5), str, "Empty line\nas a line feed");
        expect! (data, (1, 3, 6), str, r"Chomping");
        expect! (data, (1, 3, 7), str, "Clipped empty lines\n");
        expect! (data, 7, dusk);
    }



    #[test]
    fn example_06_06 () {
        let src =
r#">-
  trimmed
  
 

  as
  space"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, "trimmed\n\n\nas space");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_06_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "trimmed\n\n\nas space""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, "trimmed\n\n\nas space");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_07 () {
        let src =
r#">
  foo 
 
  	 bar

  baz
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, "foo \n\n\t bar\n\nbaz\n");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_07_canonical () {
        let src =
r#"%YAML 1.2
--- !!str
"foo \n\n\t bar\n\nbaz\n""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, "foo \n\n\t bar\n\nbaz\n");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_08 () {
        let src =
r#""
  foo 
 
  	 bar

  baz
""#; 


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, " foo\nbar\nbaz ");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_08_canonical () {
        let src =
r#"%YAML 1.2
--- !!str
" foo\nbar\nbaz ""#; 


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, " foo\nbar\nbaz ");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_09 () {
        let src =
r#"key:    # Comment
  value"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"key");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"value");
        expect! (data, 5, dusk);
    }



    #[test]
    fn example_06_09_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "key"
  : !!str "value",
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"key");
        expect! (data, (1, 3, 5), str, r"value");
        expect! (data, 5, dusk);
    }



    #[test]
    fn example_06_10 () {
        let src =
r#"  # Comment
   

"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, 2, dusk);
    }



    #[test]
    fn example_06_11 () {
        let src =
r#"key:    # Comment
        # lines
  value

"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"key");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"value");
        expect! (data, 5, dusk);
    }



    #[test]
    fn example_06_12 () {
        let src =
r#"{ first: Sammy, last: Sosa }:
# Statistics:
  hr:  # Home runs
     65
  avg: # Average
   0.278"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), map);
        expect! (data, (1, 2, 3), str, r"first");
        expect! (data, (1, 2, 4), str, r"Sammy");
        expect! (data, (1, 2, 5), str, r"last");
        expect! (data, (1, 2, 6), str, r"Sosa");
        expect! (data, (0, 0, 7), lazymap, (0, 0, 2));
        expect! (data, (1, 7, 8), str, r"hr");
        expect! (data, (1, 7, 9), lazymap, (1, 7, 8));
        expect! (data, (2, 9, 10), int, 65);
        expect! (data, (2, 9, 11), str, r"avg");
        expect! (data, (2, 9, 12), float, 0.278);
        expect! (data, 13, dusk);
    }



    #[test]
    fn example_06_12_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!map {
    ? !!str "first"
    : !!str "Sammy",
    ? !!str "last"
    : !!str "Sosa",
  }
  : !!map {
    ? !!str "hr"
    : !!int "65",
    ? !!str "avg"
    : !!float "0.278",
  },
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), map);
        expect! (data, (2, 4, 5), str, r"first");
        expect! (data, (2, 4, 6), str, r"Sammy");
        expect! (data, (2, 4, 7), str, r"last");
        expect! (data, (2, 4, 8), str, r"Sosa");

        expect! (data, (1, 3, 9), map);
        expect! (data, (2, 9, 10), str, r"hr");
        expect! (data, (2, 9, 11), int, 65);
        expect! (data, (2, 9, 12), str, r"avg");
        expect! (data, (2, 9, 13), float, 0.278);

        expect! (data, 13, dusk);
    }



    #[test]
    fn example_06_13 () {
        let src =
r#"%FOO  bar baz # Should be ignored
               # with a warning.
--- "foo""#;


        let sage = sage! (src);
        let data = read_without_dd_check! (sage);

        expect! (data, (0, 0, 1), warning, "Unknown directive at the line 0", 0);
        expect! (data, 2, dawn);
        expect! (data, (0, 0, 3), str, r"foo");
        expect! (data, 4, dusk);
    }



    #[test]
    fn example_06_14 () {
        let src =
r#"%YAML 1.3 # Attempt parsing
           # with a warning
---
"foo""#;


        let sage = sage! (src);
        let data = read_without_dd_check! (sage);

        expect! (data, (0, 0, 1), warning, "%YAML minor version is not fully supported", 10);
        expect! (data, 2, dawn);
        expect! (data, (0, 0, 4), str, r"foo");
        expect! (data, 4, dusk);
    }



    #[test]
    fn example_06_15 () {
        let src =
r#"%YAML 1.2
%YAML 1.1
foo"#;


        let sage = sage_with_error! (src, r"The YAML directive must only be given at most once per document", 10);
        let data = read_with_error! (sage);

        expect! (data, error, r"The YAML directive must only be given at most once per document", 10);

        assert_eq! (2, data.len ());
    }



    #[test]
    fn example_06_16 () {
        let src =
r#"%TAG !yaml! tag:yaml.org,2002:
---
!yaml!str "foo""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, r"foo");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_17 () {
        let src =
r#"%TAG ! !foo
%TAG ! !foo
bar"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 4), str, r"bar");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_18 () {
        let src =
r#"# Private
!foo "bar"
...
# Global
%TAG ! tag:example.com,2000:app/
---
!foo "bar""#;


        let sage = sage! (src);
        let mut data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), incognitum, r"!foo", r#""bar""#);
        expect! (data, 3, dusk);


        let data = data.split_off (3);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), incognitum, r"tag:example.com,2000:app/foo", r#""bar""#);
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_18_canonical () {
        let src =
r#"%YAML 1.2
---
!<!foo> "bar"
...
---
!<tag:example.com,2000:app/foo> "bar""#;


        let sage = sage! (src);
        let mut data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), incognitum, r"!foo", r#""bar""#);
        expect! (data, 3, dusk);


        let data = data.split_off (3);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), incognitum, r"tag:example.com,2000:app/foo", r#""bar""#);
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_19 () {
        let src =
r#"%TAG !! tag:example.com,2000:app/
---
!!int 1 - 3 # Interval, not integer"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), incognitum, r"tag:example.com,2000:app/int", r"1 - 3");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_19_canonical () {
        let src =
r#"%YAML 1.2
---
!<tag:example.com,2000:app/int> "1 - 3"
"#;

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), incognitum, r"tag:example.com,2000:app/int", r#""1 - 3""#);
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_20 () {
        let src =
r#"%TAG !e! tag:example.com,2000:app/
---
!e!foo "bar""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), incognitum, r"tag:example.com,2000:app/foo", r#""bar""#);
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_21 () {
        let src =
r#"%TAG !m! !my-
--- # Bulb here
!m!light fluorescent
...
%TAG !m! !my-
--- # Color here
!m!light green"#;


        let sage = sage! (src);
        let mut data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), incognitum, r"!my-light", r#"fluorescent"#);
        expect! (data, 3, dusk);


        let data = data.split_off (3);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), incognitum, r"!my-light", r#"green"#);
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_21_canonical () {
        let src =
r#"%YAML 1.2
---
!<!my-light> "fluorescent"
...
%YAML 1.2
---
!<!my-light> "green""#;


        let sage = sage! (src);
        let mut data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), incognitum, r"!my-light", r#""fluorescent""#);
        expect! (data, 3, dusk);


        let data = data.split_off (3);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), incognitum, r"!my-light", r#""green""#);
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_06_22 () {
        let src =
r#"%TAG !e! tag:example.com,2000:app/
---
- !e!foo "bar""#;

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), seq);
        expect! (data, (1, 3, 4), incognitum, r"tag:example.com,2000:app/foo", r#""bar""#);
        expect! (data, 4, dusk);
    }



    #[test]
    fn example_06_23 () {
        let src =
r#"!!str &a1 "foo":
  !!str bar
&a2 baz : *a1"#;



        let sage = sage! (src);
        let data = read! (sage);


        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"foo", &=r"a1");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"bar");
        expect! (data, (1, 3, 5), str, r"baz", &=r"a2");
        expect! (data, (1, 3, 6), alias, r"a1");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_06_23_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? &B1 !!str "foo"
  : !!str "bar",
  ? !!str "baz"
  : *B1,
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"foo", &=r"B1");
        expect! (data, (1, 3, 5), str, r"bar");
        expect! (data, (1, 3, 6), str, r"baz");
        expect! (data, (1, 3, 7), alias, r"B1");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_06_24 () {
        let src =
r#"!<tag:yaml.org,2002:str> foo :
  !<!bar> baz"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"foo");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), incognitum, r"!bar", r"baz");
        expect! (data, 5, dusk);
    }



    #[test]
    fn example_06_24_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !<tag:yaml.org,2002:str> "foo"
  : !<!bar> "baz",
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), map);

        expect! (data, (1, 3, 4), str, r"foo");
        expect! (data, (1, 3, 5), incognitum, r"!bar", r#""baz""#);

        expect! (data, 5, dusk);
    }



    #[test]
    fn example_06_25 () {
        let src =
r#"- !<!> foo
- !<$:?> bar"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), incognitum, r"!", r"foo");
        expect! (data, (1, 2, 4), incognitum, r"$:?", r"bar");
        expect! (data, 5, dusk);
    }



    #[test]
    fn example_06_26 () {
        let src =
r#"%TAG !e! tag:example.com,2000:app/
---
- !local foo
- !!str bar
- !e!tag%21 baz"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), seq);

        expect! (data, (1, 3, 4), incognitum, r"!local", r"foo");
        expect! (data, (1, 3, 5), str, r"bar");
        expect! (data, (1, 3, 6), incognitum, r"tag:example.com,2000:app/tag%21", r"baz");

        expect! (data, 6, dusk);
    }



    #[test]
    fn example_06_26_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !<!local> "foo",
  !<tag:yaml.org,2002:str> "bar",
  !<tag:example.com,2000:app/tag!> "baz"
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), seq);
        expect! (data, (1, 3, 4), incognitum, r"!local", r#""foo""#);
        expect! (data, (1, 3, 5), str, r"bar");
        expect! (data, (1, 3, 6), incognitum, r"tag:example.com,2000:app/tag!", r#""baz""#);

        expect! (data, 6, dusk);
    }



    #[test]
    fn example_06_27 () {
        let src =
r#"%TAG !e! tag:example,2000:app/
---
- !e! foo
- !h!bar baz"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), seq);

        expect! (data, (1, 3, 4), incognitum, r"tag:example,2000:app/", r"foo");
        expect! (data, (1, 3, 5), incognitum, r"!h!bar", r"baz");

        expect! (data, 5, dusk);
    }



    #[test]
    fn example_06_28 () {
        let src =
r#"# Assuming conventional resolution:
- "12"
- 12
- ! 12"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), str, r"12");
        expect! (data, (1, 2, 4), int, 12);
        expect! (data, (1, 2, 5), str, r"12");
        expect! (data, 6, dusk);
    }



    #[test]
    fn example_06_28_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !<tag:yaml.org,2002:str> "12",
  !<tag:yaml.org,2002:int> "12",
  !<tag:yaml.org,2002:str> "12",
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), seq);
        expect! (data, (1, 3, 4), str, r"12");
        expect! (data, (1, 3, 5), int, 12);
        expect! (data, (1, 3, 6), str, r"12");

        expect! (data, 6, dusk);
    }



    #[test]
    fn example_06_29 () {
        let src =
r#"First occurrence: &anchor Value
Second occurrence: *anchor"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"First occurrence");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"Value", &=r"anchor");
        expect! (data, (1, 3, 5), str, r"Second occurrence");
        expect! (data, (1, 3, 6), alias, r"anchor");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_06_29_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "First occurrence"
  : &A !!str "Value",
  ? !!str "Second occurrence"
  : *A,
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"First occurrence");
        expect! (data, (1, 3, 5), str, r"Value", &=r"A");
        expect! (data, (1, 3, 6), str, r"Second occurrence");
        expect! (data, (1, 3, 7), alias, r"A");

        expect! (data, 7, dusk);
    }



        #[test]
    fn example_07_1 () {
        let src =
r#"First occurrence: &anchor Foo
Second occurrence: *anchor
Override anchor: &anchor Bar
Reuse anchor: *anchor"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"First occurrence");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"Foo", &=r"anchor");
        expect! (data, (1, 3, 5), str, r"Second occurrence");
        expect! (data, (1, 3, 6), alias, r"anchor");
        expect! (data, (1, 3, 7), str, r"Override anchor");
        expect! (data, (1, 3, 8), str, r"Bar", &=r"anchor");
        expect! (data, (1, 3, 9), str, r"Reuse anchor");
        expect! (data, (1, 3, 10), alias, r"anchor");

        expect! (data, 11, dusk);
    }



    #[test]
    fn example_07_01_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "First occurrence"
  : &A !!str "Foo",
  ? !!str "Override anchor"
  : &B !!str "Bar",
  ? !!str "Second occurrence"
  : *A,
  ? !!str "Reuse anchor"
  : *B,
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"First occurrence");
        expect! (data, (1, 3, 5), str, r"Foo", &=r"A");
        expect! (data, (1, 3, 6), str, r"Override anchor");
        expect! (data, (1, 3, 7), str, r"Bar", &=r"B");
        expect! (data, (1, 3, 8), str, r"Second occurrence");
        expect! (data, (1, 3, 9), alias, r"A");
        expect! (data, (1, 3, 10), str, r"Reuse anchor");
        expect! (data, (1, 3, 11), alias, r"B");

        expect! (data, 11, dusk);
    }



    #[test]
    fn example_07_02 () {
        let src =
r#"{
  foo : !!str,
  !!str : bar,
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), map);
        expect! (data, (1, 2, 3), str, r"foo");
        expect! (data, (1, 2, 4), str, r"");
        expect! (data, (1, 2, 5), str, r"");
        expect! (data, (1, 2, 6), str, r"bar");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_07_02_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "foo" : !!str "",
  ? !!str ""    : !!str "bar",
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"foo");
        expect! (data, (1, 3, 5), str, r"");
        expect! (data, (1, 3, 6), str, r"");
        expect! (data, (1, 3, 7), str, r"bar");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_07_03 () {
        let src =
r#"{
  ? foo :,
  : bar,
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), map);
        expect! (data, (1, 2, 3), str, r"foo");
        expect! (data, (1, 2, 4), null);
        expect! (data, (1, 2, 5), null);
        expect! (data, (1, 2, 6), str, r"bar");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_07_03_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "foo" : !!null "",
  ? !!null ""   : !!str "bar",
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"foo");
        expect! (data, (1, 3, 5), null);
        expect! (data, (1, 3, 6), null);
        expect! (data, (1, 3, 7), str, r"bar");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_07_04 () {
        let src =
r#""implicit block key" : [
  "implicit flow key" : value,
 ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"implicit block key");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), str, r"implicit flow key");
        expect! (data, (2, 4, 6), lazymap, (2, 4, 5));
        expect! (data, (3, 6, 7), str, r"value");

        expect! (data, 8, dusk);
    }



    #[test]
    fn example_07_04_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "implicit block key"
  : !!seq [
    !!map {
      ? !!str "implicit flow key"
      : !!str "value",
    }
  ]
}"#;

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"implicit block key");
        expect! (data, (1, 3, 5), seq);
        expect! (data, (2, 5, 6), map);
        expect! (data, (3, 6, 7), str, r"implicit flow key");
        expect! (data, (3, 6, 8), str, r"value");

        expect! (data, 8, dusk);
    }



    #[test]
    fn example_07_05 () {
        let src =
r#""folded 
to a space,	
 
to a line feed, or 	\
 \ 	non-content""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, "folded to a space,\nto a line feed, or \t \tnon-content");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_07_05_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "folded to a space,\n\
      to a line feed, \
      or \t \tnon-content""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, "folded to a space,\nto a line feed, or \t \tnon-content");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_07_06 () {
        let src =
r#"" 1st non-empty

 2nd non-empty 
	3rd non-empty ""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, " 1st non-empty\n2nd non-empty 3rd non-empty ");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_07_06_canonical () {
        let src =
r#"%YAML 1.2
---
!!str " 1st non-empty\n\
      2nd non-empty \
      3rd non-empty ""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, " 1st non-empty\n2nd non-empty 3rd non-empty ");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_07_07 () {
        let src =
r#"'here''s to "quotes"'"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r#"here's to "quotes""#);
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_07_07_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "here's to \"quotes\"""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, "here's to \"quotes\"");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_07_08 () {
        let src =
r#"'implicit block key' : [
  'implicit flow key' : value,
 ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"implicit block key");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), str, r"implicit flow key");
        expect! (data, (2, 4, 6), lazymap, (2, 4, 5));
        expect! (data, (3, 6, 7), str, r"value");

        expect! (data, 8, dusk);
    }



    #[test]
    fn example_07_09 () {
        let src =
r#"' 1st non-empty

 2nd non-empty 
	3rd non-empty '"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, " 1st non-empty\n2nd non-empty 3rd non-empty ");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_07_10 () {
        let src =
r#"# Outside flow collection:
- ::vector
- ": - ()"
- Up, up, and away!
- -123
- http://example.com/foo#bar
# Inside flow collection:
- [ ::vector,
  ": - ()",
  "Up, up and away!",
  -123,
  http://example.com/foo#bar ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), str, r"::vector");
        expect! (data, (1, 2, 7), str, r": - ()");
        expect! (data, (1, 2, 8), str, r"Up, up, and away!");
        expect! (data, (1, 2, 16), int, -123);
        expect! (data, (1, 2, 17), str, r"http://example.com/foo#bar");
        expect! (data, (1, 2, 23), seq);
        expect! (data, (2, 23, 24), str, r"::vector");
        expect! (data, (2, 23, 28), str, r": - ()");
        expect! (data, (2, 23, 29), str, r"Up, up and away!");
        expect! (data, (2, 23, 30), int, -123);
        expect! (data, (2, 23, 31), str, r"http://example.com/foo#bar");
        expect! (data, 14, dusk);
    }



    #[test]
    fn example_07_10_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!str "::vector",
  !!str ": - ()",
  !!str "Up, up, and away!",
  !!int "-123",
  !!str "http://example.com/foo#bar",
  !!seq [
    !!str "::vector",
    !!str ": - ()",
    !!str "Up, up, and away!",
    !!int "-123",
    !!str "http://example.com/foo#bar",
  ],
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), seq);
        expect! (data, (1, 3, 4), str, r"::vector");
        expect! (data, (1, 3, 5), str, r": - ()");
        expect! (data, (1, 3, 6), str, r"Up, up, and away!");
        expect! (data, (1, 3, 7), int, -123);
        expect! (data, (1, 3, 8), str, r"http://example.com/foo#bar");
        expect! (data, (1, 3, 9), seq);
        expect! (data, (2, 9, 10), str, r"::vector");
        expect! (data, (2, 9, 11), str, r": - ()");
        expect! (data, (2, 9, 12), str, r"Up, up, and away!");
        expect! (data, (2, 9, 13), int, -123);
        expect! (data, (2, 9, 14), str, r"http://example.com/foo#bar");
        expect! (data, 14, dusk);
    }



    #[test]
    fn example_07_11 () {
        let src =
r#"implicit block key : [
  implicit flow key : value,
 ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"implicit block key");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), str, r"implicit flow key");
        expect! (data, (2, 4, 6), lazymap, (2, 4, 5));
        expect! (data, (3, 6, 7), str, r"value");
        expect! (data, 8, dusk);
    }



    #[test]
    fn example_07_11_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "implicit block key"
  : !!seq [
    !!map {
      ? !!str "implicit flow key"
      : !!str "value",
    }
  ]
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"implicit block key");
        expect! (data, (1, 3, 5), seq);
        expect! (data, (2, 5, 6), map);
        expect! (data, (3, 6, 7), str, r"implicit flow key");
        expect! (data, (3, 6, 8), str, r"value");

        expect! (data, 8, dusk);
    }



    #[test]
    fn example_07_12 () {
        let src =
r#"1st non-empty

 2nd non-empty 
	3rd non-empty
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, "1st non-empty\n2nd non-empty 3rd non-empty");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_07_12_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "1st non-empty\n\
      2nd non-empty \
      3rd non-empty"
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, "1st non-empty\n2nd non-empty 3rd non-empty");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_07_13 () {
        let src =
r#"- [ one, two, ]
- [three ,four]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);

        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), str, r"one");
        expect! (data, (2, 3, 5), str, r"two");

        expect! (data, (1, 2, 6), seq);
        expect! (data, (2, 6, 7), str, r"three");
        expect! (data, (2, 6, 8), str, r"four");

        expect! (data, 9, dusk);
    }



    #[test]
    fn example_07_13_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!seq [
    !!str "one",
    !!str "two",
  ],
  !!seq [
    !!str "three",
    !!str "four",
  ],
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), seq);

        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), str, r"one");
        expect! (data, (2, 4, 6), str, r"two");

        expect! (data, (1, 3, 7), seq);
        expect! (data, (2, 7, 8), str, r"three");
        expect! (data, (2, 7, 9), str, r"four");

        expect! (data, 9, dusk);
    }



    #[test]
    fn example_07_14 () {
        let src =
r#"[
"double
 quoted", 'single
           quoted',
plain
 text, [ nested ],
single: pair,
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), str, r"double quoted");
        expect! (data, (1, 2, 4), str, r"single quoted");
        expect! (data, (1, 2, 5), str, r"plain text");

        expect! (data, (1, 2, 9), seq);
        expect! (data, (2, 9, 10), str, r"nested");

        expect! (data, (1, 2, 11), str, r"single");
        expect! (data, (1, 2, 12), lazymap, (1, 2, 11));
        expect! (data, (2, 12, 13), str, r"pair");
        expect! (data, 11, dusk);
    }



    #[test]
    fn example_07_14_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!str "double quoted",
  !!str "single quoted",
  !!str "plain text",
  !!seq [
    !!str "nested",
  ],
  !!map {
    ? !!str "single"
    : !!str "pair",
  },
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), seq);

        expect! (data, (1, 3, 4), str, r"double quoted");
        expect! (data, (1, 3, 5), str, r"single quoted");
        expect! (data, (1, 3, 6), str, r"plain text");

        expect! (data, (1, 3, 7), seq);
        expect! (data, (2, 7, 8), str, r"nested");

        expect! (data, (1, 3, 9), map);
        expect! (data, (2, 9, 10), str, r"single");
        expect! (data, (2, 9, 11), str, r"pair");

        expect! (data, 11, dusk);
    }



    #[test]
    fn example_07_15 () {
        let src =
r#"- { one : two , three: four , }
- {five: six,seven : eight}"#;

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), map);
        expect! (data, (2, 3, 4), str, r"one");
        expect! (data, (2, 3, 5), str, r"two");
        expect! (data, (2, 3, 6), str, r"three");
        expect! (data, (2, 3, 7), str, r"four");

        expect! (data, (1, 2, 8), map);
        expect! (data, (2, 8, 9), str, r"five");
        expect! (data, (2, 8, 10), str, r"six");
        expect! (data, (2, 8, 11), str, r"seven");
        expect! (data, (2, 8, 12), str, r"eight");

        expect! (data, 13, dusk);
    }



    #[test]
    fn example_07_15_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!map {
    ? !!str "one"   : !!str "two",
    ? !!str "three" : !!str "four",
  },
  !!map {
    ? !!str "five"  : !!str "six",
    ? !!str "seven" : !!str "eight",
  },
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), seq);
        expect! (data, (1, 3, 4), map);
        expect! (data, (2, 4, 5), str, r"one");
        expect! (data, (2, 4, 6), str, r"two");
        expect! (data, (2, 4, 7), str, r"three");
        expect! (data, (2, 4, 8), str, r"four");

        expect! (data, (1, 3, 9), map);
        expect! (data, (2, 9, 10), str, r"five");
        expect! (data, (2, 9, 11), str, r"six");
        expect! (data, (2, 9, 12), str, r"seven");
        expect! (data, (2, 9, 13), str, r"eight");

        expect! (data, 13, dusk);
    }



    #[test]
    fn example_07_16 () {
        let src =
r#"{
? explicit: entry,
implicit: entry,
?
}
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), map);
        expect! (data, (1, 2, 3), str, r"explicit");
        expect! (data, (1, 2, 4), str, r"entry");
        expect! (data, (1, 2, 5), str, r"implicit");
        expect! (data, (1, 2, 6), str, r"entry");
        expect! (data, (1, 2, 7), null);
        expect! (data, (1, 2, 8), null);
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_07_16_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "explicit" : !!str "entry",
  ? !!str "implicit" : !!str "entry",
  ? !!null "" : !!null "",
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"explicit");
        expect! (data, (1, 3, 5), str, r"entry");
        expect! (data, (1, 3, 6), str, r"implicit");
        expect! (data, (1, 3, 7), str, r"entry");
        expect! (data, (1, 3, 8), null);
        expect! (data, (1, 3, 9), null);

        expect! (data, 9, dusk);
    }



    #[test]
    fn example_07_17 () {
        let src =
r#"{
unquoted : "separate",
http://foo.com,
omitted value:,
: omitted key,
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), map);
        expect! (data, (1, 2, 3), str, r"unquoted");
        expect! (data, (1, 2, 4), str, r"separate");
        expect! (data, (1, 2, 5), str, r"http://foo.com");
        expect! (data, (1, 2, 9), null);
        expect! (data, (1, 2, 10), str, r"omitted value");
        expect! (data, (1, 2, 11), null);
        expect! (data, (1, 2, 12), null);
        expect! (data, (1, 2, 13), str, r"omitted key");
        expect! (data, 11, dusk);
    }



    #[test]
    fn example_07_17_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "unquoted" : !!str "separate",
  ? !!str "http://foo.com" : !!null "",
  ? !!str "omitted value" : !!null "",
  ? !!null "" : !!str "omitted key",
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"unquoted");
        expect! (data, (1, 3, 5), str, r"separate");
        expect! (data, (1, 3, 6), str, r"http://foo.com");
        expect! (data, (1, 3, 7), null);
        expect! (data, (1, 3, 8), str, r"omitted value");
        expect! (data, (1, 3, 9), null);
        expect! (data, (1, 3, 10), null);
        expect! (data, (1, 3, 11), str, r"omitted key");
        expect! (data, 11, dusk);
    }



    #[test]
    fn example_07_18 () {
        let src =
r#"{
"adjacent":value,
"readable": value,
"empty":
}"#;

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), map);
        expect! (data, (1, 2, 3), str, r"adjacent");
        expect! (data, (1, 2, 4), str, r"value");
        expect! (data, (1, 2, 5), str, r"readable");
        expect! (data, (1, 2, 6), str, r"value");
        expect! (data, (1, 2, 7), str, r"empty");
        expect! (data, (1, 2, 8), null);
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_07_18_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "adjacent" : !!str "value",
  ? !!str "readable" : !!str "value",
  ? !!str "empty"    : !!null "",
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"adjacent");
        expect! (data, (1, 3, 5), str, r"value");
        expect! (data, (1, 3, 6), str, r"readable");
        expect! (data, (1, 3, 7), str, r"value");
        expect! (data, (1, 3, 8), str, r"empty");
        expect! (data, (1, 3, 9), null);
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_07_19 () {
        let src =
r#"[
foo: bar
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), str, r"foo");
        expect! (data, (1, 2, 4), lazymap, (1, 2, 3));
        expect! (data, (2, 4, 5), str, r"bar");
        expect! (data, 6, dusk);
    }



    #[test]
    fn example_07_19_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!map { ? !!str "foo" : !!str "bar" }
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), seq);
        expect! (data, (1, 3, 4), map);
        expect! (data, (2, 4, 5), str, r"foo");
        expect! (data, (2, 4, 6), str, r"bar");
        expect! (data, 6, dusk);
    }



    #[test]
    fn example_07_20 () {
        let src =
r#"[
? foo
 bar : baz
]
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), map);
        expect! (data, (2, 3, 4), str, r"foo bar");
        expect! (data, (2, 3, 8), str, r"baz");
        expect! (data, 6, dusk);
    }



    #[test]
    fn example_07_20_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!map {
    ? !!str "foo bar"
    : !!str "baz",
  },
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), seq);
        expect! (data, (1, 3, 4), map);
        expect! (data, (2, 4, 5), str, r"foo bar");
        expect! (data, (2, 4, 6), str, r"baz");
        expect! (data, 6, dusk);
    }



    #[test]
    fn example_07_21 () {
        let src =
r#"- [ YAML : separate ]
- [ : empty key entry ]
- [ {JSON: like}:adjacent ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);

        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), str, r"YAML");
        expect! (data, (2, 3, 5), lazymap, (2, 3, 4));
        expect! (data, (3, 5, 6), str, r"separate");

        expect! (data, (1, 2, 7), seq);
        expect! (data, (2, 7, 8), map);
        expect! (data, (3, 8, 9), null);
        expect! (data, (3, 8, 10), str, r"empty key entry");

        expect! (data, (1, 2, 11), seq);
        expect! (data, (2, 11, 12), map);
        expect! (data, (3, 12, 13), str, r"JSON");
        expect! (data, (3, 12, 14), str, r"like");
        expect! (data, (2, 11, 15), lazymap, (2, 11, 12));
        expect! (data, (3, 15, 16), str, r"adjacent");

        expect! (data, 17, dusk);
    }



    #[test]
    fn example_07_21_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!seq [
    !!map {
      ? !!str "YAML"
      : !!str "separate"
    },
  ],
  !!seq [
    !!map {
      ? !!null ""
      : !!str "empty key entry"
    },
  ],
  !!seq [
    !!map {
      ? !!map {
        ? !!str "JSON"
        : !!str "like"
      } : "adjacent",
    },
  ],
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), seq);

        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), map);
        expect! (data, (3, 5, 6), str, r"YAML");
        expect! (data, (3, 5, 7), str, r"separate");

        expect! (data, (1, 3, 8), seq);
        expect! (data, (2, 8, 9), map);
        expect! (data, (3, 9, 10), null);
        expect! (data, (3, 9, 11), str, r"empty key entry");

        expect! (data, (1, 3, 12), seq);
        expect! (data, (2, 12, 13), map);
        expect! (data, (3, 13, 14), map);
        expect! (data, (4, 14, 15), str, r"JSON");
        expect! (data, (4, 14, 16), str, r"like");
        expect! (data, (3, 13, 17), str, r"adjacent");

        expect! (data, 17, dusk);
    }



    #[test]
    fn example_07_22 () {
        let src =
r#"[ foo
 bar: invalid,
 "foo...>1K characters...bar": invalid ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);

        expect! (data, (1, 2, 3), str, r"foo bar");
        expect! (data, (1, 2, 7), lazymap, (1, 2, 3));
        expect! (data, (2, 7, 8), str, r"invalid");
        expect! (data, (1, 2, 9), str, r"foo...>1K characters...bar");
        expect! (data, (1, 2, 10), lazymap, (1, 2, 9));
        expect! (data, (2, 10, 11), str, r"invalid");
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_07_22_extra () {
        let src =
r#"[ foo
 bar: invalid,
 "foo aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa bar": invalid ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);

        expect! (data, (1, 2, 3), str, r"foo bar");
        expect! (data, (1, 2, 7), lazymap, (1, 2, 3));
        expect! (data, (2, 7, 8), str, r"invalid");
        expect! (data, (1, 2, 9), str, r"foo aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa bar");
        expect! (data, (1, 2, 10), lazymap, (1, 2, 9));
        expect! (data, (2, 10, 11), str, r"invalid");
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_07_23 () {
        let src =
r#"- [ a, b ]
- { a: b }
- "a"
- 'b'
- c"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);

        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), str, r"a");
        expect! (data, (2, 3, 5), str, r"b");

        expect! (data, (1, 2, 6), map);
        expect! (data, (2, 6, 7), str, r"a");
        expect! (data, (2, 6, 8), str, r"b");

        expect! (data, (1, 2, 9), str, r"a");
        expect! (data, (1, 2, 10), str, r"b");
        expect! (data, (1, 2, 11), str, r"c");

        expect! (data, 12, dusk);
    }



    #[test]
    fn example_07_23_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!seq [ !!str "a", !!str "b" ],
  !!map { ? !!str "a" : !!str "b" },
  !!str "a",
  !!str "b",
  !!str "c",
]
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), seq);

        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), str, r"a");
        expect! (data, (2, 4, 6), str, r"b");

        expect! (data, (1, 3, 7), map);
        expect! (data, (2, 7, 8), str, r"a");
        expect! (data, (2, 7, 9), str, r"b");

        expect! (data, (1, 3, 10), str, r"a");
        expect! (data, (1, 3, 11), str, r"b");
        expect! (data, (1, 3, 12), str, r"c");

        expect! (data, 12, dusk);
    }



    #[test]
    fn example_07_24 () {
        let src =
r#"- !!str "a"
- 'b'
- &anchor "c"
- *anchor
- !!str"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);

        expect! (data, (1, 2, 3), str, r"a");
        expect! (data, (1, 2, 4), str, r"b");
        expect! (data, (1, 2, 5), str, r"c", &=r"anchor");
        expect! (data, (1, 2, 6), alias, r"anchor");
        expect! (data, (1, 2, 7), str, r"");

        expect! (data, 8, dusk);
    }



    #[test]
    fn example_07_24_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!str "a",
  !!str "b",
  &A !!str "c",
  *A,
  !!str "",
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), seq);

        expect! (data, (1, 3, 4), str, r"a");
        expect! (data, (1, 3, 5), str, r"b");
        expect! (data, (1, 3, 6), str, r"c", &=r"A");
        expect! (data, (1, 3, 7), alias, r"A");
        expect! (data, (1, 3, 8), str, r"");

        expect! (data, 8, dusk);
    }



    #[test]
    fn example_08_01 () {
        let src =
r#"- | # Empty header
 literal
- >1 # Indentation indicator
  folded
- |+ # Chomping indicator
 keep

- >1- # Both indicators
  strip

"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);

        expect! (data, (1, 2, 3), str, "literal\n");
        expect! (data, (1, 2, 6), str, " folded\n");
        expect! (data, (1, 2, 9), str, "keep\n\n");
        expect! (data, (1, 2, 13), str, " strip");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_08_01_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!str "literal\n",
  !!str "·folded\n",
  !!str "keep\n\n",
  !!str "·strip",
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), seq);

        expect! (data, (1, 3, 4), str, "literal\n");
        expect! (data, (1, 3, 5), str, "·folded\n");
        expect! (data, (1, 3, 6), str, "keep\n\n");
        expect! (data, (1, 3, 7), str, "·strip");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_08_02 () {
        let src =
r#"- |
 detected
- >
 
  
  # detected
- |1
  explicit
- >
 	
 detected
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);

        expect! (data, (1, 2, 3), str, "detected\n");
        expect! (data, (1, 2, 6), str, "\n\n# detected\n");
        expect! (data, (1, 2, 11), str, " explicit\n");
        expect! (data, (1, 2, 14), str, "\t\ndetected\n");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_08_03_01 () {
        let src =
r#"- |
  
 text
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);

        expect! (data, (1, 2, 3), str, "\ntext\n");

        expect! (data, 4, dusk);
    }



    #[test]
    fn example_08_03_02 () {
        let src =
r#"- >
  text
 text
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);

        expect! (data, (1, 2, 3), str, "text\n");
        expect! (data, (0, 0, 6), str, r"text");

        expect! (data, 5, dusk);
    }



    #[test]
    fn example_08_03_03 () {
        let src =
r#"- |2
 text
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);

        expect! (data, (1, 2, 3), null);
        expect! (data, (0, 0, 4), str, r"text");

        expect! (data, 5, dusk);
    }



    #[test]
    fn example_08_04 () {
        let src =
r#"strip: |-
  text
clip: |
  text
keep: |+
  text
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"strip");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"text");
        expect! (data, (1, 3, 6), str, r"clip");
        expect! (data, (1, 3, 7), str, "text\n");
        expect! (data, (1, 3, 10), str, "keep");
        expect! (data, (1, 3, 11), str, "text\n");

        expect! (data, 9, dusk);
    }



    #[test]
    fn example_08_04_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "strip"
  : !!str "text",
  ? !!str "clip"
  : !!str "text\n",
  ? !!str "keep"
  : !!str "text\n",
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"strip");
        expect! (data, (1, 3, 5), str, r"text");
        expect! (data, (1, 3, 6), str, r"clip");
        expect! (data, (1, 3, 7), str, "text\n");
        expect! (data, (1, 3, 8), str, "keep");
        expect! (data, (1, 3, 9), str, "text\n");
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_08_05 () {
        let src =
r#" # Strip
  # Comments:
strip: |-
  # text
  
 # Clip
  # comments:

clip: |
  # text
 
 # Keep
  # comments:

keep: |+
  # text

 # Trail
  # comments."#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"strip");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"# text");
        expect! (data, (1, 3, 6), str, r"clip");
        expect! (data, (1, 3, 7), str, "# text\n");
        expect! (data, (1, 3, 10), str, "keep");
        expect! (data, (1, 3, 11), str, "# text\n\n");

        expect! (data, 9, dusk);
    }



    #[test]
    fn example_08_05_canonical () {
        let src =
r##"%YAML 1.2
---
!!map {
  ? !!str "strip"
  : !!str "# text",
  ? !!str "clip"
  : !!str "# text\n",
  ? !!str "keep"
  : !!str "# text\n",
}
"##;

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"strip");
        expect! (data, (1, 3, 5), str, r"# text");
        expect! (data, (1, 3, 6), str, r"clip");
        expect! (data, (1, 3, 7), str, "# text\n");
        expect! (data, (1, 3, 8), str, "keep");
        expect! (data, (1, 3, 9), str, "# text\n");
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_08_06 () {
        let src =
r#"strip: >-

clip: >

keep: |+

"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"strip");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), null);
        expect! (data, (1, 3, 5), str, r"clip");
        expect! (data, (1, 3, 6), null);
        expect! (data, (1, 3, 7), str, r"keep");
        expect! (data, (1, 3, 8), str, "\n");

        expect! (data, 9, dusk);
    }



    #[test]
    fn example_08_06_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "strip"
  : !!str "",
  ? !!str "clip"
  : !!str "",
  ? !!str "keep"
  : !!str "\n",
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"strip");
        expect! (data, (1, 3, 5), str, r"");
        expect! (data, (1, 3, 6), str, r"clip");
        expect! (data, (1, 3, 7), str, r"");
        expect! (data, (1, 3, 8), str, "keep");
        expect! (data, (1, 3, 9), str, "\n");
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_08_07 () {
        let src =
r#"|
 literal
 	text

"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, "literal\n\ttext\n");

        expect! (data, 3, dusk);
    }



    #[test]
    fn example_08_07_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "literal\n\ttext\n""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), str, "literal\n\ttext\n");

        expect! (data, 3, dusk);
    }



    #[test]
    fn example_08_08 () {
        let src =
r#"|
 
  
  literal
   
  
  text

 # Comment"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, "\n\nliteral\n \n\ntext\n");

        expect! (data, 3, dusk);
    }



    #[test]
    fn example_08_09 () {
        let src =
r#">
 folded
 text

"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, "folded text\n");

        expect! (data, 3, dusk);
    }



    #[test]
    fn example_08_10 () {
        let src =
r#">

 folded
 line

 next
 line
   * bullet

   * list
   * lines

 last
 line

# Comment"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, "\nfolded line\nnext line\n  * bullet\n\n  * list\n  * lines\n\nlast line\n");

        expect! (data, 3, dusk);
    }



    #[test]
    fn example_08_10_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "\n\
      folded line\n\
      next line\n\
      \  * bullet\n
      \n\
      \  * list\n\
      \  * lines\n\
      \n\
      last line\n""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), str, "\nfolded line\nnext line\n  * bullet\n \n  * list\n  * lines\n\nlast line\n");

        expect! (data, 3, dusk);
    }



    #[test]
    fn example_08_11 () {
        let src =
r#">

 folded
 line

 next
 line
   * bullet

   * list
   * lines

 last
 line

# Comment"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, "\nfolded line\nnext line\n  * bullet\n\n  * list\n  * lines\n\nlast line\n");

        expect! (data, 3, dusk);
    }



    #[test]
    fn example_08_14 () {
        let src =
r#"block sequence:
  - one
  - two : three
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"block sequence");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), str, r"one");
        expect! (data, (2, 4, 6), str, r"two");
        expect! (data, (2, 4, 7), lazymap, (2, 4, 6));
        expect! (data, (3, 7, 8), str, r"three");

        expect! (data, 9, dusk);
    }



    #[test]
    fn example_08_14_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "block sequence"
  : !!seq [
    !!str "one",
    !!map {
      ? !!str "two"
      : !!str "three"
    },
  ],
}"#;

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"block sequence");
        expect! (data, (1, 3, 5), seq);
        expect! (data, (2, 5, 6), str, r"one");
        expect! (data, (2, 5, 7), map);
        expect! (data, (3, 7, 8), str, r"two");
        expect! (data, (3, 7, 9), str, r"three");
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_08_15 () {
        let src =
r#"- # Empty
- |
 block node
- - one # Compact
  - two # sequence
- one: two # Compact mapping"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), null);
        expect! (data, (1, 2, 4), str, "block node\n");
        expect! (data, (1, 2, 7), seq);
        expect! (data, (2, 7, 8), str, r"one");
        expect! (data, (2, 7, 9), str, r"two");
        expect! (data, (1, 2, 10), str, r"one");
        expect! (data, (1, 2, 11), lazymap, (1, 2, 10));
        expect! (data, (2, 11, 12), str, r"two");

        expect! (data, 11, dusk);
    }



    #[test]
    fn example_08_15_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!null "",
  !!str "block node\n",
  !!seq [
    !!str "one"
    !!str "two",
  ],
  !!map {
    ? !!str "one"
    : !!str "two",
  },
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 3), seq);
        expect! (data, (1, 3, 4), null);
        expect! (data, (1, 3, 5), str, "block node\n");

        expect! (data, (1, 3, 6), seq);
        expect! (data, (2, 6, 7), str, r"one");
        expect! (data, (2, 6, 8), str, r"two");

        expect! (data, (1, 3, 9), map);
        expect! (data, (2, 9, 10), str, r"one");
        expect! (data, (2, 9, 11), str, r"two");

        expect! (data, 11, dusk);
    }



    #[test]
    fn example_08_16 () {
        let src =
r#"block mapping:
 key: value
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"block mapping");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"key");
        expect! (data, (1, 3, 5), lazymap, (1, 3, 4));
        expect! (data, (2, 5, 6), str, r"value");
        expect! (data, 7, dusk);
    }



    #[test]
    fn example_08_17 () {
        let src =
r#"? explicit key # Empty value
? |
  block key
: - one # Explicit compact
  - two # block value
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), map);
        expect! (data, (1, 2, 3), str, r"explicit key");
        expect! (data, (1, 2, 4), null);
        expect! (data, (1, 2, 5), str, "block key\n");
        expect! (data, (1, 2, 8), seq);
        expect! (data, (2, 8, 9), str, r"one");
        expect! (data, (2, 8, 10), str, r"two");
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_08_18 () {
        let src =
r#"plain key: in-line value
: # Both empty
"quoted key":
- entry
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"plain key");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"in-line value");
        expect! (data, (1, 3, 5), null);
        expect! (data, (1, 3, 6), null);
        expect! (data, (1, 3, 7), str, r"quoted key");
        expect! (data, (1, 3, 8), seq);
        expect! (data, (2, 8, 9), str, r"entry");
        expect! (data, 10, dusk);
    }



    #[test]
    fn example_08_18_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "plain key"
  : !!str "in-line value",
  ? !!null ""
  : !!null "",
  ? !!str "quoted key"
  : !!seq [ !!str "entry" ],
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"plain key");
        expect! (data, (1, 3, 5), str, r"in-line value");
        expect! (data, (1, 3, 6), null);
        expect! (data, (1, 3, 7), null);
        expect! (data, (1, 3, 8), str, r"quoted key");
        expect! (data, (1, 3, 9), seq);
        expect! (data, (2, 9, 10), str, r"entry");
        expect! (data, 10, dusk);
    }



    #[test]
    fn example_08_19 () {
        let src =
r#"- sun: yellow
- ? earth: blue
  : moon: white
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), str, r"sun");
        expect! (data, (1, 2, 4), lazymap, (1, 2, 3));
        expect! (data, (2, 4, 5), str, r"yellow");
        expect! (data, (1, 2, 6), map);
        expect! (data, (2, 6, 7), str, r"earth");
        expect! (data, (2, 6, 8), lazymap, (2, 6, 7));
        expect! (data, (3, 8, 9), str, r"blue");
        expect! (data, (2, 6, 10), str, r"moon");
        expect! (data, (2, 6, 11), lazymap, (2, 6, 10));
        expect! (data, (3, 11, 12), str, r"white");
        expect! (data, 13, dusk);
    }



    #[test]
    fn example_08_19_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!map {
     !!str "sun" : !!str "yellow",
  },
  !!map {
    ? !!map {
      ? !!str "earth"
      : !!str "blue"
    },
    : !!map {
      ? !!str "moon"
      : !!str "white"
    },
  }
]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), seq);

        expect! (data, (1, 3, 4), map);
        expect! (data, (2, 4, 5), str, r"sun");
        expect! (data, (2, 4, 6), str, r"yellow");

        expect! (data, (1, 3, 7), map);
        expect! (data, (2, 7, 8), map);
        expect! (data, (3, 8, 9), str, r"earth");
        expect! (data, (3, 8, 10), str, r"blue");

        expect! (data, (2, 7, 11), null);
        expect! (data, (2, 7, 12), null);

        expect! (data, (2, 7, 13), map);
        expect! (data, (3, 13, 14), str, r"moon");
        expect! (data, (3, 13, 15), str, r"white");

        expect! (data, 15, dusk);
    }



    #[test]
    fn example_08_20 () {
        let src =
r#"-
  "flow in block"
- >
 Block scalar
- !!map # Block collection
  foo : bar
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), str, r"flow in block");
        expect! (data, (1, 2, 4), str, "Block scalar\n");
        expect! (data, (1, 2, 7), str, r"foo");
        expect! (data, (1, 2, 8), lazymap, (1, 2, 7));
        expect! (data, (2, 8, 9), str, r"bar");

        expect! (data, 8, dusk);
    }



    #[test]
    fn example_08_20_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!str "flow in block",
  !!str "Block scalar\n",
  !!map {
    ? !!str "foo"
    : !!str "bar",
  },
]
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), seq);
        expect! (data, (1, 3, 4), str, r"flow in block");
        expect! (data, (1, 3, 5), str, "Block scalar\n");
        expect! (data, (1, 3, 6), map);
        expect! (data, (2, 6, 7), str, r"foo");
        expect! (data, (2, 6, 8), str, r"bar");
        expect! (data, 8, dusk);
    }



    #[test]
    fn example_08_21 () {
        let src =
r#"literal: |2
  value
folded:
   !foo
  >1
 value"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"literal");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, "value\n");

        expect! (data, (1, 3, 7), str, r"folded");
        expect! (data, (1, 3, 8), incognitum, r"!foo", r"value");

        expect! (data, 7, dusk);
    }



    #[test]
    fn example_08_21_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "literal"
  : !!str "value",
  ? !!str "folded"
  : !<!foo> "value",
}"#;

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"literal");
        expect! (data, (1, 3, 5), str, r"value");
        expect! (data, (1, 3, 6), str, r"folded");
        expect! (data, (1, 3, 7), incognitum, r"!foo", r#""value""#);
        expect! (data, 7, dusk);
    }



    #[test]
    fn example_08_22 () {
        let src =
r#"sequence: !!seq
- entry
- !!seq
 - nested
mapping: !!map
 foo: bar"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), str, r"sequence");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), str, r"entry");
        expect! (data, (2, 4, 6), seq);
        expect! (data, (3, 6, 7), str, r"nested");
        expect! (data, (1, 3, 8), str, r"mapping");
        expect! (data, (1, 3, 9), str, r"foo");
        expect! (data, (1, 3, 10), lazymap, (1, 3, 9));
        expect! (data, (2, 10, 11), str, r"bar");

        expect! (data, 12, dusk);
    }



    #[test]
    fn example_08_22_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  ? !!str "sequence"
  : !!seq [
    !!str "entry",
    !!seq [ !!str "nested" ],
  ],
  ? !!str "mapping"
  : !!map {
    ? !!str "foo" : !!str "bar",
  },
}"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);

        expect! (data, (1, 3, 4), str, r"sequence");
        expect! (data, (1, 3, 5), seq);
        expect! (data, (2, 5, 6), str, r"entry");
        expect! (data, (2, 5, 7), seq);
        expect! (data, (3, 7, 8), str, r"nested");
        expect! (data, (1, 3, 9), str, r"mapping");
        expect! (data, (1, 3, 10), map);
        expect! (data, (2, 10, 11), str, r"foo");
        expect! (data, (2, 10, 12), str, r"bar");

        expect! (data, 12, dusk);
    }



    #[test]
    fn example_09_01 () {
        let src =
b"\xEF\xBB\xBF# Comment
# lines
Document";


        let sage = sage_bytes! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"Document");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_09_01_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "Document""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, r"Document");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_09_02 () {
        let src =
r#"%YAML 1.2
---
Document
... # Suffix"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, r"Document");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_09_02_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "Document""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, r"Document");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_09_03 () {
        let src =
r#"Bare
document
...
# No document
...
|
%!PS-Adobe-2.0 # Not the first line"#;


        let sage = sage! (src);
        let mut data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"Bare document");
        expect! (data, 3, dusk);

        let mut data = data.split_off (3);
        expect! (data, 1, dawn);
        expect! (data, 2, dusk);

        let data = data.split_off (2);
        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"%!PS-Adobe-2.0 # Not the first line");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_09_03_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "Bare document"
%YAML 1.2
---
!!str "%!PS-Adobe-2.0\n""#;


        let sage = sage! (src);
        let mut data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, r"Bare document");
        expect! (data, 3, dusk);

        let data = data.split_off(3);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, "%!PS-Adobe-2.0\n");
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_09_04 () {
        let src =
r#"---
{ matches
% : 20 }
...
---
# Empty
..."#;


        let sage = sage! (src);
        let mut data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), map);
        expect! (data, (1, 2, 3), str, r"matches %");
        expect! (data, (1, 2, 7), int, 20);
        expect! (data, 5, dusk);

        let data = data.split_off (5);
        expect! (data, 1, dawn);
        expect! (data, 2, dusk);
    }



    #[test]
    fn example_09_04_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  !!str "matches %": !!int "20"
}
...
%YAML 1.2
---
!!null """#;


        let sage = sage! (src);
        let mut data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"matches %");
        expect! (data, (1, 3, 5), int, 20);
        expect! (data, 5, dusk);

        let data = data.split_off(5);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), null);
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_09_05 () {
        let src =
r#"%YAML 1.2
--- |
 %!PS-Adobe-2.0
...
%YAML 1.2
---
# Empty
..."#;


        let sage = sage! (src);
        let mut data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, "%!PS-Adobe-2.0\n");
        expect! (data, 3, dusk);

        let data = data.split_off(3);

        expect! (data, 1, dawn);
        expect! (data, 2, dusk);
    }



    #[test]
    fn example_09_05_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "%!PS-Adobe-2.0\n"
...
%YAML 1.2
---
!!null """#;


        let sage = sage! (src);
        let mut data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, "%!PS-Adobe-2.0\n");
        expect! (data, 3, dusk);

        let data = data.split_off(3);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), null);
        expect! (data, 3, dusk);
    }



    #[test]
    fn example_09_06 () {
        let src =
r#"Document
---
# Empty
...
%YAML 1.2
---
matches %: 20
"#;


        let sage = sage! (src);
        let mut data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"Document");
        expect! (data, 3, dusk);


        let mut data = data.split_off(3);

        expect! (data, 1, dawn);
        expect! (data, 2, dusk);


        let data = data.split_off (2);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, r"matches %");
        expect! (data, (0, 0, 4), lazymap, (0, 0, 3));
        expect! (data, (1, 4, 5), int, 20);
        expect! (data, 5, dusk);
    }



    #[test]
    fn example_09_06_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "Document"
...
%YAML 1.2
---
!!null ""
...
%YAML 1.2
---
!!map {
  !!str "matches %": !!int "20"
}
"#;


        let sage = sage! (src);
        let mut data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), str, r"Document");
        expect! (data, 3, dusk);


        let mut data = data.split_off(3);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), null);
        expect! (data, 3, dusk);


        let data = data.split_off (3);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"matches %");
        expect! (data, (1, 3, 5), int, 20);
        expect! (data, 5, dusk);
    }



    #[test]
    fn example_10_01 () {
        let src =
r#"Block style: !!map
  Clark : Evans
  Ingy  : döt Net
  Oren  : Ben-Kiki

Flow style: !!map { Clark: Evans, Ingy: döt Net, Oren: Ben-Kiki }"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"Block style");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"Clark");
        expect! (data, (1, 3, 5), lazymap, (1, 3, 4));
        expect! (data, (2, 5, 6), str, r"Evans");
        expect! (data, (2, 5, 7), str, r"Ingy");
        expect! (data, (2, 5, 8), str, r"döt Net");
        expect! (data, (2, 5, 9), str, r"Oren");
        expect! (data, (2, 5, 10), str, r"Ben-Kiki");
        expect! (data, (1, 3, 11), str, r"Flow style");
        expect! (data, (1, 3, 12), map);
        expect! (data, (2, 12, 13), str, r"Clark");
        expect! (data, (2, 12, 14), str, r"Evans");
        expect! (data, (2, 12, 15), str, r"Ingy");
        expect! (data, (2, 12, 16), str, r"döt Net");
        expect! (data, (2, 12, 17), str, r"Oren");
        expect! (data, (2, 12, 18), str, r"Ben-Kiki");
        expect! (data, 19, dusk);
    }



    #[test]
    fn example_10_02 () {
        let src =
r#"Block style: !!seq
- Clark Evans
- Ingy döt Net
- Oren Ben-Kiki

Flow style: !!seq [ Clark Evans, Ingy döt Net, Oren Ben-Kiki ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"Block style");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), seq);
        expect! (data, (2, 4, 5), str, r"Clark Evans");
        expect! (data, (2, 4, 6), str, r"Ingy döt Net");
        expect! (data, (2, 4, 7), str, r"Oren Ben-Kiki");

        expect! (data, (1, 3, 8), str, r"Flow style");
        expect! (data, (1, 3, 9), seq);
        expect! (data, (2, 9, 10), str, r"Clark Evans");
        expect! (data, (2, 9, 11), str, r"Ingy döt Net");
        expect! (data, (2, 9, 12), str, r"Oren Ben-Kiki");
        expect! (data, 13, dusk);
    }



    #[test]
    fn example_10_03 () {
        let src =
r#"Block style: !!str |-
  String: just a theory.

Flow style: !!str "String: just a theory.""#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"Block style");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"String: just a theory.");
        expect! (data, (1, 3, 6), str, r"Flow style");
        expect! (data, (1, 3, 7), str, r"String: just a theory.");
        expect! (data, 7, dusk);
    }



    #[test]
    fn example_10_04 () {
        let src =
r#"!!null null: value for null key
key with null value: !!null null"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), null);
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), str, r"value for null key");
        expect! (data, (1, 3, 5), str, r"key with null value");
        expect! (data, (1, 3, 6), null);
        expect! (data, 7, dusk);
    }



    #[test]
    fn example_10_05 () {
        let src =
r#"YAML is a superset of JSON: !!bool true
Pluto is a planet: !!bool false"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"YAML is a superset of JSON");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), bool, true);
        expect! (data, (1, 3, 5), str, r"Pluto is a planet");
        expect! (data, (1, 3, 6), bool, false);
        expect! (data, 7, dusk);
    }



    #[test]
    fn example_10_06 () {
        let src =
r#"negative: !!int -12
zero: !!int 0
positive: !!int 34"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"negative");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), int, -12);
        expect! (data, (1, 3, 5), str, r"zero");
        expect! (data, (1, 3, 6), int, 0);
        expect! (data, (1, 3, 7), str, r"positive");
        expect! (data, (1, 3, 8), int, 34);
        expect! (data, 9, dusk);
    }



    #[test]
    fn example_10_07 () {
        let src =
r#"negative: !!float -1
zero: !!float 0
positive: !!float 2.3e4
infinity: !!float .inf
not a number: !!float .nan"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"negative");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), float, -1.0);
        expect! (data, (1, 3, 5), str, r"zero");
        expect! (data, (1, 3, 6), float, 0.0);
        expect! (data, (1, 3, 7), str, r"positive");
        expect! (data, (1, 3, 8), float, 2.3e4);
        expect! (data, (1, 3, 9), str, r"infinity");
        expect! (data, (1, 3, 10), float::inf);
        expect! (data, (1, 3, 11), str, r"not a number");
        expect! (data, (1, 3, 12), float::nan);
        expect! (data, 13, dusk);
    }



    #[test]
    fn example_10_08 () {
        let src =
r#"A null: null
Booleans: [ true, false ]
Integers: [ 0, -0, 3, -19 ]
Floats: [ 0., -0.0, 12e03, -2E+05 ]
Invalid: [ True, Null, 0o7, 0x3A, +12.3 ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"A null");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), null);
        expect! (data, (1, 3, 5), str, r"Booleans");
        expect! (data, (1, 3, 6), seq);
        expect! (data, (2, 6, 7), bool, true);
        expect! (data, (2, 6, 8), bool, false);
        expect! (data, (1, 3, 9), str, r"Integers");
        expect! (data, (1, 3, 10), seq);
        expect! (data, (2, 10, 11), int, 0);
        expect! (data, (2, 10, 12), int, 0);
        expect! (data, (2, 10, 13), int, 3);
        expect! (data, (2, 10, 14), int, -19);
        expect! (data, (1, 3, 15), str, r"Floats");
        expect! (data, (1, 3, 16), seq);
        expect! (data, (2, 16, 17), float, 0.0);
        expect! (data, (2, 16, 18), float, 0.0);
        expect! (data, (2, 16, 19), float, 12e03);
        expect! (data, (2, 16, 20), float, -2e+5);
        expect! (data, (1, 3, 21), str, r"Invalid");
        expect! (data, (1, 3, 22), seq);
        expect! (data, (2, 22, 23), bool, true);
        expect! (data, (2, 22, 24), null);
        expect! (data, (2, 22, 25), int, 7);
        expect! (data, (2, 22, 26), int, 0x3A);
        expect! (data, (2, 22, 27), float, 12.3);
        expect! (data, 28, dusk);
    }



    #[test]
    fn example_10_08_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  !!str "A null" : !!null "null",
  !!str "Booleans": !!seq [
    !!bool "true", !!bool "false"
  ],
  !!str "Integers": !!seq [
    !!int "0", !!int "-0",
    !!int "3", !!int "-19"
  ],
  !!str "Floats": !!seq [
    !!float "0.", !!float "-0.0",
    !!float "12e03", !!float "-2E+05"
  ],
  !!str "Invalid": !!seq [
    # Rejected by the schema
    True, Null, 0o7, 0x3A, +12.3,
  ],
}
..."#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"A null");
        expect! (data, (1, 3, 5), null);
        expect! (data, (1, 3, 6), str, r"Booleans");
        expect! (data, (1, 3, 7), seq);
        expect! (data, (2, 7, 8), bool, true);
        expect! (data, (2, 7, 9), bool, false);
        expect! (data, (1, 3, 10), str, r"Integers");
        expect! (data, (1, 3, 11), seq);
        expect! (data, (2, 11, 12), int, 0);
        expect! (data, (2, 11, 13), int, 0);
        expect! (data, (2, 11, 14), int, 3);
        expect! (data, (2, 11, 15), int, -19);
        expect! (data, (1, 3, 16), str, r"Floats");
        expect! (data, (1, 3, 17), seq);
        expect! (data, (2, 17, 18), float, 0.0);
        expect! (data, (2, 17, 19), float, 0.0);
        expect! (data, (2, 17, 20), float, 12e03);
        expect! (data, (2, 17, 21), float, -2e05);
        expect! (data, (1, 3, 22), str, r"Invalid");
        expect! (data, (1, 3, 23), seq);
        expect! (data, (2, 23, 24), bool, true);
        expect! (data, (2, 23, 25), null);
        expect! (data, (2, 23, 26), int, 7);
        expect! (data, (2, 23, 27), int, 0x3A);
        expect! (data, (2, 23, 28), float, 12.3);
        expect! (data, 28, dusk);
    }



    #[test]
    fn example_10_09 () {
        let src =
r#"A null: null
Also a null: # Empty
Not a null: ""
Booleans: [ true, True, false, FALSE ]
Integers: [ 0, 0o7, 0x3A, -19 ]
Floats: [ 0., -0.0, .5, +12e03, -2E+05 ]
Also floats: [ .inf, -.Inf, +.INF, .NAN ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), str, r"A null");
        expect! (data, (0, 0, 3), lazymap, (0, 0, 2));
        expect! (data, (1, 3, 4), null);
        expect! (data, (1, 3, 5), str, r"Also a null");
        expect! (data, (1, 3, 6), null);
        expect! (data, (1, 3, 7), str, r"Not a null");
        expect! (data, (1, 3, 8), str, r"");
        expect! (data, (1, 3, 9), str, r"Booleans");
        expect! (data, (1, 3, 10), seq);
        expect! (data, (2, 10, 11), bool, true);
        expect! (data, (2, 10, 12), bool, true);
        expect! (data, (2, 10, 13), bool, false);
        expect! (data, (2, 10, 14), bool, false);
        expect! (data, (1, 3, 15), str, r"Integers");
        expect! (data, (1, 3, 16), seq);
        expect! (data, (2, 16, 17), int, 0);
        expect! (data, (2, 16, 18), int, 7);
        expect! (data, (2, 16, 19), int, 0x3A);
        expect! (data, (2, 16, 20), int, -19);
        expect! (data, (1, 3, 21), str, r"Floats");
        expect! (data, (1, 3, 22), seq);
	expect! (data, (2, 22, 23), float, 0.0);
        expect! (data, (2, 22, 24), float, 0.0);
        expect! (data, (2, 22, 25), float, 0.5);
        expect! (data, (2, 22, 26), float, 12e3);
        expect! (data, (2, 22, 27), float, -2e5);
        expect! (data, (1, 3, 28), str, r"Also floats");
        expect! (data, (1, 3, 29), seq);
        expect! (data, (2, 29, 30), float::inf);
        expect! (data, (2, 29, 31), float::neg_inf);
        expect! (data, (2, 29, 32), float::inf);
        expect! (data, (2, 29, 33), float::nan);
        expect! (data, 34, dusk);
    }



    #[test]
    fn example_10_09_canonical () {
        let src =
r#"%YAML 1.2
---
!!map {
  !!str "A null" : !!null "null",
  !!str "Also a null" : !!null "",
  !!str "Not a null" : !!str "",
  !!str "Booleans": !!seq [
    !!bool "true", !!bool "True",
    !!bool "false", !!bool "FALSE",
  ],
  !!str "Integers": !!seq [
    !!int "0", !!int "0o7",
    !!int "0x3A", !!int "-19",
  ],
  !!str "Floats": !!seq [
    !!float "0.", !!float "-0.0", !!float ".5",
    !!float "+12e03", !!float "-2E+05"
  ],
  !!str "Also floats": !!seq [
    !!float ".inf", !!float "-.Inf",
    !!float "+.INF", !!float ".NAN",
  ],
}
...
"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 3), map);
        expect! (data, (1, 3, 4), str, r"A null");
        expect! (data, (1, 3, 5), null);
        expect! (data, (1, 3, 6), str, r"Also a null");
        expect! (data, (1, 3, 7), null);
        expect! (data, (1, 3, 8), str, r"Not a null");
        expect! (data, (1, 3, 9), str, r"");
        expect! (data, (1, 3, 10), str, r"Booleans");
        expect! (data, (1, 3, 11), seq);
        expect! (data, (2, 11, 12), bool, true);
        expect! (data, (2, 11, 13), bool, true);
        expect! (data, (2, 11, 14), bool, false);
        expect! (data, (2, 11, 15), bool, false);
        expect! (data, (1, 3, 16), str, r"Integers");
        expect! (data, (1, 3, 17), seq);
        expect! (data, (2, 17, 18), int, 0);
        expect! (data, (2, 17, 19), int, 7);
        expect! (data, (2, 17, 20), int, 0x3A);
        expect! (data, (2, 17, 21), int, -19);
        expect! (data, (1, 3, 22), str, r"Floats");
        expect! (data, (1, 3, 23), seq);
        expect! (data, (2, 23, 24), float, 0.0);
        expect! (data, (2, 23, 25), float, 0.0);
        expect! (data, (2, 23, 26), float, 0.5);
        expect! (data, (2, 23, 27), float, 12e3);
        expect! (data, (2, 23, 28), float, -2e5);
        expect! (data, (1, 3, 29), str, r"Also floats");
        expect! (data, (1, 3, 30), seq);
        expect! (data, (2, 30, 31), float::inf);
        expect! (data, (2, 30, 32), float::neg_inf);
        expect! (data, (2, 30, 33), float::inf);
        expect! (data, (2, 30, 34), float::nan);
        expect! (data, 34, dusk);
    }



    #[test]
    fn extra_desolation_00 () {
        let src =
r#"---
- [ !!str ]
- [ !!str , ]
- [ ]
- { !!str : !!str , }
- { !!str : !!str }
- { !!str : }
- { !!str }
- { }
- !!str
- !!str :
    !!str
- !!str :
- ? !!str
  : !!str
- ? !!str
  :
- ? !!str
-"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);

        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), str, r"");

        expect! (data, (1, 2, 5), seq);
        expect! (data, (2, 5, 6), str, r"");

        expect! (data, (1, 2, 7), seq);

        expect! (data, (1, 2, 8), map);
        expect! (data, (2, 8, 9), str, r"");
        expect! (data, (2, 8, 10), str, r"");

        expect! (data, (1, 2, 11), map);
        expect! (data, (2, 11, 12), str, r"");
        expect! (data, (2, 11, 13), str, r"");

        expect! (data, (1, 2, 14), map);
        expect! (data, (2, 14, 15), str, r"");
        expect! (data, (2, 14, 16), null);

        expect! (data, (1, 2, 17), map);
        expect! (data, (2, 17, 18), str, r"");
        expect! (data, (2, 17, 19), null);

        expect! (data, (1, 2, 20), map);

        expect! (data, (1, 2, 21), str, r"");

        expect! (data, (1, 2, 22), str, r"");
        expect! (data, (1, 2, 23), lazymap, (1, 2, 22));
        expect! (data, (2, 23, 24), str, r"");

        expect! (data, (1, 2, 25), str, r"");
        expect! (data, (1, 2, 26), lazymap, (1, 2, 25));

        expect! (data, (1, 2, 27), map);
        expect! (data, (2, 27, 28), str, r"");
        expect! (data, (2, 27, 29), str, r"");

        expect! (data, (1, 2, 30), map);
        expect! (data, (2, 30, 31), str, r"");

        expect! (data, (1, 2, 32), map);
        expect! (data, (2, 32, 33), str, r"");

        expect! (data, (1, 2, 34), null);

        expect! (data, 35, dusk);
    }



    #[test]
    fn extra_1_01 () {
        let src =
r#"---
- [ !!binary 'dGVzdCBzdHJpbmc='  ,
    !!binary  dGVzdCBzdHJpbmc=   ,
    !!binary "dGVzdCBzdHJpbmc="  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), binary, "test string".as_bytes ());
        expect! (data, (2, 3, 5), binary, "test string".as_bytes ());
        expect! (data, (2, 3, 6), binary, "test string".as_bytes ());

        expect! (data, 7, dusk);
    }



    #[test]
    fn extra_1_02 () {
        let src =
r#"---
- [ !!binary 'dGVzdCBzdHJpbmc=  ',
    !!binary  dGVzdCBzdHJpbmc=   ,
    !!binary "dGVzdCBzdHJpbmc=  ", ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), binary, "test string".as_bytes ());
        expect! (data, (2, 3, 5), binary, "test string".as_bytes ());
        expect! (data, (2, 3, 6), binary, "test string".as_bytes ());

        expect! (data, 7, dusk);
    }



    #[test]
    fn extra_1_03 () {
        let src =
r#"---
- [ !!bool 'true'  ,
    !!bool  true   ,
    !!bool "true"  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), bool, true);
        expect! (data, (2, 3, 5), bool, true);
        expect! (data, (2, 3, 6), bool, true);

        expect! (data, 7, dusk);
    }



    #[test]
    fn extra_1_04 () {
        let src =
r#"---
- [ !!bool 'false'  ,
    !!bool  false   ,
    !!bool "false"  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), bool, false);
        expect! (data, (2, 3, 5), bool, false);
        expect! (data, (2, 3, 6), bool, false);

        expect! (data, 7, dusk);
    }



    #[test]
    fn extra_1_05 () {
        let src =
r#"---
- [ !!float '0.451'  ,
    !!float  0.451   ,
    !!float "0.451"  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), float, 0.451);
        expect! (data, (2, 3, 5), float, 0.451);
        expect! (data, (2, 3, 6), float, 0.451);

        expect! (data, 7, dusk);
    }



    #[test]
    fn extra_1_06 () {
        let src =
r#"---
- [ !!int '42'  ,
    !!int  42   ,
    !!int "42"  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), int, 42);
        expect! (data, (2, 3, 5), int, 42);
        expect! (data, (2, 3, 6), int, 42);

        expect! (data, 7, dusk);
    }



    #[test]
    fn extra_1_07 () {
        let src =
r#"---
- [ !!merge '<<'  ,
    !!merge  <<   ,
    !!merge "<<"  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), merge);
        expect! (data, (2, 3, 5), merge);
        expect! (data, (2, 3, 6), merge);

        expect! (data, 7, dusk);
    }



    #[test]
    fn extra_1_08 () {
        let src =
r#"---
- [ !!null '~'  ,
    !!null  ~   ,
    !!null "~"  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), null);
        expect! (data, (2, 3, 5), null);
        expect! (data, (2, 3, 6), null);

        expect! (data, 7, dusk);
    }



    #[test]
    fn extra_1_09 () {
        let src =
r#"---
- [ !!null 'null'  ,
    !!null  null   ,
    !!null "null"  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), null);
        expect! (data, (2, 3, 5), null);
        expect! (data, (2, 3, 6), null);

        expect! (data, 7, dusk);
    }



    #[test]
    fn extra_1_10 () {
        let src =
r#"---
- [ !!null ''  ,
    !!null     ,
    !!null ""  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), null);
        expect! (data, (2, 3, 5), null);
        expect! (data, (2, 3, 6), null);

        expect! (data, 7, dusk);
    }



    #[test]
    fn extra_1_11 () {
        let src =
r#"---
- [ !!str ''  ,
    !!str     ,
    !!str ""  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), str, r"");
        expect! (data, (2, 3, 5), str, r"");
        expect! (data, (2, 3, 6), str, r"");

        expect! (data, 7, dusk);
    }



    #[test]
    fn extra_1_12 () {
        let src =
r#"---
- [ !!str 'test' ,
    !!str  test  ,
    !!str "test" , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), str, r"test");
        expect! (data, (2, 3, 5), str, r"test");
        expect! (data, (2, 3, 6), str, r"test");

        expect! (data, 7, dusk);
    }



    #[test]
    fn extra_1_13 () {
        let src =
r#"---
- [ !!timestamp '2001-12-15T02:59:43.1Z'  ,
    !!timestamp  2001-12-15T02:59:43.1Z   ,
    !!timestamp "2001-12-15T02:59:43.1Z"  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);

        expect! (
            data,
            (2, 3, 4),
            timestamp,
            Some (2001),
            Some (12),
            Some (15),
            Some (2),
            Some (59),
            Some (43),
            Some (FloatValue::from (Fraction::from (0.1))),
            Some (0),
            Some (0)
        );

        expect! (
            data,
            (2, 3, 5),
            timestamp,
            Some (2001),
            Some (12),
            Some (15),
            Some (2),
            Some (59),
            Some (43),
            Some (FloatValue::from (Fraction::from (0.1))),
            Some (0),
            Some (0)
        );

        expect! (
            data,
            (2, 3, 11),
            timestamp,
            Some (2001),
            Some (12),
            Some (15),
            Some (2),
            Some (59),
            Some (43),
            Some (FloatValue::from (Fraction::from (0.1))),
            Some (0),
            Some (0)
        );

        expect! (data, 7, dusk);
    }



    #[test]
    fn extra_1_14 () {
        let src =
r#"---
- [ !!value '='  ,
    !!value  =   ,
    !!value "="  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), value);
        expect! (data, (2, 3, 5), value);
        expect! (data, (2, 3, 6), value);

        expect! (data, 7, dusk);
    }



    #[test]
    fn extra_1_15 () {
        let src =
r#"---
- [ !!yaml '!'  ,
    !!yaml "!"  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);

        expect! (data, (2, 3, 4), yaml, tag);
        expect! (data, (2, 3, 5), yaml, tag);

        expect! (data, 6, dusk);
    }



    #[test]
    fn extra_1_16 () {
        let src =
r#"---
- [ !!yaml '&'  ,
    !!yaml "&"  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);

        expect! (data, (2, 3, 4), yaml, anchor);
        expect! (data, (2, 3, 5), yaml, anchor);

        expect! (data, 6, dusk);
    }



    #[test]
    fn extra_1_17 () {
        let src =
r#"---
- [ !!yaml '*'  ,
    !!yaml "*"  , ]"#;


        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);

        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);

        expect! (data, (2, 3, 4), yaml, alias);
        expect! (data, (2, 3, 5), yaml, alias);

        expect! (data, 6, dusk);
    }


    #[test]
    fn extra_2_01 () {
        let src =
r"- [a, b, c]
- &b [d, e, f]
- [g, h, i]
- *b
- [e, ., +]
";

        let sage = sage! (src);
        let data = read! (sage);

        expect! (data, 1, dawn);
        expect! (data, (0, 0, 2), seq);
        expect! (data, (1, 2, 3), seq);
        expect! (data, (2, 3, 4), str, "a");
        expect! (data, (2, 3, 5), str, "b");
        expect! (data, (2, 3, 6), str, "c");
        expect! (data, (1, 2, 7), seq, &=r"b");
        expect! (data, (2, 7, 8), str, "d");
        expect! (data, (2, 7, 9), str, "e");
        expect! (data, (2, 7, 10), str, "f");
        expect! (data, (1, 2, 11), seq);
        expect! (data, (2, 11, 12), str, "g");
        expect! (data, (2, 11, 13), str, "h");
        expect! (data, (2, 11, 14), str, "i");
        expect! (data, (1, 2, 15), alias, r"b");
        expect! (data, (1, 2, 16), seq);
        expect! (data, (2, 16, 17), str, "e");
        expect! (data, (2, 16, 18), str, ".");
        expect! (data, (2, 16, 19), str, "+");
    }
}
