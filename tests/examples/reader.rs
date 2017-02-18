macro_rules! data {
    () => {{ Data::with_capacity (4) }};
}


macro_rules! read {
    ($src:expr) => {{
        let (sender, receiver) = channel ();
        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new ($src.as_bytes ()),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        ).unwrap_or_else (|err| { assert! (false, format! ("Unexpected result: {}, :{}", err, err.position)); });

        receiver
    }}
}


macro_rules! read_with_error {
    ($src:expr, $err_desc:expr, $err_pos:expr) => {{
        let (sender, receiver) = channel ();
        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

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

        receiver
    }}
}


macro_rules! read_bytes {
    ($src:expr) => {{
        let (sender, receiver) = channel ();
        let mut reader = Reader::new (Tokenizer::new (get_charset_utf8 ()));

        reader.read (
            SliceReader::new ($src),
            &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
        ).unwrap_or_else (|err| { assert! (false, format! ("Unexpected result: {}, :{}", err, err.position)); });

        receiver
    }}
}


macro_rules! the_end {
    ( $receiver:expr ) => {
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, 0, 0, 0);

            if let BlockType::StreamEnd = block.cargo { } else { assert! (false, format! ("Unexpected result: expected StreamEnd, got {:?}", block.cargo)) }
        } else { assert! (false, "Cannot fetch the last block") }
    }
}


macro_rules! assert_id {
    ( $id:expr, $level:expr, $parent:expr, $index:expr ) => {{
        assert! ($id.level == $level, format! ("Level; actual != expected; {} != {}", $id.level, $level));
        assert! ($id.parent == $parent, format! ("Parent; actual != expected; {} != {}", $id.parent, $parent));
        assert! ($id.index == $index, format! ("Index; actual != expected; {} != {}", $id.index, $index));
    }}
}


macro_rules! expect {
    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), datum, $data:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Datum (datum) = block.cargo {
                $data.push (datum);
            } else { assert! (false, "Not a datum provided") }
        } else { assert! (false, "No datum provided") };
    }};

    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), error, $desc:expr, $pos:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Error (err, pos) = block.cargo {
                assert_eq! (err, $desc);
                assert_eq! (pos, $pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), warning, $desc:expr, $pos:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Warning (err, pos) = block.cargo {
                assert_eq! (err, $desc);
                assert_eq! (pos, $pos);
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), dir, yaml, $version:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::DirectiveYaml ( version ) = block.cargo {
                assert_eq! ($version, version);
            } else { assert! (false, format! ("Unexpected result: expected DirectiveYaml, got {:?}", block.cargo)) }
        } else { assert! (false, "Cannot fetch a new block") }
    }};

    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, dir, tag, $handle:expr, $prefix:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::DirectiveTag ( (ref handle, ref prefix) ) = block.cargo {
                assert_eq! ($data.chunk (handle).as_slice (), $handle.as_bytes ());
                assert_eq! ($data.chunk (prefix).as_slice (), $prefix.as_bytes ());
            } else { assert! (false, format! ("Unexpected result: expected DirectiveTag, got {:?}", block.cargo)) }
        } else { assert! (false, "Cannot fetch a new block") }
    }};

    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), doc, start ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::DocStart = block.cargo { } else { assert! (false, format! ("Unexpected result: expected DocStart, got {:?}", block.cargo)) }
        } else { assert! (false, "Cannot fetch a new block") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), doc, end ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::DocEnd = block.cargo { } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, alias, $val:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Alias (ref marker) = block.cargo {
                assert_eq! ($data.chunk (marker).as_slice (), $val.as_bytes ());
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, node, mapping ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.anchor.is_none ());
                assert! (node.tag.is_none ());
                if let NodeKind::Mapping = node.content { } else { assert! (false, format! ("Unexpected result {:?}", node)); }
            } else { assert! (false, format! ("Unexpected result {:?}", block)) }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, node, mapping, &=$anchor:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.tag.is_none ());

                if let Some (ref marker) = node.anchor {
                    assert_eq! ($data.chunk (marker).as_slice (), $anchor.as_bytes ());
                } else { assert! (false, "Unexpected result"); }

                if let NodeKind::Mapping = node.content { } else { assert! (false, "Unexpected result"); }
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, node, mapping, !=$tag:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.anchor.is_none ());

                if let Some (ref marker) = node.tag {
                    assert_eq! ($data.chunk (marker).as_slice (), $tag.as_bytes ());
                } else { assert! (false, "Unexpected result"); }

                if let NodeKind::Mapping = node.content { } else { assert! (false, "Unexpected result {:?}", node.content); }
            } else { assert! (false, "Unexpected result {:?}", block.cargo) }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), node, sequence ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.anchor.is_none ());
                assert! (node.tag.is_none ());
                if let NodeKind::Sequence = node.content { } else { assert! (false, format! ("Unexpected node {:?}", node)); }
            } else { assert! (false, format! ("Unexpected block {:?}", block)) }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, node, sequence, !=$tag:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.anchor.is_none ());

                if let Some (ref marker) = node.tag {
                    assert_eq! ($data.chunk (marker).as_slice (), $tag.as_bytes ());
                } else { assert! (false, "Tag is None, must be {:?}", $tag); }

                if let NodeKind::Sequence = node.content { } else { assert! (false, "Unexpected result {:?}", node.content); }
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), node, null ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.anchor.is_none ());
                assert! (node.tag.is_none ());
                if let NodeKind::Null = node.content { } else { assert! (false, "Unexpected result"); }
            } else { assert! (false, "Unexpected result: {:?}", block) }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), node, null, !=$tag:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.anchor.is_none ());
                if let Some (chunk) = node.tag {
                    assert_eq! (&*chunk, $tag.as_bytes ());
                } else { assert! (false, "Tag is None, must be {:?}", $tag); }
                if let NodeKind::Null = node.content { } else { assert! (false, "Unexpected result"); }
            } else { assert! (false, "Unexpected result: {:?}", block) }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, node, scalar, $val:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.anchor.is_none ());
                assert! (node.tag.is_none ());
                if let NodeKind::Scalar (ref marker) = node.content {
                    assert_eq! ($data.chunk (marker).as_slice (), $val.as_bytes ());
                } else { assert! (false, "Unexpected result / not a scalar / {:?}", node.content); }
            } else { assert! (false, format! ("Unexpected result / not a node / {:?}", block.cargo)) }
        } else { assert! (false, "Unexpected result / empty queue") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, node, scalar, $val:expr, &=$anchor:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.tag.is_none ());

                if let Some (ref marker) = node.anchor {
                    assert_eq! ($data.chunk (marker).as_slice (), $anchor.as_bytes ());
                } else { assert! (false, "Unexpected result"); }

                if let NodeKind::Scalar (ref marker) = node.content {
                    assert_eq! ($data.chunk (marker).as_slice (), $val.as_bytes ());
                } else { assert! (false, "Unexpected result"); }
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, node, scalar, $val:expr, !=$tag:expr, &=$anchor:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                if let Some (ref marker) = node.tag {
                    assert_eq! ($data.chunk (marker).as_slice (), $tag.as_bytes ());
                } else { assert! (false, "Unexpected result / tag unequality"); }

                if let Some (ref marker) = node.anchor {
                    assert_eq! ($data.chunk (marker).as_slice (), $anchor.as_bytes ());
                } else { assert! (false, "Unexpected result"); }

                if let NodeKind::Scalar (ref marker) = node.content {
                    assert_eq! ($data.chunk (marker).as_slice (), $val.as_bytes ());
                } else { assert! (false, "Unexpected result / scalar unequality"); }
            } else { assert! (false, "Unexpected result / not a node") }
        } else { assert! (false, "Unexpected result / cannot fetch a block") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, node, scalar, $val:expr, !=$tag:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.anchor.is_none ());

                if let Some (ref marker) = node.tag {
                    assert_eq! ($data.chunk (marker).as_slice (), $tag.as_bytes ());
                } else { assert! (false, "Unexpected result / tag unequality / orig != expect / {:?} != {:?}", node.tag, $tag); }

                if let NodeKind::Scalar (ref marker) = node.content {
                    assert_eq! ($data.chunk (marker).as_slice (), $val.as_bytes ());
                } else { assert! (false, "Unexpected scalar / {:?}", &node.content); }
            } else { assert! (false, "Unexpected result / not a node") }
        } else { assert! (false, "Unexpected result / cannot fetch a block") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, node, scalar !=$tag:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.anchor.is_none ());

                if let Some (ref marker) = node.tag {
                    assert_eq! ($data.chunk (marker).as_slice (), $tag.as_bytes ());
                } else { assert! (false, "Unexpected result / tag unequality / orig != expect / {:?} != {:?}", node.tag, $tag); }

                if let NodeKind::Null = node.content {
                } else { assert! (false, "Unexpected result / scalar unequality"); }
            } else { assert! (false, "Unexpected result / not a node") }
        } else { assert! (false, "Unexpected result / cannot fetch a block") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, literal, $val:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Literal (ref marker) = block.cargo {
                assert_eq! ($data.chunk (marker).as_slice (), $val.as_bytes ());
            } else if let BlockType::Rune (ref rune, amount) = block.cargo {
                let mut vec: Vec<u8> = Vec::with_capacity (rune.len () * amount);
                for _ in 0 .. amount { vec.extend (rune.as_slice ()); }
                assert_eq! (vec.as_slice (), $val.as_bytes ());
            } else { assert! (false, format! ("Unexpected result: {:?}", block)) }
        } else { assert! (false, "No result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, block, map, ( $m_level:expr, $m_parent:expr, $m_index:expr ) ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::BlockMap (id, None, None) = block.cargo {
                assert_id! (id, $m_level, $m_parent, $m_index);
            } else { assert! (false, "Unexpected result: {:?}", block.cargo) }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, block, map, ( $m_level:expr, $m_parent:expr, $m_index:expr ), !=$tag:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::BlockMap (id, None, tag) = block.cargo {
                if tag.is_none () { assert! (false, "Tag is None, must be {:?}", $tag); }

                let tag = tag.unwrap ();

                assert_id! (id, $m_level, $m_parent, $m_index);
                assert_eq! ($data.chunk (&tag).as_slice (), $tag.as_bytes ());
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, block, map, ( $m_level:expr, $m_parent:expr, $m_index:expr ), &=$anchor:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::BlockMap (id, Some (ref anchor), None) = block.cargo {
                assert_id! (id, $m_level, $m_parent, $m_index);
                assert_eq! ($data.chunk (anchor).as_slice (), $anchor.as_bytes ());
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, block, map, ( $m_level:expr, $m_parent:expr, $m_index:expr ), &=$anchor:expr, !=$tag:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::BlockMap (id, anchor, tag) = block.cargo {
                if anchor.is_none () { assert! (false, "Anchor is None, must be {:?}", $anchor); }
                if tag.is_none () { assert! (false, "Tag is None, must be {:?}", $tag); }

                let anchor = anchor.unwrap ();
                let tag = tag.unwrap ();

                assert_id! (id, $m_level, $m_parent, $m_index);
                assert_eq! ($data.chunk (&anchor).as_slice (), $anchor.as_bytes ());
                assert_eq! ($data.chunk (&tag).as_slice (), $tag.as_bytes ());
            } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), node, block, open ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.anchor.is_none ());
                assert! (node.tag.is_none ());

                if let NodeKind::LiteralBlockOpen  = node.content { } else { assert! (false, "Unexpected result / not a block: {:?}", node.content); }
            } else { assert! (false, format! ("Unexpected result / not a node / {:?}", block.cargo)) }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), node, block, open, !=$tag:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.anchor.is_none ());

                if let Some (chunk) = node.tag {
                    assert_eq! (&*chunk, $tag.as_bytes ());
                } else { assert! (false, "Unexpected result / tag unequality"); }

                if let NodeKind::LiteralBlockOpen  = node.content { } else { assert! (false, "Unexpected result / not a block: {:?}", node.content); }
            } else { assert! (false, format! ("Unexpected result / not a node / {:?}", block.cargo)) }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), $data:expr, node, block, close, !=$tag:expr ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.anchor.is_none ());

                if let Some (ref marker) = node.tag {
                    assert_eq! ($data.chunk (marker).as_slice (), $tag.as_bytes ());
                } else { assert! (false, "Unexpected result / tag unequality"); }

                if let NodeKind::LiteralBlockClose  = node.content { } else { assert! (false, "Unexpected result / not a block"); }
            } else { assert! (false, format! ("Unexpected result / not a node / {:?}", block.cargo)) }
            // if let BlockType::BlockScalar = block.cargo { } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }};


    ( $receiver:expr, ( $level:expr, $parent:expr, $index:expr ), node, block, close ) => {{
        if let Ok (block) = $receiver.try_recv () {
            assert_id! (block.id, $level, $parent, $index);

            if let BlockType::Node (node) = block.cargo {
                assert! (node.anchor.is_none ());
                assert! (node.tag.is_none ());
                if let NodeKind::LiteralBlockClose  = node.content { } else { assert! (false, "Unexpected result / not a block"); }
            } else { assert! (false, format! ("Unexpected result / not a node / {:?}", block.cargo)) }
            // if let BlockType::BlockScalar = block.cargo { } else { assert! (false, "Unexpected result") }
        } else { assert! (false, "Unexpected result") }
    }};
}



// #[cfg (all (test, not (feature = "dev")))]
#[cfg (test)]
mod stable {
    extern crate skimmer;
    extern crate yamlette;

    use self::skimmer::{ Data, Symbol };
    use self::skimmer::reader::SliceReader;

    use self::yamlette::txt::{ Twine, get_charset_utf8 };

    use self::yamlette::reader::BlockType;
    use self::yamlette::reader::NodeKind;
    use self::yamlette::reader::Reader;

    use self::yamlette::tokenizer::Tokenizer;

    use std::sync::mpsc::channel;



    #[test]
    fn example_02_01 () {
        let src =
r"- Mark McGwire
- Sammy Sosa
- Ken Griffey";


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"Mark McGwire");
        expect! (receiver, (1, 2, 4), data, node, scalar, r"Sammy Sosa");
        expect! (receiver, (1, 2, 5), data, node, scalar, r"Ken Griffey");
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_02_02 () {
        let src =
r"hr:  65    # Home runs
avg: 0.278 # Batting average
rbi: 147   # Runs Batted In";


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"hr");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"65");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"avg");
        expect! (receiver, (1, 3, 6), data, node, scalar, r"0.278");
        expect! (receiver, (1, 3, 7), data, node, scalar, r"rbi");
        expect! (receiver, (1, 3, 8), data, node, scalar, r"147");
        expect! (receiver, (0, 0, 9), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"american");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, sequence);
        expect! (receiver, (2, 4, 5), data, node, scalar, r"Boston Red Sox");
        expect! (receiver, (2, 4, 6), data, node, scalar, r"Detroit Tigers");
        expect! (receiver, (2, 4, 7), data, node, scalar, r"New York Yankees");
        expect! (receiver, (1, 3, 8), data, node, scalar, r"national");
        expect! (receiver, (1, 3, 9), node, sequence);
        expect! (receiver, (2, 9, 10), data, node, scalar, r"New York Mets");
        expect! (receiver, (2, 9, 11), data, node, scalar, r"Chicago Cubs");
        expect! (receiver, (2, 9, 12), data, node, scalar, r"Atlanta Braves");
        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"name");
        expect! (receiver, (1, 2, 4), data, block, map, (1, 2, 3));
        expect! (receiver, (2, 4, 5), data, node, scalar, r"Mark McGwire");
        expect! (receiver, (2, 4, 6), data, node, scalar, r"hr");
        expect! (receiver, (2, 4, 7), data, node, scalar, r"65");
        expect! (receiver, (2, 4, 8), data, node, scalar, r"avg");
        expect! (receiver, (2, 4, 9), data, node, scalar, r"0.278");
        expect! (receiver, (1, 2, 10), data, node, scalar, r"name");
        expect! (receiver, (1, 2, 11), data, block, map, (1, 2, 10));
        expect! (receiver, (2, 11, 12), data, node, scalar, r"Sammy Sosa");
        expect! (receiver, (2, 11, 13), data, node, scalar, r"hr");
        expect! (receiver, (2, 11, 14), data, node, scalar, r"63");
        expect! (receiver, (2, 11, 15), data, node, scalar, r"avg");
        expect! (receiver, (2, 11, 16), data, node, scalar, r"0.288");
        expect! (receiver, (0, 0, 17), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_02_05 () {
        let src =
r"- [name        , hr, avg  ]
- [Mark McGwire, 65, 0.278]
- [Sammy Sosa  , 63, 0.288]";


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);

        expect! (receiver, (0, 0, 0), datum, data);

        expect! (receiver, (2, 3, 4), data, node, scalar, r"name");
        expect! (receiver, (2, 3, 5), data, node, scalar, r"hr");
        expect! (receiver, (2, 3, 6), data, node, scalar, r"avg");
        expect! (receiver, (1, 2, 3), node, sequence);

        expect! (receiver, (2, 7, 8), data, node, scalar, r"Mark McGwire");
        expect! (receiver, (2, 7, 9), data, node, scalar, r"65");
        expect! (receiver, (2, 7, 10), data, node, scalar, r"0.278");
        expect! (receiver, (1, 2, 7), node, sequence);

        expect! (receiver, (2, 11, 12), data, node, scalar, r"Sammy Sosa");
        expect! (receiver, (2, 11, 13), data, node, scalar, r"63");
        expect! (receiver, (2, 11, 14), data, node, scalar, r"0.288");
        expect! (receiver, (1, 2, 11), node, sequence);

        expect! (receiver, (0, 0, 15), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_02_06 () {
        let src =
r"Mark McGwire: {hr: 65, avg: 0.278}
Sammy Sosa: {
    hr: 63,
    avg: 0.288
  }";


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"Mark McGwire");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (2, 4, 5), data, node, scalar, r"hr");
        expect! (receiver, (2, 4, 6), data, node, scalar, r"65");
        expect! (receiver, (2, 4, 7), data, node, scalar, r"avg");
        expect! (receiver, (2, 4, 8), data, node, scalar, r"0.278");
        expect! (receiver, (1, 3, 4), data, node, mapping);
        expect! (receiver, (1, 3, 9), data, node, scalar, r"Sammy Sosa");
        expect! (receiver, (2, 10, 11), data, node, scalar, r"hr");
        expect! (receiver, (2, 10, 12), data, node, scalar, r"63");
        expect! (receiver, (2, 10, 13), data, node, scalar, r"avg");
        expect! (receiver, (2, 10, 14), data, node, scalar, r"0.288");
        expect! (receiver, (1, 3, 10), data, node, mapping);
        expect! (receiver, (0, 0, 15), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"Mark McGwire");
        expect! (receiver, (1, 2, 4), data, node, scalar, r"Sammy Sosa");
        expect! (receiver, (1, 2, 5), data, node, scalar, r"Ken Griffey");
        expect! (receiver, (0, 0, 6), doc, end);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"Chicago Cubs");
        expect! (receiver, (1, 2, 4), data, node, scalar, r"St Louis Cardinals");
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"time");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, block, open);
        expect! (receiver, (2, 4, 5), data, literal, r"20");
        expect! (receiver, (2, 4, 6), data, literal, r":");
        expect! (receiver, (2, 4, 7), data, literal, r"03");
        expect! (receiver, (2, 4, 8), data, literal, r":");
        expect! (receiver, (2, 4, 9), data, literal, r"20");
        expect! (receiver, (1, 3, 4), node, block, close);
        expect! (receiver, (1, 3, 10), data, node, scalar, r"player");
        expect! (receiver, (1, 3, 11), data, node, scalar, r"Sammy Sosa");
        expect! (receiver, (1, 3, 12), data, node, scalar, r"action");
        expect! (receiver, (1, 3, 13), data, node, scalar, r"strike (miss)");
        expect! (receiver, (0, 0, 14), doc, end);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"time");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, block, open);
        expect! (receiver, (2, 4, 5), data, literal, r"20");
        expect! (receiver, (2, 4, 6), data, literal, r":");
        expect! (receiver, (2, 4, 7), data, literal, r"03");
        expect! (receiver, (2, 4, 8), data, literal, r":");
        expect! (receiver, (2, 4, 9), data, literal, r"47");
        expect! (receiver, (1, 3, 4), node, block, close);
        expect! (receiver, (1, 3, 10), data, node, scalar, r"player");
        expect! (receiver, (1, 3, 11), data, node, scalar, r"Sammy Sosa");
        expect! (receiver, (1, 3, 12), data, node, scalar, r"action");
        expect! (receiver, (1, 3, 13), data, node, scalar, r"grand slam");
        expect! (receiver, (0, 0, 14), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"hr");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, sequence);
        expect! (receiver, (2, 4, 5), data, node, scalar, r"Mark McGwire");
        expect! (receiver, (2, 4, 6), data, node, scalar, r"Sammy Sosa");
        expect! (receiver, (1, 3, 7), data, node, scalar, r"rbi");
        expect! (receiver, (1, 3, 8), node, sequence);
        expect! (receiver, (2, 8, 9), data, node, scalar, r"Sammy Sosa");
        expect! (receiver, (2, 8, 10), data, node, scalar, r"Ken Griffey");
        expect! (receiver, (0, 0, 11), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"hr");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, sequence);
        expect! (receiver, (2, 4, 5), data, node, scalar, r"Mark McGwire");
        expect! (receiver, (2, 4, 6), data, node, scalar, r"Sammy Sosa", &=r"SS");
        expect! (receiver, (1, 3, 7), data, node, scalar, r"rbi");
        expect! (receiver, (1, 3, 8), node, sequence);
        expect! (receiver, (2, 8, 9), data, alias, r"SS");
        expect! (receiver, (2, 8, 10), data, node, scalar, r"Ken Griffey");
        expect! (receiver, (0, 0, 11), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), data, node, mapping);
        expect! (receiver, (1, 2, 3), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar, r"Detroit Tigers");
        expect! (receiver, (2, 3, 5), data, node, scalar, r"Chicago cubs");
        expect! (receiver, (1, 2, 6), node, sequence);
        expect! (receiver, (2, 6, 7), data, node, scalar, r"2001-07-23");
        expect! (receiver, (2, 8, 9), data, node, scalar, r"New York Yankees");
        expect! (receiver, (2, 8, 10), data, node, scalar, r"Atlanta Braves");
        expect! (receiver, (1, 2, 8), node, sequence);
        expect! (receiver, (2, 11, 12), data, node, scalar, r"2001-07-02");
        expect! (receiver, (2, 11, 13), data, node, scalar, r"2001-08-12");
        expect! (receiver, (2, 11, 14), data, node, scalar, r"2001-08-14");
        expect! (receiver, (1, 2, 11), node, sequence);
        expect! (receiver, (0, 0, 15), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"item");
        expect! (receiver, (1, 2, 4), data, block, map, (1, 2, 3));
        expect! (receiver, (2, 4, 5), data, node, scalar, r"Super Hoop");
        expect! (receiver, (2, 4, 6), data, node, scalar, r"quantity");
        expect! (receiver, (2, 4, 7), data, node, scalar, r"1");
        expect! (receiver, (1, 2, 8), data, node, scalar, r"item");
        expect! (receiver, (1, 2, 9), data, block, map, (1, 2, 8));
        expect! (receiver, (2, 9, 10), data, node, scalar, r"Basketball");
        expect! (receiver, (2, 9, 11), data, node, scalar, r"quantity");
        expect! (receiver, (2, 9, 12), data, node, scalar, r"4");
        expect! (receiver, (1, 2, 13), data, node, scalar, r"item");
        expect! (receiver, (1, 2, 14), data, block, map, (1, 2, 13));
        expect! (receiver, (2, 14, 15), data, node, scalar, r"Big Shoes");
        expect! (receiver, (2, 14, 16), data, node, scalar, r"quantity");
        expect! (receiver, (2, 14, 17), data, node, scalar, r"1");
        expect! (receiver, (0, 0, 18), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_02_13 () {
        let src =
r"# ASCII Art
--- |
  \//||\/||
  // ||  ||__";


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, r"\//||\/||");
        expect! (receiver, (1, 2, 4), data, literal, "\n");
        expect! (receiver, (1, 2, 5), data, literal, r"// ||  ||__");
        expect! (receiver, (0, 0, 2), node, block, close);
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_02_14 () {
        let src =
r"--- >
  Mark McGwire's
  year was crippled
  by a knee injury.";


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, r"Mark McGwire's");
        expect! (receiver, (1, 2, 4), data, literal, r" ");
        expect! (receiver, (1, 2, 5), data, literal, r"year was crippled");
        expect! (receiver, (1, 2, 6), data, literal, r" ");
        expect! (receiver, (1, 2, 7), data, literal, r"by a knee injury.");
        expect! (receiver, (0, 0, 2), node, block, close);
        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, r"Sammy Sosa completed another");
        expect! (receiver, (1, 2, 4), data, literal, r" ");
        expect! (receiver, (1, 2, 5), data, literal, r"fine season with great stats.");
        expect! (receiver, (1, 2, 6), data, literal, "\n");
        expect! (receiver, (1, 2, 7), data, literal, "\n");
        expect! (receiver, (1, 2, 8), data, literal, r"  63 Home Runs");
        expect! (receiver, (1, 2, 9), data, literal, "\n");
        expect! (receiver, (1, 2, 10), data, literal, "  0.288 Batting Average");
        expect! (receiver, (1, 2, 11), data, literal, "\n");
        expect! (receiver, (1, 2, 12), data, literal, "\n");
        expect! (receiver, (1, 2, 13), data, literal, "What a year!");
        expect! (receiver, (0, 0, 2), node, block, close);
        expect! (receiver, (0, 0, 14), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"name");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"Mark McGwire");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"accomplishment");
        expect! (receiver, (1, 3, 6), node, block, open);
        expect! (receiver, (2, 6, 7), data, literal, r"Mark set a major league");
        expect! (receiver, (2, 6, 8), data, literal, r" ");
        expect! (receiver, (2, 6, 9), data, literal, r"home run record in 1998.");
        expect! (receiver, (2, 6, 10), data, literal, "\n");
        expect! (receiver, (1, 3, 6), node, block, close);
        expect! (receiver, (1, 3, 11), data, node, scalar, r"stats");
        expect! (receiver, (1, 3, 12), node, block, open);
        expect! (receiver, (2, 12, 13), data, literal, r"65 Home Runs");
        expect! (receiver, (2, 12, 14), data, literal, "\n");
        expect! (receiver, (2, 12, 15), data, literal, r"0.278 Batting Average");
        expect! (receiver, (1, 3, 12), node, block, close);
        expect! (receiver, (0, 0, 16), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"unicode");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r#""Sosa did fine.\u263A""#);
        expect! (receiver, (1, 3, 5), data, node, scalar, r"control");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""\b1998\t1999\t2000\n""#);
        expect! (receiver, (1, 3, 7), data, node, scalar, r"hex esc");
        expect! (receiver, (1, 3, 8), data, node, scalar, r#""\x0d\x0a is \r\n""#);

        expect! (receiver, (1, 3, 9), data, node, scalar, r"single");
        expect! (receiver, (1, 3, 10), data, node, scalar, r#"'"Howdy!" he cried.'"#);
        expect! (receiver, (1, 3, 11), data, node, scalar, r"quoted");
        expect! (receiver, (1, 3, 12), data, node, scalar, r#"' # Not a ''comment''.'"#);
        expect! (receiver, (1, 3, 13), data, node, scalar, r"tie-fighter");
        expect! (receiver, (1, 3, 14), data, node, scalar, r"'|\-*-/|'");
        expect! (receiver, (0, 0, 15), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_02_18 () {
        let src =
r#"plain:
  This unquoted scalar
  spans many lines.

quoted: "So does this
  quoted scalar.\n""#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"plain");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, block, open);
        expect! (receiver, (2, 4, 5), data, literal, r"This unquoted scalar");
        expect! (receiver, (2, 4, 6), data, literal, r" ");
        expect! (receiver, (2, 4, 7), data, literal, r"spans many lines.");
        expect! (receiver, (1, 3, 4), node, block, close);
        expect! (receiver, (1, 3, 8), data, node, scalar, r"quoted");
        expect! (receiver, (1, 3, 9), data, node, scalar, "\"So does this\n  quoted scalar.\\n\"");
        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_02_19 () {
        let src =
r"canonical: 12345
decimal: +12345
octal: 0o14
hexadecimal: 0xC";

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"canonical");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"12345");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"decimal");
        expect! (receiver, (1, 3, 6), data, node, scalar, r"+12345");
        expect! (receiver, (1, 3, 7), data, node, scalar, r"octal");
        expect! (receiver, (1, 3, 8), data, node, scalar, r"0o14");
        expect! (receiver, (1, 3, 9), data, node, scalar, r"hexadecimal");
        expect! (receiver, (1, 3, 10), data, node, scalar, r"0xC");
        expect! (receiver, (0, 0, 11), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_02_20 () {
        let src =
r"canonical: 1.23015e+3
exponential: 12.3015e+02
fixed: 1230.15
negative infinity: -.inf
not a number: .NaN";


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"canonical");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"1.23015e+3");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"exponential");
        expect! (receiver, (1, 3, 6), data, node, scalar, r"12.3015e+02");
        expect! (receiver, (1, 3, 7), data, node, scalar, r"fixed");
        expect! (receiver, (1, 3, 8), data, node, scalar, r"1230.15");
        expect! (receiver, (1, 3, 9), data, node, scalar, r"negative infinity");
        expect! (receiver, (1, 3, 10), data, node, scalar, r"-.inf");
        expect! (receiver, (1, 3, 11), data, node, scalar, r"not a number");
        expect! (receiver, (1, 3, 12), data, node, scalar, r".NaN");
        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_02_21 () {
        let src =
r"null:
booleans: [ true, false ]
string: '012345'";


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"null");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, null);
        expect! (receiver, (1, 3, 5), data, node, scalar, r"booleans");
        expect! (receiver, (2, 6, 7), data, node, scalar, r"true");
        expect! (receiver, (2, 6, 8), data, node, scalar, r"false");
        expect! (receiver, (1, 3, 6), node, sequence);
        expect! (receiver, (1, 3, 9), data, node, scalar, r"string");
        expect! (receiver, (1, 3, 10), data, node, scalar, r"'012345'");
        expect! (receiver, (0, 0, 11), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_02_22 () {
        let src =
r"canonical: 2001-12-15T02:59:43.1Z
iso8601: 2001-12-14t21:59:43.10-05:00
spaced: 2001-12-14 21:59:43.10 -5
date: 2002-12-14";


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"canonical");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, block, open);
        expect! (receiver, (2, 4, 5), data, literal, r"2001-12-15T02");
        expect! (receiver, (2, 4, 6), data, literal, r":");
        expect! (receiver, (2, 4, 7), data, literal, r"59");
        expect! (receiver, (2, 4, 8), data, literal, r":");
        expect! (receiver, (2, 4, 9), data, literal, r"43.1Z");
        expect! (receiver, (1, 3, 4), node, block, close);
        expect! (receiver, (1, 3, 10), data, node, scalar, r"iso8601");
        expect! (receiver, (1, 3, 11), node, block, open);
        expect! (receiver, (2, 11, 12), data, literal, r"2001-12-14t21");
        expect! (receiver, (2, 11, 13), data, literal, r":");
        expect! (receiver, (2, 11, 14), data, literal, r"59");
        expect! (receiver, (2, 11, 15), data, literal, r":");
        expect! (receiver, (2, 11, 16), data, literal, r"43.10-05");
        expect! (receiver, (2, 11, 17), data, literal, r":");
        expect! (receiver, (2, 11, 18), data, literal, r"00");
        expect! (receiver, (1, 3, 11), node, block, close);
        expect! (receiver, (1, 3, 19), data, node, scalar, r"spaced");
        expect! (receiver, (1, 3, 20), node, block, open);
        expect! (receiver, (2, 20, 21), data, literal, r"2001-12-14 21");
        expect! (receiver, (2, 20, 22), data, literal, r":");
        expect! (receiver, (2, 20, 23), data, literal, r"59");
        expect! (receiver, (2, 20, 24), data, literal, r":");
        expect! (receiver, (2, 20, 25), data, literal, r"43.10 -5");
        expect! (receiver, (1, 3, 20), node, block, close);
        expect! (receiver, (1, 3, 26), data, node, scalar, r"date");
        expect! (receiver, (1, 3, 27), data, node, scalar, r"2002-12-14");
        expect! (receiver, (0, 0, 28), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"not-date");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"2002-04-28", !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"picture");
        expect! (receiver, (1, 3, 6), node, block, open);
        expect! (receiver, (2, 6, 7), data, literal, r"R0lGODlhDAAMAIQAAP//9/X");
        expect! (receiver, (2, 6, 8), data, literal, "\n");
        expect! (receiver, (2, 6, 9), data, literal, r"17unp5WZmZgAAAOfn515eXv");
        expect! (receiver, (2, 6, 10), data, literal, "\n");
        expect! (receiver, (2, 6, 11), data, literal, r"Pz7Y6OjuDg4J+fn5OTk6enp");
        expect! (receiver, (2, 6, 12), data, literal, "\n");
        expect! (receiver, (2, 6, 13), data, literal, r"56enmleECcgggoBADs=");
        expect! (receiver, (2, 6, 14), data, literal, "\n");
        expect! (receiver, (1, 3, 6), data, node, block, close, !=r"!!binary");

        expect! (receiver, (1, 3, 15), data, node, scalar, r"application specific tag");
        expect! (receiver, (1, 3, 16), node, block, open);
        expect! (receiver, (2, 16, 17), data, literal, r"The semantics of the tag");
        expect! (receiver, (2, 16, 18), data, literal, "\n");
        expect! (receiver, (2, 16, 19), data, literal, r"above may be different for");
        expect! (receiver, (2, 16, 20), data, literal, "\n");
        expect! (receiver, (2, 16, 21), data, literal, r"different documents.");
        expect! (receiver, (2, 16, 22), data, literal, "\n");
        expect! (receiver, (1, 3, 16), data, node, block, close, !=r"!something");
        expect! (receiver, (0, 0, 23), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), data, dir, tag, r"!", r"tag:clarkevans.com,2002:");
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!shape");
        expect! (receiver, (1, 3, 4), data, node, scalar, r"center");
        expect! (receiver, (1, 3, 5), data, block, map, (1, 3, 4), !=r"!circle");

        expect! (receiver, (3, 6, 7), data, node, scalar, r"x");
        expect! (receiver, (3, 6, 8), data, node, scalar, r"73");
        expect! (receiver, (3, 6, 9), data, node, scalar, r"y");
        expect! (receiver, (3, 6, 10), data, node, scalar, r"129");
        expect! (receiver, (2, 5, 6), data, node, mapping, &=r"ORIGIN");

        expect! (receiver, (2, 5, 11), data, node, scalar, r"radius");
        expect! (receiver, (2, 5, 12), data, node, scalar, r"7");
        expect! (receiver, (1, 3, 13), data, node, scalar, r"start");
        expect! (receiver, (1, 3, 14), data, block, map, (1, 3, 13), !=r"!line");
        expect! (receiver, (2, 14, 15), data, alias, r"ORIGIN");
        expect! (receiver, (2, 14, 16), data, node, scalar, r"finish");

        expect! (receiver, (3, 17, 18), data, node, scalar, r"x");
        expect! (receiver, (3, 17, 19), data, node, scalar, r"89");
        expect! (receiver, (3, 17, 20), data, node, scalar, r"y");
        expect! (receiver, (3, 17, 21), data, node, scalar, r"102");
        expect! (receiver, (2, 14, 17), data, node, mapping);

        expect! (receiver, (1, 3, 22), data, node, scalar, r"start");
        expect! (receiver, (1, 3, 23), data, block, map, (1, 3, 22), !=r"!label");
        expect! (receiver, (2, 23, 24), data, alias, r"ORIGIN");
        expect! (receiver, (2, 23, 25), data, node, scalar, r"color");
        expect! (receiver, (2, 23, 26), data, node, scalar, r"0xFFEEBB");
        expect! (receiver, (2, 23, 27), data, node, scalar, r"text");
        expect! (receiver, (2, 23, 28), data, node, scalar, r"Pretty vector drawing.");
        expect! (receiver, (0, 0, 29), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, mapping, !=r"!!set");
        expect! (receiver, (1, 2, 3), data, node, scalar, r"Mark McGwire");
        expect! (receiver, (1, 2, 4), node, null);
        expect! (receiver, (1, 2, 5), data, node, scalar, r"Sammy Sosa");
        expect! (receiver, (1, 2, 6), node, null);
        expect! (receiver, (1, 2, 7), data, node, scalar, r"Ken Griff");
        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, sequence, !=r"!!omap");
        expect! (receiver, (1, 2, 3), data, node, scalar, r"Mark McGwire");
        expect! (receiver, (1, 2, 4), data, block, map, (1, 2, 3));
        expect! (receiver, (2, 4, 5), data, node, scalar, r"65");
        expect! (receiver, (1, 2, 6), data, node, scalar, r"Sammy Sosa");
        expect! (receiver, (1, 2, 7), data, block, map, (1, 2, 6));
        expect! (receiver, (2, 7, 8), data, node, scalar, r"63");
        expect! (receiver, (1, 2, 9), data, node, scalar, r"Ken Griffy");
        expect! (receiver, (1, 2, 10), data, block, map, (1, 2, 9));
        expect! (receiver, (2, 10, 11), data, node, scalar, r"58");
        expect! (receiver, (0, 0, 12), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"invoice");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2), !=r"!<tag:clarkevans.com,2002:invoice>");
        expect! (receiver, (1, 3, 4), data, node, scalar, r"34843");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"date");
        expect! (receiver, (1, 3, 6), data, node, scalar, r"2001-01-23");
        expect! (receiver, (1, 3, 7), data, node, scalar, r"bill-to");
        expect! (receiver, (1, 3, 8), data, node, scalar, r"given");
        expect! (receiver, (1, 3, 9), data, block, map, (1, 3, 8), &=r"id001");
        expect! (receiver, (2, 9, 10), data, node, scalar, r"Chris");
        expect! (receiver, (2, 9, 11), data, node, scalar, r"family");
        expect! (receiver, (2, 9, 12), data, node, scalar, r"Dumars");
        expect! (receiver, (2, 9, 13), data, node, scalar, r"address");
        expect! (receiver, (2, 9, 14), data, node, scalar, r"lines");
        expect! (receiver, (2, 9, 15), data, block, map, (2, 9, 14));
        expect! (receiver, (3, 15, 16), node, block, open);
        expect! (receiver, (4, 16, 17), data, literal, r"458 Walkman Dr.");
        expect! (receiver, (4, 16, 18), data, literal, "\n");
        expect! (receiver, (4, 16, 19), data, literal, r"Suite #292");
        expect! (receiver, (4, 16, 20), data, literal, "\n");
        expect! (receiver, (3, 15, 16), node, block, close);
        expect! (receiver, (3, 15, 21), data, node, scalar, r"city");
        expect! (receiver, (3, 15, 22), data, node, scalar, r"Royal Oak");
        expect! (receiver, (3, 15, 23), data, node, scalar, r"state");
        expect! (receiver, (3, 15, 24), data, node, scalar, r"MI");
        expect! (receiver, (3, 15, 25), data, node, scalar, r"postal");
        expect! (receiver, (3, 15, 26), data, node, scalar, r"48046");
        expect! (receiver, (1, 3, 27), data, node, scalar, r"ship-to");
        expect! (receiver, (1, 3, 28), data, alias, r"id001");
        expect! (receiver, (1, 3, 29), data, node, scalar, r"product");
        expect! (receiver, (1, 3, 30), node, sequence);
        expect! (receiver, (2, 30, 31), data, node, scalar, r"sku");
        expect! (receiver, (2, 30, 32), data, block, map, (2, 30, 31));
        expect! (receiver, (3, 32, 33), data, node, scalar, r"BL394D");
        expect! (receiver, (3, 32, 34), data, node, scalar, r"quantity");
        expect! (receiver, (3, 32, 35), data, node, scalar, r"4");
        expect! (receiver, (3, 32, 36), data, node, scalar, r"description");
        expect! (receiver, (3, 32, 37), data, node, scalar, r"Basketball");
        expect! (receiver, (3, 32, 38), data, node, scalar, r"price");
        expect! (receiver, (3, 32, 39), data, node, scalar, r"450.00");
        expect! (receiver, (2, 30, 40), data, node, scalar, r"sku");
        expect! (receiver, (2, 30, 41), data, block, map, (2, 30, 40));
        expect! (receiver, (3, 41, 42), data, node, scalar, r"BL4438H");
        expect! (receiver, (3, 41, 43), data, node, scalar, r"quantity");
        expect! (receiver, (3, 41, 44), data, node, scalar, r"1");
        expect! (receiver, (3, 41, 45), data, node, scalar, r"description");
        expect! (receiver, (3, 41, 46), data, node, scalar, r"Super Hoop");
        expect! (receiver, (3, 41, 47), data, node, scalar, r"price");
        expect! (receiver, (3, 41, 48), data, node, scalar, r"2392.00");
        expect! (receiver, (1, 3, 49), data, node, scalar, r"tax");
        expect! (receiver, (1, 3, 50), data, node, scalar, r"251.42");
        expect! (receiver, (1, 3, 51), data, node, scalar, r"total");
        expect! (receiver, (1, 3, 52), data, node, scalar, r"4443.52");
        expect! (receiver, (1, 3, 53), data, node, scalar, r"comments");
        expect! (receiver, (1, 3, 54), node, block, open);
        expect! (receiver, (2, 54, 55), data, literal, r"Late afternoon is best.");
        expect! (receiver, (2, 54, 56), data, literal, r" ");
        expect! (receiver, (2, 54, 57), data, literal, r"Backup contact is Nancy");
        expect! (receiver, (2, 54, 58), data, literal, r" ");
        expect! (receiver, (2, 54, 59), data, literal, r"Billsmer @ 338-4338.");
        expect! (receiver, (1, 3, 54), node, block, close);
        expect! (receiver, (0, 0, 60), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"Time");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, block, open);
        expect! (receiver, (2, 4, 5), data, literal, r"2001-11-23 15");
        expect! (receiver, (2, 4, 6), data, literal, r":");
        expect! (receiver, (2, 4, 7), data, literal, r"01");
        expect! (receiver, (2, 4, 8), data, literal, r":");
        expect! (receiver, (2, 4, 9), data, literal, r"42 -5");
        expect! (receiver, (1, 3, 4), node, block, close);
        expect! (receiver, (1, 3, 10), data, node, scalar, r"User");
        expect! (receiver, (1, 3, 11), data, node, scalar, r"ed");
        expect! (receiver, (1, 3, 12), data, node, scalar, r"Warning");
        expect! (receiver, (1, 3, 13), node, block, open);
        expect! (receiver, (2, 13, 14), data, literal, r"This is an error message");
        expect! (receiver, (2, 13, 15), data, literal, r" ");
        expect! (receiver, (2, 13, 16), data, literal, r"for the log file");
        expect! (receiver, (1, 3, 13), node, block, close);
        expect! (receiver, (0, 0, 17), doc, end);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"Time");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, block, open);
        expect! (receiver, (2, 4, 5), data, literal, r"2001-11-23 15");
        expect! (receiver, (2, 4, 6), data, literal, r":");
        expect! (receiver, (2, 4, 7), data, literal, r"02");
        expect! (receiver, (2, 4, 8), data, literal, r":");
        expect! (receiver, (2, 4, 9), data, literal, r"31 -5");
        expect! (receiver, (1, 3, 4), node, block, close);
        expect! (receiver, (1, 3, 10), data, node, scalar, r"User");
        expect! (receiver, (1, 3, 11), data, node, scalar, r"ed");
        expect! (receiver, (1, 3, 12), data, node, scalar, r"Warning");
        expect! (receiver, (1, 3, 13), node, block, open);
        expect! (receiver, (2, 13, 14), data, literal, r"A slightly different error");
        expect! (receiver, (2, 13, 15), data, literal, r" ");
        expect! (receiver, (2, 13, 16), data, literal, r"message.");
        expect! (receiver, (1, 3, 13), node, block, close);
        expect! (receiver, (0, 0, 17), doc, end);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"Date");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, block, open);
        expect! (receiver, (2, 4, 5), data, literal, r"2001-11-23 15");
        expect! (receiver, (2, 4, 6), data, literal, r":");
        expect! (receiver, (2, 4, 7), data, literal, r"03");
        expect! (receiver, (2, 4, 8), data, literal, r":");
        expect! (receiver, (2, 4, 9), data, literal, r"17 -5");
        expect! (receiver, (1, 3, 4), node, block, close);
        expect! (receiver, (1, 3, 10), data, node, scalar, r"User");
        expect! (receiver, (1, 3, 11), data, node, scalar, r"ed");
        expect! (receiver, (1, 3, 12), data, node, scalar, r"Fatal");
        expect! (receiver, (1, 3, 13), data, node, scalar, "Unknown variable \"bar\"");
        expect! (receiver, (1, 3, 14), data, node, scalar, r"Stack");
        expect! (receiver, (1, 3, 15), node, sequence);
        expect! (receiver, (2, 15, 16), data, node, scalar, r"file");
        expect! (receiver, (2, 15, 17), data, block, map, (2, 15, 16));
        expect! (receiver, (3, 17, 18), data, node, scalar, r"TopClass.py");
        expect! (receiver, (3, 17, 19), data, node, scalar, r"line");
        expect! (receiver, (3, 17, 20), data, node, scalar, r"23");
        expect! (receiver, (3, 17, 21), data, node, scalar, r"code");
        expect! (receiver, (3, 17, 22), node, block, open);
        expect! (receiver, (4, 22, 23), data, literal, r#"x = MoreObject("345\n")"#);
        expect! (receiver, (4, 22, 24), data, literal, "\n");
        expect! (receiver, (3, 17, 22), node, block, close);
        expect! (receiver, (2, 15, 25), data, node, scalar, r"file");
        expect! (receiver, (2, 15, 26), data, block, map, (2, 15, 25));
        expect! (receiver, (3, 26, 27), data, node, scalar, r"MoreClass.py");
        expect! (receiver, (3, 26, 28), data, node, scalar, r"line");
        expect! (receiver, (3, 26, 29), data, node, scalar, r"58");
        expect! (receiver, (3, 26, 30), data, node, scalar, r"code");
        expect! (receiver, (3, 26, 31), node, block, open);
        expect! (receiver, (4, 31, 32), data, literal, r"foo = bar");
        expect! (receiver, (3, 26, 31), node, block, close);
        expect! (receiver, (0, 0, 33), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"baseball players");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, mapping, !=r"!!set");
        expect! (receiver, (2, 4, 5), data, node, scalar, r"Mark McGwire");
        expect! (receiver, (2, 4, 6), node, null);
        expect! (receiver, (2, 4, 7), data, node, scalar, r"Sammy Sosa");
        expect! (receiver, (2, 4, 8), node, null);
        expect! (receiver, (2, 4, 9), data, node, scalar, r"Ken Griffey");

        expect! (receiver, (1, 3, 10), data, node, scalar, r"baseball teams");
        
        expect! (receiver, (2, 11, 12), data, node, scalar, r"Boston Red Sox");
        expect! (receiver, (2, 11, 13), node, null);
        expect! (receiver, (2, 11, 14), data, node, scalar, r"Detroit Tigers");
        expect! (receiver, (2, 11, 15), node, null);
        expect! (receiver, (2, 11, 16), data, node, scalar, r"New York Yankees");
        expect! (receiver, (2, 11, 17), node, null);

        expect! (receiver, (1, 3, 11), data, node, mapping, !=r"!!set");
        expect! (receiver, (0, 0, 18), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"link with");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, sequence);
        expect! (receiver, (2, 4, 5), data, node, scalar, r"library1.dll");
        expect! (receiver, (2, 4, 6), data, node, scalar, r"library2.dll");
        expect! (receiver, (0, 0, 7), doc, end);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"link with");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, sequence);
        expect! (receiver, (2, 4, 5), data, node, scalar, r"=");
        expect! (receiver, (2, 4, 6), data, block, map, (2, 4, 5));
        expect! (receiver, (3, 6, 7), data, node, scalar, r"library1.dll");
        expect! (receiver, (3, 6, 8), data, node, scalar, r"version");
        expect! (receiver, (3, 6, 9), data, node, scalar, r"1.2");
        expect! (receiver, (2, 4, 10), data, node, scalar, r"=");
        expect! (receiver, (2, 4, 11), data, block, map, (2, 4, 10));
        expect! (receiver, (3, 11, 12), data, node, scalar, r"library2.dll");
        expect! (receiver, (3, 11, 13), data, node, scalar, r"version");
        expect! (receiver, (3, 11, 14), data, node, scalar, r"2.3");
        expect! (receiver, (0, 0, 15), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_05_01 () {
        let src =
b"\xEF\xBB\xBF# Comment only.";

        let receiver = read_bytes! (src);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_05_02 () {
        let src =
b"- Invalid use of BOM
\xEF\xBB\xBF
- Inside a document.";

        let receiver = read_bytes! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"Invalid use of BOM");
        expect! (receiver, (1, 2, 4), data, node, scalar, r"Inside a document.");
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"sequence");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, sequence);
        expect! (receiver, (2, 4, 5), data, node, scalar, r"one");
        expect! (receiver, (2, 4, 6), data, node, scalar, r"two");
        expect! (receiver, (1, 3, 7), data, node, scalar, r"mapping");
        expect! (receiver, (1, 3, 8), data, node, mapping);
        expect! (receiver, (2, 8, 9), data, node, scalar, r"sky");
        expect! (receiver, (2, 8, 10), data, node, scalar, r"blue");
        expect! (receiver, (2, 8, 11), data, node, scalar, r"sea");
        expect! (receiver, (2, 8, 12), data, node, scalar, r"green");
        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1,2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (1, 3, 4), data, node, scalar, r#""sequence""#, !=r"!!str");
        expect! (receiver, (2, 5, 6), data, node, scalar, r#""one""#, !=r"!!str");
        expect! (receiver, (2, 5, 7), data, node, scalar, r#""two""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, sequence, !=r"!!seq");
        expect! (receiver, (1, 3, 8), data, node, scalar, r#""mapping""#, !=r"!!str");
        expect! (receiver, (2, 9, 10), data, node, scalar, r#""sky""#, !=r"!!str");
        expect! (receiver, (2, 9, 11), data, node, scalar, r#""blue""#, !=r"!!str");
        expect! (receiver, (2, 9, 12), data, node, scalar, r#""sea""#, !=r"!!str");
        expect! (receiver, (2, 9, 13), data, node, scalar, r#""green""#, !=r"!!str");
        expect! (receiver, (1, 3, 9), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 14), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_05_04 () {
        let src =
"sequence: [ one, two, ]
mapping: { sky: blue, sea: green }";

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"sequence");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));

        expect! (receiver, (2, 4, 5), data, node, scalar, r"one");
        expect! (receiver, (2, 4, 6), data, node, scalar, r"two");
        expect! (receiver, (1, 3, 4), node, sequence);

        expect! (receiver, (1, 3, 7), data, node, scalar, r"mapping");

        expect! (receiver, (2, 8, 9), data, node, scalar, r"sky");
        expect! (receiver, (2, 8, 10), data, node, scalar, r"blue");
        expect! (receiver, (2, 8, 11), data, node, scalar, r"sea");
        expect! (receiver, (2, 8, 12), data, node, scalar, r"green");
        expect! (receiver, (1, 3, 8), data, node, mapping);

        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_05_05 () {
        let src = "# Comment only.";

        let receiver = read! (src);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_05_06 () {
        let src =
"anchored: !local &anchor value
alias: *anchor";

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"anchored");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"value", !=r"!local", &=r"anchor");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"alias");
        expect! (receiver, (1, 3, 6), data, alias, r"anchor");
        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""anchored""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""value""#, !=r"!local", &=r"A1");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""alias""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, alias, r"A1");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"literal");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, block, open);
        expect! (receiver, (2, 4, 5), data, literal, r"some");
        expect! (receiver, (2, 4, 6), data, literal, "\n");
        expect! (receiver, (2, 4, 7), data, literal, r"text");
        expect! (receiver, (2, 4, 8), data, literal, "\n");
        expect! (receiver, (1, 3, 4), node, block, close);
        expect! (receiver, (1, 3, 9), data, node, scalar, r"folded");
        expect! (receiver, (1, 3, 10), node, block, open);
        expect! (receiver, (2, 10, 11), data, literal, r"some");
        expect! (receiver, (2, 10, 12), data, literal, r" ");
        expect! (receiver, (2, 10, 13), data, literal, r"text");
        expect! (receiver, (2, 10, 14), data, literal, "\n");
        expect! (receiver, (1, 3, 10), node, block, close);
        expect! (receiver, (0, 0, 15), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""literal""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""some\ntext\n""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""folded""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""some text\n""#, !=r"!!str");
        expect! (receiver, (0, 0, 3), data, node, mapping, !="!!map");

        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_05_08 () {
        let src =
r#"single: 'text'
double: "text""#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"single");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"'text'");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"double");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""text""#);

        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""single""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""text""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""double""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""text""#, !=r"!!str");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 8), doc, end);
        the_end! (receiver);
    }



    #[test]
    fn example_05_09 () {
        let src =
"%YAML 1.2
--- text";

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r"text");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_05_09_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "text""#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""text""#, !=r"!!str");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_05_10 () {
        let src =
"commercial-at: @text
grave-accent: `text";

        let receiver = read_with_error! (src, r"@ character is reserved and may not be used to start a plain scalar", 15);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"commercial-at");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), error, r"@ character is reserved and may not be used to start a plain scalar", 15);

        the_end! (receiver);
    }



    #[test]
    fn example_05_11 () {
        let src =
"|
  Line break (no glyph)
  Line break (glyphed)
";

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, r"Line break (no glyph)");
        expect! (receiver, (1, 2, 4), data, literal, "\n");
        expect! (receiver, (1, 2, 5), data, literal, r"Line break (glyphed)");
        expect! (receiver, (1, 2, 6), data, literal, "\n");
        expect! (receiver, (0, 0, 2), node, block, close);
        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_05_11_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "line break (no glyph)\n\
      line break (glyphed)\n""#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, "\"line break (no glyph)\\n\\\n      line break (glyphed)\\n\"", !=r"!!str");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"quoted");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r#""Quoted 	""#);
        expect! (receiver, (1, 3, 5), data, node, scalar, r"block");
        expect! (receiver, (1, 3, 6), node, block, open);
        expect! (receiver, (2, 6, 7), data, literal, r"void main() {");
        expect! (receiver, (2, 6, 8), data, literal, "\n");
        expect! (receiver, (2, 6, 9), data, literal, r#"	printf("Hello, world!\n");"#);
        expect! (receiver, (2, 6, 10), data, literal, "\n");
        expect! (receiver, (2, 6, 11), data, literal, r"}");
        expect! (receiver, (1, 3, 6), node, block, close);
        expect! (receiver, (0, 0, 12), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""quoted""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""Quoted \t""#);
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""block""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, "\"void main() {\\n\\\n    \\tprintf(\\\"Hello, world!\\\\n\\\");\\n\\\n    }\\n\"");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_05_13 () {
        let src =
r#""Fun with \\
\" \a \b \e \f \↓
\n \r \t \v \0 \↓
\  \_ \N \L \P \↓
\x41 \u0041 \U00000041""#;

        let receiver = read! (src);
        let mut data = data! ();;

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, "\"Fun with \\\\\n\\\" \\a \\b \\e \\f \\↓\n\\n \\r \\t \\v \\0 \\↓\n\\  \\_ \\N \\L \\P \\↓\n\\x41 \\u0041 \\U00000041\"");
        expect! (receiver, (0, 0, 3), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();;

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, "\"Fun with \\x5C\n\\x22 \\x07 \\x08 \\x1B \\x0C\n\\x0A \\x0D \\x09 \\x0B \\x00\n\\x20 \\xA0 \\x85 \\u2028 \\u2029\nA A A\"");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_05_14 () {
        let src =
r#"Bad escapes:
  "\c
  \xq-""#;

        let receiver = read! (src);
        let mut data = data! ();;

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"Bad escapes");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, "\"\\c\n  \\xq-\"");
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"Not indented");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"By one space");
        expect! (receiver, (1, 3, 5), data, block, map, (1, 3, 4));
        expect! (receiver, (2, 5, 6), node, block, open);
        expect! (receiver, (3, 6, 7), data, literal, r"By four");
        expect! (receiver, (3, 6, 8), data, literal, "\n");
        expect! (receiver, (3, 6, 9), data, literal, r"  spaces");
        expect! (receiver, (3, 6, 10), data, literal, "\n");
        expect! (receiver, (2, 5, 6), node, block, close);
        expect! (receiver, (2, 5, 11), data, node, scalar, r"Flow style");
        expect! (receiver, (3, 12, 13), data, node, scalar, r"By two");
        expect! (receiver, (3, 12, 14), data, node, scalar, r"Also by two");
        expect! (receiver, (3, 12, 15), data, node, scalar, r"Still by two");
        expect! (receiver, (2, 5, 12), node, sequence);
        expect! (receiver, (0, 0, 16), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (0, 0, 3), node, sequence);
        expect! (receiver, (1, 3, 4), node, sequence);
        expect! (receiver, (2, 4, 5), node, sequence);
        expect! (receiver, (3, 5, 6), node, null);

        expect! (receiver, (1, 7, 8), data, node, scalar, "\"Not indented\"", !=r"!!str");

        expect! (receiver, (2, 9, 10), data, node, scalar, r#""By one space""#, !=r"!!str");
        expect! (receiver, (2, 9, 11), data, node, scalar, r#""By four\n  spaces\n""#, !=r"!!str");
        expect! (receiver, (2, 9, 12), data, node, scalar, r#""Flow style""#, !=r"!!str");

        expect! (receiver, (3, 13, 14), data, node, scalar, r#""By two""#, !=r"!!str");
        expect! (receiver, (3, 13, 15), data, node, scalar, r#""Also by two""#, !=r"!!str");
        expect! (receiver, (3, 13, 16), data, node, scalar, r#""Still by two""#, !=r"!!str");

        expect! (receiver, (2, 9, 13), data, node, sequence, !="!!seq");
        expect! (receiver, (1, 7, 9), data, node, mapping, !="!!map");
        expect! (receiver, (0, 0, 7), data, node, mapping, !="!!map");

        expect! (receiver, (0, 0, 17), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_02 () {
        let src =
r"? a
: -	b
  -  -	c
     - d
";

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"a");
        expect! (receiver, (1, 2, 4), node, sequence);
        expect! (receiver, (2, 4, 5), data, node, scalar, r"b");
        expect! (receiver, (2, 4, 6), node, sequence);
        expect! (receiver, (3, 6, 7), data, node, scalar, r"c");
        expect! (receiver, (3, 6, 8), data, node, scalar, r"d");
        expect! (receiver, (0, 0, 9), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""a""#, !=r"!!str");
        expect! (receiver, (2, 5, 6), data, node, scalar, r#""b""#, !=r"!!str");
        expect! (receiver, (3, 7, 8), data, node, scalar, r#""c""#, !=r"!!str");
        expect! (receiver, (3, 7, 9), data, node, scalar, r#""d""#, !=r"!!str");

        expect! (receiver, (2, 5, 7), data, node, sequence, !=r"!!seq");
        expect! (receiver, (1, 3, 5), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_03 () {
        let src =
r#"- foo:	 bar
- - baz
  -	baz"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"foo");
        expect! (receiver, (1, 2, 4), data, block, map, (1, 2, 3));
        expect! (receiver, (2, 4, 5), data, node, scalar, r"bar");
        expect! (receiver, (1, 2, 6), node, sequence);
        expect! (receiver, (2, 6, 7), data, node, scalar, r"baz");
        expect! (receiver, (2, 6, 8), data, node, scalar, r"baz");
        expect! (receiver, (0, 0, 9), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (2, 4, 5), data, node, scalar, r#""foo""#, !=r"!!str");
        expect! (receiver, (2, 4, 6), data, node, scalar, r#""bar""#, !=r"!!str");
        expect! (receiver, (1, 3, 4), data, node, mapping, !=r"!!map");

        expect! (receiver, (2, 7, 8), data, node, scalar, r#""baz""#, !=r"!!str");
        expect! (receiver, (2, 7, 9), data, node, scalar, r#""baz""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, sequence, !=r"!!seq");

        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"plain");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, block, open);
        expect! (receiver, (2, 4, 5), data, literal, r"text");
        expect! (receiver, (2, 4, 6), data, literal, " ");
        expect! (receiver, (2, 4, 7), data, literal, r"lines");
        expect! (receiver, (1, 3, 4), node, block, close);
        expect! (receiver, (1, 3, 8), data, node, scalar, r"quoted");
        expect! (receiver, (1, 3, 9), data, node, scalar, "\"text\n  	lines\"");
        expect! (receiver, (1, 3, 10), data, node, scalar, r"block");
        expect! (receiver, (1, 3, 11), node, block, open);
        expect! (receiver, (2, 11, 12), data, literal, r"text");
        expect! (receiver, (2, 11, 13), data, literal, "\n");
        expect! (receiver, (2, 11, 14), data, literal, " 	lines");
        expect! (receiver, (2, 11, 15), data, literal, "\n");
        expect! (receiver, (1, 3, 11), node, block, close);
        expect! (receiver, (0, 0, 16), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""plain""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""text lines""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""quoted""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""text lines""#, !=r"!!str");
        expect! (receiver, (1, 3, 8), data, node, scalar, r#""block""#, !=r"!!str");
        expect! (receiver, (1, 3, 9), data, node, scalar, r#""text\n 	lines\n""#, !=r"!!str");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"Folding");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, "\"Empty line\n   	\n  as a line feed\"");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"Chomping");
        expect! (receiver, (1, 3, 6), node, block, open);
        expect! (receiver, (2, 6, 7), data, literal, r"Clipped empty lines");
        expect! (receiver, (2, 6, 8), data, literal, "\n");
        expect! (receiver, (1, 3, 6), node, block, close);
        expect! (receiver, (0, 0, 9), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""Folding""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""Empty line\nas a line feed""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""Chomping""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""Clipped empty lines\n""#, !=r"!!str");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_06 () {
        let src =
r#">-
  trimmed
  
 

  as
  space"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, r"trimmed");
        expect! (receiver, (1, 2, 4), data, literal, "\n\n\n");
        expect! (receiver, (1, 2, 5), data, literal, r"as");
        expect! (receiver, (1, 2, 6), data, literal, r" ");
        expect! (receiver, (1, 2, 7), data, literal, r"space");
        expect! (receiver, (0, 0, 2), node, block, close);
        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_06_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "trimmed\n\n\nas space""#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""trimmed\n\n\nas space""#, !=r"!!str");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_07 () {
        let src =
r#">
  foo 
 
  	 bar

  baz
"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, r"foo ");
        expect! (receiver, (1, 2, 4), data, literal, "\n");
        expect! (receiver, (1, 2, 5), data, literal, "\n");
        expect! (receiver, (1, 2, 6), data, literal, "\t bar");
        expect! (receiver, (1, 2, 7), data, literal, "\n");
        expect! (receiver, (1, 2, 8), data, literal, "\n");
        expect! (receiver, (1, 2, 9), data, literal, r"baz");
        expect! (receiver, (1, 2, 10), data, literal, "\n");
        expect! (receiver, (0, 0, 2), node, block, close);
        expect! (receiver, (0, 0, 11), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_07_canonical () {
        let src =
r#"%YAML 1.2
--- !!str
"foo \n\n\t bar\n\nbaz\n""#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""foo \n\n\t bar\n\nbaz\n""#, !=r"!!str");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_08 () {
        let src =
r#""
  foo 
 
  	 bar

  baz
""#; 


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, "\"\n  foo \n \n  	 bar\n\n  baz\n\"");
        expect! (receiver, (0, 0, 3), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_08_canonical () {
        let src =
r#"%YAML 1.2
--- !!str
" foo\nbar\nbaz ""#; 


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#"" foo\nbar\nbaz ""#, !=r"!!str");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_09 () {
        let src =
r#"key:    # Comment
  value"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"key");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"value");

        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""key""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""value""#, !=r"!!str");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_10 () {
        let src =
r#"  # Comment
   

"#;

        let receiver = read! (src);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), doc, end);
        the_end! (receiver);
    }



    #[test]
    fn example_06_11 () {
        let src =
r#"key:    # Comment
        # lines
  value

"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"key");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"value");

        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"first");
        expect! (receiver, (1, 2, 4), data, node, scalar, r"Sammy");
        expect! (receiver, (1, 2, 5), data, node, scalar, r"last");
        expect! (receiver, (1, 2, 6), data, node, scalar, r"Sosa");
        expect! (receiver, (0, 0, 2), data, node, mapping);

        expect! (receiver, (0, 0, 7), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 7, 8), data, node, scalar, r"hr");

        expect! (receiver, (1, 7, 9), data, block, map, (1, 7, 8));
        expect! (receiver, (2, 9, 10), data, node, scalar, r"65");
        expect! (receiver, (2, 9, 11), data, node, scalar, r"avg");
        expect! (receiver, (2, 9, 12), data, node, scalar, r"0.278");

        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (2, 4, 5), data, node, scalar, r#""first""#, !=r"!!str");
        expect! (receiver, (2, 4, 6), data, node, scalar, r#""Sammy""#, !=r"!!str");
        expect! (receiver, (2, 4, 7), data, node, scalar, r#""last""#, !=r"!!str");
        expect! (receiver, (2, 4, 8), data, node, scalar, r#""Sosa""#, !=r"!!str");
        expect! (receiver, (1, 3, 4), data, node, mapping, !=r"!!map");

        expect! (receiver, (2, 9, 10), data, node, scalar, r#""hr""#, !=r"!!str");
        expect! (receiver, (2, 9, 11), data, node, scalar, r#""65""#, !=r"!!int");
        expect! (receiver, (2, 9, 12), data, node, scalar, r#""avg""#, !=r"!!str");
        expect! (receiver, (2, 9, 13), data, node, scalar, r#""0.278""#, !=r"!!float");

        expect! (receiver, (1, 3, 9), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 14), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_13 () {
        let src =
r#"%FOO  bar baz # Should be ignored
               # with a warning.
--- "foo""#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), warning, "Unknown directive at the line 0", 0);
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""foo""#);
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_14 () {
        let src =
r#"%YAML 1.3 # Attempt parsing
           # with a warning
---
"foo""#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), warning, "%YAML minor version is not fully supported", 10);
        expect! (receiver, (0, 0, 2), dir, yaml, (1, 3));
        expect! (receiver, (0, 0, 3), doc, start);
        expect! (receiver, (0, 0, 4), data, node, scalar, r#""foo""#);
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_15 () {
        let src =
r#"%YAML 1.2
%YAML 1.1
foo"#;


        let receiver = read_with_error! (src, r"The YAML directive must only be given at most once per document", 10);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), error, r"The YAML directive must only be given at most once per document", 10);

        the_end! (receiver);
    }



    #[test]
    fn example_06_16 () {
        let src =
r#"%TAG !yaml! tag:yaml.org,2002:
---
!yaml!str "foo""#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), data, dir, tag, r"!yaml!", r"tag:yaml.org,2002:");
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""foo""#, !=r"!yaml!str");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_17 () {
        let src =
r#"%TAG ! !foo
%TAG ! !foo
bar"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), data, dir, tag, r"!", r"!foo");
        expect! (receiver, (0, 0, 2), data, dir, tag, r"!", r"!foo");

        expect! (receiver, (0, 0, 3), doc, start);
        expect! (receiver, (0, 0, 4), data, node, scalar, r"bar");
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r#""bar""#, !=r"!foo");
        expect! (receiver, (0, 0, 3), doc, end);

        expect! (receiver, (0, 0, 1), data, dir, tag, r"!", r"tag:example.com,2000:app/");
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""bar""#, !=r"!foo");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""bar""#, !=r"!<!foo>");
        expect! (receiver, (0, 0, 4), doc, end);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), data, node, scalar, r#""bar""#, !=r"!<tag:example.com,2000:app/foo>");
        expect! (receiver, (0, 0, 3), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_19 () {
        let src =
r#"%TAG !! tag:example.com,2000:app/
---
!!int 1 - 3 # Interval, not integer"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), data, dir, tag, r"!!", r"tag:example.com,2000:app/");

        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r"1 - 3", !=r"!!int");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_19_canonical () {
        let src =
r#"%YAML 1.2
---
!<tag:example.com,2000:app/int> "1 - 3"
"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""1 - 3""#, !=r"!<tag:example.com,2000:app/int>");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_20 () {
        let src =
r#"%TAG !e! tag:example.com,2000:app/
---
!e!foo "bar""#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), data, dir, tag, r"!e!", r"tag:example.com,2000:app/");

        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""bar""#, !=r"!e!foo");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), data, dir, tag, r"!m!", r"!my-");
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r"fluorescent", !=r"!m!light");
        expect! (receiver, (0, 0, 4), doc, end);

        expect! (receiver, (0, 0, 1), data, dir, tag, r"!m!", r"!my-");
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r"green", !=r"!m!light");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""fluorescent""#, !=r"!<!my-light>");
        expect! (receiver, (0, 0, 4), doc, end);

        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""green""#, !=r"!<!my-light>");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_22 () {
        let src =
r#"%TAG !e! tag:example.com,2000:app/
---
- !e!foo "bar""#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), data, dir, tag, r"!e!", r"tag:example.com,2000:app/");

        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), node, sequence);
        expect! (receiver, (1, 3, 4), data, node, scalar, r#""bar""#, !=r"!e!foo");
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_23 () {
        let src =
r#"!!str &a1 "foo":
  !!str bar
&a2 baz : *a1"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r#""foo""#, !=r"!!str", &=r"a1"); 
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"bar", !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"baz", &=r"a2");
        expect! (receiver, (1, 3, 6), data, alias, r"a1");
        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""foo""#, !=r"!!str", &=r"B1");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""bar""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""baz""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, alias, r"B1");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_24 () {
        let src =
r#"!<tag:yaml.org,2002:str> foo :
  !<!bar> baz"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"foo", !=r"!<tag:yaml.org,2002:str>");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"baz", !=r"!<!bar>");
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""foo""#, !=r"!<tag:yaml.org,2002:str>");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""baz""#, !=r"!<!bar>");
        expect! (receiver, (0, 0, 3), data, node, mapping, !="!!map");

        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_25 () {
        let src =
r#"- !<!> foo
- !<$:?> bar"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"foo", !=r"!<!>");
        expect! (receiver, (1, 2, 4), data, node, scalar, r"bar", !=r"!<$:?>");
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_26 () {
        let src =
r#"%TAG !e! tag:example.com,2000:app/
---
- !local foo
- !!str bar
- !e!tag%21 baz"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), data, dir, tag, r"!e!", r"tag:example.com,2000:app/");
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (0, 0, 3), node, sequence);

        expect! (receiver, (1, 3, 4), data, node, scalar, r"foo", !=r"!local");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"bar", !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r"baz", !=r"!e!tag%21");

        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""foo""#, !=r"!<!local>");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""bar""#, !=r"!<tag:yaml.org,2002:str>");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""baz""#, !=r"!<tag:example.com,2000:app/tag!>");

        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");

        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_27 () {
        let src =
r#"%TAG !e! tag:example,2000:app/
---
- !e! foo
- !h!bar baz"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), data, dir, tag, r"!e!", r"tag:example,2000:app/");
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (0, 0, 3), node, sequence);

        expect! (receiver, (1, 3, 4), data, node, scalar, r"foo", !=r"!e!");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"baz", !=r"!h!bar");

        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_28 () {
        let src =
r#"# Assuming conventional resolution:
- "12"
- 12
- ! 12"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r#""12""#);
        expect! (receiver, (1, 2, 4), data, node, scalar, r"12");
        expect! (receiver, (1, 2, 5), data, node, scalar, r"12", !=r"!");
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""12""#, !=r"!<tag:yaml.org,2002:str>");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""12""#, !=r"!<tag:yaml.org,2002:int>");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""12""#, !=r"!<tag:yaml.org,2002:str>");

        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");

        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_29 () {
        let src =
r#"First occurrence: &anchor Value
Second occurrence: *anchor"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"First occurrence");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"Value", &=r"anchor");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"Second occurrence");
        expect! (receiver, (1, 3, 6), data, alias, r"anchor");
        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""First occurrence""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""Value""#, !=r"!!str", &=r"A");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""Second occurrence""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, alias, r"A");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_01 () {
        let src =
r#"First occurrence: &anchor Foo
Second occurrence: *anchor
Override anchor: &anchor Bar
Reuse anchor: *anchor"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"First occurrence");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"Foo", &=r"anchor");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"Second occurrence");
        expect! (receiver, (1, 3, 6), data, alias, r"anchor");
        expect! (receiver, (1, 3, 7), data, node, scalar, r"Override anchor");
        expect! (receiver, (1, 3, 8), data, node, scalar, r"Bar", &=r"anchor");
        expect! (receiver, (1, 3, 9), data, node, scalar, r"Reuse anchor");
        expect! (receiver, (1, 3, 10), data, alias, r"anchor");
        expect! (receiver, (0, 0, 11), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""First occurrence""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""Foo""#, !=r"!!str", &=r"A");

        expect! (receiver, (1, 3, 6), data, node, scalar, r#""Override anchor""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""Bar""#, !=r"!!str", &=r"B");

        expect! (receiver, (1, 3, 8), data, node, scalar, r#""Second occurrence""#, !=r"!!str");
        expect! (receiver, (1, 3, 9), data, alias, r"A");

        expect! (receiver, (1, 3, 10), data, node, scalar, r#""Reuse anchor""#, !=r"!!str");
        expect! (receiver, (1, 3, 11), data, alias, r"B");

        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 12), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_02 () {
        let src =
r#"{
  foo : !!str,
  !!str : bar,
}"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"foo");
        expect! (receiver, (1, 2, 4), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 5), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 6), data, node, scalar, r"bar");
        expect! (receiver, (0, 0, 2), data, node, mapping);
        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_02_extra_01 () {
        let src =
r#"{
  foo  : !!str,
  !!str: bar,
}"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"foo");
        expect! (receiver, (1, 2, 4), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 5), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 6), data, node, scalar, r"bar");
        expect! (receiver, (0, 0, 2), data, node, mapping);
        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""foo""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""bar""#, !=r"!!str");

        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_03 () {
        let src =
r#"{
  ? foo :,
  : bar,
}"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"foo");
        expect! (receiver, (1, 2, 4), node, null);
        expect! (receiver, (1, 2, 5), node, null);
        expect! (receiver, (1, 2, 6), data, node, scalar, r"bar");
        expect! (receiver, (0, 0, 2), data, node, mapping);
        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""foo""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""""#, !=r"!!null");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""""#, !=r"!!null");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""bar""#, !=r"!!str");

        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_04 () {
        let src =
r#""implicit block key" : [
  "implicit flow key" : value,
 ]"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r#""implicit block key""#);
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (2, 4, 5), data, node, scalar, r#""implicit flow key""#);
        expect! (receiver, (2, 4, 6), data, block, map, (2, 4, 5));
        expect! (receiver, (3, 6, 7), data, node, scalar, r"value");
        expect! (receiver, (1, 3, 4), node, sequence);
        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_04_extra_01 () {
        let src =
r#""implicit block key" : [
  "implicit flow key" : value,
  "implicit flow key2" : value2
 ]"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r#""implicit block key""#);
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (2, 4, 5), data, node, scalar, r#""implicit flow key""#);
        expect! (receiver, (2, 4, 6), data, block, map, (2, 4, 5));
        expect! (receiver, (3, 6, 7), data, node, scalar, r"value");
        expect! (receiver, (2, 4, 8), data, node, scalar, r#""implicit flow key2""#);
        expect! (receiver, (2, 4, 9), data, block, map, (2, 4, 8));
        expect! (receiver, (3, 9, 10), data, node, scalar, r"value2");
        expect! (receiver, (1, 3, 4), node, sequence);
        expect! (receiver, (0, 0, 11), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""implicit block key""#, !=r"!!str");
        expect! (receiver, (3, 6, 7), data, node, scalar, r#""implicit flow key""#, !=r"!!str");
        expect! (receiver, (3, 6, 8), data, node, scalar, r#""value""#, !=r"!!str");
        expect! (receiver, (2, 5, 6), data, node, mapping, !=r"!!map");
        expect! (receiver, (1, 3, 5), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 9), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_05 () {
        let src =
r#""folded 
to a space,	
 
to a line feed, or 	\
 \ 	non-content""#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, "\"folded \nto a space,	\n \nto a line feed, or 	\\\n \\ 	non-content\"");
        expect! (receiver, (0, 0, 3), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_05_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "folded to a space,\n\
      to a line feed, \
      or \t \tnon-content""#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, "\"folded to a space,\\n\\\n      to a line feed, \\\n      or \\t \\tnon-content\"", !=r"!!str");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_06 () {
        let src =
r#"" 1st non-empty

 2nd non-empty 
	3rd non-empty ""#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, "\" 1st non-empty\n\n 2nd non-empty \n	3rd non-empty \"");
        expect! (receiver, (0, 0, 3), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_06_canonical () {
        let src =
r#"%YAML 1.2
---
!!str " 1st non-empty\n\
      2nd non-empty \
      3rd non-empty ""#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, "\" 1st non-empty\\n\\\n      2nd non-empty \\\n      3rd non-empty \"", !=r"!!str");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_07 () {
        let src =
r#"'here''s to "quotes"'"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r#"'here''s to "quotes"'"#);
        expect! (receiver, (0, 0, 3), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_07_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "here's to \"quotes\"""#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, "\"here's to \\\"quotes\\\"\"", !=r"!!str");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_08 () {
        let src =
r#"'implicit block key' : [
  'implicit flow key' : value,
 ]"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r#"'implicit block key'"#);
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (2, 4, 5), data, node, scalar, r#"'implicit flow key'"#);
        expect! (receiver, (2, 4, 6), data, block, map, (2, 4, 5));
        expect! (receiver, (3, 6, 7), data, node, scalar, r"value");
        expect! (receiver, (1, 3, 4), node, sequence);
        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_09 () {
        let src =
r#"' 1st non-empty

 2nd non-empty 
	3rd non-empty '"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, "' 1st non-empty\n\n 2nd non-empty \n	3rd non-empty '");
        expect! (receiver, (0, 0, 3), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), node, block, open);
        expect! (receiver, (2, 3, 4), data, literal, r":");
        expect! (receiver, (2, 3, 5), data, literal, r":");
        expect! (receiver, (2, 3, 6), data, literal, r"vector");
        expect! (receiver, (1, 2, 3), node, block, close);

        expect! (receiver, (1, 2, 7), data, node, scalar, r#"": - ()""#);
        expect! (receiver, (1, 2, 8), node, block, open);
        expect! (receiver, (2, 8, 9), data, literal, r"Up");
        expect! (receiver, (2, 8, 10), data, literal, r",");
        expect! (receiver, (2, 8, 11), data, literal, r" ");
        expect! (receiver, (2, 8, 12), data, literal, r"up");
        expect! (receiver, (2, 8, 13), data, literal, r",");
        expect! (receiver, (2, 8, 14), data, literal, r" ");
        expect! (receiver, (2, 8, 15), data, literal, r"and away!");
        expect! (receiver, (1, 2, 8), node, block, close);

        expect! (receiver, (1, 2, 16), data, node, scalar, r"-123");
        expect! (receiver, (1, 2, 17), node, block, open);
        expect! (receiver, (2, 17, 18), data, literal, r"http");
        expect! (receiver, (2, 17, 19), data, literal, r":");
        expect! (receiver, (2, 17, 20), data, literal, r"//example.com/foo");
        expect! (receiver, (2, 17, 21), data, literal, "#");
        expect! (receiver, (2, 17, 22), data, literal, "bar");
        expect! (receiver, (1, 2, 17), node, block, close);

        expect! (receiver, (2, 23, 24), node, block, open);
        expect! (receiver, (3, 24, 25), data, literal, r":");
        expect! (receiver, (3, 24, 26), data, literal, r":");
        expect! (receiver, (3, 24, 27), data, literal, r"vector");
        expect! (receiver, (2, 23, 24), node, block, close);

        expect! (receiver, (2, 23, 28), data, node, scalar, r#"": - ()""#);
        expect! (receiver, (2, 23, 29), data, node, scalar, r#""Up, up and away!""#);
        expect! (receiver, (2, 23, 30), data, node, scalar, r"-123");
        expect! (receiver, (2, 23, 31), node, block, open);
        expect! (receiver, (3, 31, 32), data, literal, r"http");
        expect! (receiver, (3, 31, 33), data, literal, r":");
        expect! (receiver, (3, 31, 34), data, literal, r"//example.com/foo");
        expect! (receiver, (3, 31, 35), data, literal, r"#");
        expect! (receiver, (3, 31, 36), data, literal, r"bar");
        expect! (receiver, (2, 23, 31), node, block, close);
        expect! (receiver, (1, 2, 23), node, sequence);
        expect! (receiver, (0, 0, 37), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""::vector""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#"": - ()""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""Up, up, and away!""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""-123""#, !=r"!!int");
        expect! (receiver, (1, 3, 8), data, node, scalar, r#""http://example.com/foo#bar""#, !=r"!!str");
        expect! (receiver, (2, 9, 10), data, node, scalar, r#""::vector""#, !=r"!!str");
        expect! (receiver, (2, 9, 11), data, node, scalar, r#"": - ()""#, !=r"!!str");
        expect! (receiver, (2, 9, 12), data, node, scalar, r#""Up, up, and away!""#, !=r"!!str");
        expect! (receiver, (2, 9, 13), data, node, scalar, r#""-123""#, !=r"!!int");
        expect! (receiver, (2, 9, 14), data, node, scalar, r#""http://example.com/foo#bar""#, !=r"!!str");
        expect! (receiver, (1, 3, 9), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");

        expect! (receiver, (0, 0, 15), doc, end);
        the_end!(receiver);
    }



    #[test]
    fn example_07_11 () {
        let src =
r#"implicit block key : [
  implicit flow key : value,
 ]"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"implicit block key");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (2, 4, 5), data, node, scalar, r"implicit flow key");
        expect! (receiver, (2, 4, 6), data, block, map, (2, 4, 5));
        expect! (receiver, (3, 6, 7), data, node, scalar, r"value");
        expect! (receiver, (1, 3, 4), node, sequence);
        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""implicit block key""#, !=r"!!str");
        expect! (receiver, (3, 6, 7), data, node, scalar, r#""implicit flow key""#, !=r"!!str");
        expect! (receiver, (3, 6, 8), data, node, scalar, r#""value""#, !=r"!!str");
        expect! (receiver, (2, 5, 6), data, node, mapping, !=r"!!map");
        expect! (receiver, (1, 3, 5), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 9), doc, end);
        the_end! (receiver);
    }



    #[test]
    fn example_07_12 () {
        let src =
r#"1st non-empty

 2nd non-empty 
	3rd non-empty
"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (1, 2, 3), data, literal, r"1st non-empty");
        expect! (receiver, (1, 2, 4), data, literal, "\n");
        expect! (receiver, (1, 2, 5), data, literal, r"2nd non-empty");
        expect! (receiver, (1, 2, 6), data, literal, r" ");
        expect! (receiver, (1, 2, 7), data, literal, r"3rd non-empty");
        expect! (receiver, (0, 0, 2), node, block, close);
        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (0, 0, 3), data, node, scalar, "\"1st non-empty\\n\\\n      2nd non-empty \\\n      3rd non-empty\"", !=r"!!str");

        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_13 () {
        let src =
r#"- [ one, two, ]
- [three ,four]"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar, r"one");
        expect! (receiver, (2, 3, 5), data, node, scalar, r"two");
        expect! (receiver, (1, 2, 3), node, sequence);

        expect! (receiver, (2, 6, 7), data, node, scalar, r"three");
        expect! (receiver, (2, 6, 8), data, node, scalar, r"four");
        expect! (receiver, (1, 2, 6), node, sequence);

        expect! (receiver, (0, 0, 9), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (2, 4, 5), data, node, scalar, r#""one""#, !=r"!!str");
        expect! (receiver, (2, 4, 6), data, node, scalar, r#""two""#, !=r"!!str");
        expect! (receiver, (1, 3, 4), data, node, sequence, !=r"!!seq");

        expect! (receiver, (2, 7, 8), data, node, scalar, r#""three""#, !=r"!!str");
        expect! (receiver, (2, 7, 9), data, node, scalar, r#""four""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, sequence, !=r"!!seq");

        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, "\"double\n quoted\"");
        expect! (receiver, (1, 2, 4), data, node, scalar, "'single\n           quoted'");
        expect! (receiver, (1, 2, 5), node, block, open);
        expect! (receiver, (2, 5, 6), data, literal, r"plain");
        expect! (receiver, (2, 5, 7), data, literal, r" ");
        expect! (receiver, (2, 5, 8), data, literal, r"text");
        expect! (receiver, (1, 2, 5), node, block, close);
        expect! (receiver, (2, 9, 10), data, node, scalar, r"nested");
        expect! (receiver, (1, 2, 9), node, sequence);
        expect! (receiver, (1, 2, 11), data, node, scalar, r"single");
        expect! (receiver, (1, 2, 12), data, block, map, (1, 2, 11));
        expect! (receiver, (2, 12, 13), data, node, scalar, r"pair");
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 14), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""double quoted""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""single quoted""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""plain text""#, !=r"!!str");

        expect! (receiver, (2, 7, 8), data, node, scalar, r#""nested""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, sequence, !=r"!!seq");

        expect! (receiver, (2, 9, 10), data, node, scalar, r#""single""#, !=r"!!str");
        expect! (receiver, (2, 9, 11), data, node, scalar, r#""pair""#, !=r"!!str");
        expect! (receiver, (1, 3, 9), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 12), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_15 () {
        let src =
r#"- { one : two , three: four , }
- {five: six,seven : eight}"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 2), node, sequence);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar, r"one");
        expect! (receiver, (2, 3, 5), data, node, scalar, r"two");
        expect! (receiver, (2, 3, 6), data, node, scalar, r"three");
        expect! (receiver, (2, 3, 7), data, node, scalar, r"four");
        expect! (receiver, (1, 2, 3), data, node, mapping);

        expect! (receiver, (2, 8, 9), data, node, scalar, r"five");
        expect! (receiver, (2, 8, 10), data, node, scalar, r"six");
        expect! (receiver, (2, 8, 11), data, node, scalar, r"seven");
        expect! (receiver, (2, 8, 12), data, node, scalar, r"eight");
        expect! (receiver, (1, 2, 8), data, node, mapping);

        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (2, 4, 5), data, node, scalar, r#""one""#, !=r"!!str");
        expect! (receiver, (2, 4, 6), data, node, scalar, r#""two""#, !=r"!!str");
        expect! (receiver, (2, 4, 7), data, node, scalar, r#""three""#, !=r"!!str");
        expect! (receiver, (2, 4, 8), data, node, scalar, r#""four""#, !=r"!!str");
        expect! (receiver, (1, 3, 4), data, node, mapping, !=r"!!map");

        expect! (receiver, (2, 9, 10), data, node, scalar, r#""five""#, !=r"!!str");
        expect! (receiver, (2, 9, 11), data, node, scalar, r#""six""#, !=r"!!str");
        expect! (receiver, (2, 9, 12), data, node, scalar, r#""seven""#, !=r"!!str");
        expect! (receiver, (2, 9, 13), data, node, scalar, r#""eight""#, !=r"!!str");
        expect! (receiver, (1, 3, 9), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 14), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"explicit");
        expect! (receiver, (1, 2, 4), data, node, scalar, r"entry");
        expect! (receiver, (1, 2, 5), data, node, scalar, r"implicit");
        expect! (receiver, (1, 2, 6), data, node, scalar, r"entry");
        expect! (receiver, (1, 2, 7), node, null);
        expect! (receiver, (1, 2, 8), node, null);
        expect! (receiver, (0, 0, 2), data, node, mapping);

        expect! (receiver, (0, 0, 9), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""explicit""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""entry""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""implicit""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""entry""#, !=r"!!str");
        expect! (receiver, (1, 3, 8), data, node, scalar, r#""""#, !=r"!!null");
        expect! (receiver, (1, 3, 9), data, node, scalar, r#""""#, !=r"!!null");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"unquoted");
        expect! (receiver, (1, 2, 4), data, node, scalar, r#""separate""#);
        expect! (receiver, (1, 2, 5), node, block, open);
        expect! (receiver, (2, 5, 6), data, literal, r"http");
        expect! (receiver, (2, 5, 7), data, literal, r":");
        expect! (receiver, (2, 5, 8), data, literal, r"//foo.com");
        expect! (receiver, (1, 2, 5), node, block, close);
        expect! (receiver, (1, 2, 9), node, null);
        expect! (receiver, (1, 2, 10), data, node, scalar, r"omitted value");
        expect! (receiver, (1, 2, 11), node, null);
        expect! (receiver, (1, 2, 12), node, null);
        expect! (receiver, (1, 2, 13), data, node, scalar, r"omitted key");

        expect! (receiver, (0, 0, 2), data, node, mapping);
        expect! (receiver, (0, 0, 14), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""unquoted""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""separate""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""http://foo.com""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""""#, !=r"!!null");
        expect! (receiver, (1, 3, 8), data, node, scalar, r#""omitted value""#, !=r"!!str");
        expect! (receiver, (1, 3, 9), data, node, scalar, r#""""#, !=r"!!null");
        expect! (receiver, (1, 3, 10), data, node, scalar, r#""""#, !=r"!!null");
        expect! (receiver, (1, 3, 11), data, node, scalar, r#""omitted key""#, !=r"!!str");

        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 12), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_18 () {
        let src =
r#"{
"adjacent":value,
"readable": value,
"empty":
}"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r#""adjacent""#);
        expect! (receiver, (1, 2, 4), data, node, scalar, r"value");
        expect! (receiver, (1, 2, 5), data, node, scalar, r#""readable""#);
        expect! (receiver, (1, 2, 6), data, node, scalar, r"value");
        expect! (receiver, (1, 2, 7), data, node, scalar, r#""empty""#);
        expect! (receiver, (1, 2, 8), node, null);

        expect! (receiver, (0, 0, 2), data, node, mapping);
        expect! (receiver, (0, 0, 9), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""adjacent""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""value""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""readable""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""value""#, !=r"!!str");
        expect! (receiver, (1, 3, 8), data, node, scalar, r#""empty""#, !=r"!!str");
        expect! (receiver, (1, 3, 9), data, node, scalar, r#""""#, !=r"!!null");

        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_19 () {
        let src =
r#"[
foo: bar
]"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"foo");
        expect! (receiver, (1, 2, 4), data, block, map, (1, 2, 3));
        expect! (receiver, (2, 4, 5), data, node, scalar, r"bar");
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_19_canonical () {
        let src =
r#"%YAML 1.2
---
!!seq [
  !!map { ? !!str "foo" : !!str "bar" }
]"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (2, 4, 5), data, node, scalar, r#""foo""#, !=r"!!str");
        expect! (receiver, (2, 4, 6), data, node, scalar, r#""bar""#, !=r"!!str");

        expect! (receiver, (1, 3, 4), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_20 () {
        let src =
r#"[
? foo
 bar : baz
]
"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), node, block, open);
        expect! (receiver, (3, 4, 5), data, literal, r"foo");
        expect! (receiver, (3, 4, 6), data, literal, r" ");
        expect! (receiver, (3, 4, 7), data, literal, r"bar");
        expect! (receiver, (2, 3, 4), node, block, close);
        expect! (receiver, (2, 3, 8), data, node, scalar, r"baz");
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 9), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (2, 4, 5), data, node, scalar, r#""foo bar""#, !=r"!!str");
        expect! (receiver, (2, 4, 6), data, node, scalar, r#""baz""#, !=r"!!str");
        expect! (receiver, (1, 3, 4), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_21 () {
        let src =
r#"- [ YAML : separate ]
- [ : empty key entry ]
- [ {JSON: like}:adjacent ]"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar, r"YAML");
        expect! (receiver, (2, 3, 5), data, block, map, (2, 3, 4));
        expect! (receiver, (3, 5, 6), data, node, scalar, r"separate");
        expect! (receiver, (1, 2, 3), node, sequence);

        expect! (receiver, (2, 7, 8), data, node, mapping);
        expect! (receiver, (3, 8, 9), node, null);
        expect! (receiver, (3, 8, 10), data, node, scalar, r"empty key entry");
        expect! (receiver, (1, 2, 7), node, sequence);

        expect! (receiver, (3, 12, 13), data, node, scalar, r"JSON");
        expect! (receiver, (3, 12, 14), data, node, scalar, r"like");
        expect! (receiver, (2, 11, 12), data, node, mapping);
        expect! (receiver, (2, 11, 15), data, block, map, (2, 11, 12));
        expect! (receiver, (3, 15, 16), data, node, scalar, r"adjacent");
        expect! (receiver, (1, 2, 11), node, sequence);
        expect! (receiver, (0, 0, 17), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (3, 5, 6), data, node, scalar, r#""YAML""#, !=r"!!str");
        expect! (receiver, (3, 5, 7), data, node, scalar, r#""separate""#, !=r"!!str");
        expect! (receiver, (2, 4, 5), data, node, mapping, !=r"!!map");
        expect! (receiver, (1, 3, 4), data, node, sequence, !=r"!!seq");

        expect! (receiver, (3, 9, 10), data, node, scalar, r#""""#, !=r"!!null");
        expect! (receiver, (3, 9, 11), data, node, scalar, r#""empty key entry""#, !=r"!!str");
        expect! (receiver, (2, 8, 9), data, node, mapping, !=r"!!map");
        expect! (receiver, (1, 3, 8), data, node, sequence, !=r"!!seq");

        expect! (receiver, (4, 14, 15), data, node, scalar, r#""JSON""#, !=r"!!str");
        expect! (receiver, (4, 14, 16), data, node, scalar, r#""like""#, !=r"!!str");
        expect! (receiver, (3, 13, 14), data, node, mapping, !=r"!!map");
        expect! (receiver, (3, 13, 17), data, node, scalar, r#""adjacent""#);
        expect! (receiver, (2, 12, 13), data, node, mapping, !=r"!!map");
        expect! (receiver, (1, 3, 12), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");

        expect! (receiver, (0, 0, 18), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_22 () {
        let src =
r#"[ foo
 bar: invalid,
 "foo...>1K characters...bar": invalid ]"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), node, block, open);
        expect! (receiver, (2, 3, 4), data, literal, r"foo");
        expect! (receiver, (2, 3, 5), data, literal, r" ");
        expect! (receiver, (2, 3, 6), data, literal, r"bar");
        expect! (receiver, (1, 2, 3), node, block, close);
        expect! (receiver, (1, 2, 7), data, block, map, (1, 2, 3));
        expect! (receiver, (2, 7, 8), data, node, scalar, r"invalid");
        expect! (receiver, (1, 2, 9), data, node, scalar, r#""foo...>1K characters...bar""#);
        expect! (receiver, (1, 2, 10), data, block, map, (1, 2, 9));
        expect! (receiver, (2, 10, 11), data, node, scalar, r"invalid");
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 12), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_22_extra () {
        let src =
r#"[ foo
 bar: invalid,
 "foo aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa bar": invalid ]"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), node, block, open);
        expect! (receiver, (2, 3, 4), data, literal, r"foo");
        expect! (receiver, (2, 3, 5), data, literal, r" ");
        expect! (receiver, (2, 3, 6), data, literal, r"bar");
        expect! (receiver, (1, 2, 3), node, block, close);
        expect! (receiver, (1, 2, 7), data, block, map, (1, 2, 3));
        expect! (receiver, (2, 7, 8), data, node, scalar, r"invalid");
        expect! (receiver, (1, 2, 9), data, node, scalar, r#""foo aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa bar""#);
        expect! (receiver, (1, 2, 10), data, block, map, (1, 2, 9));
        expect! (receiver, (2, 10, 11), data, node, scalar, r"invalid");
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 12), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_23 () {
        let src =
r#"- [ a, b ]
- { a: b }
- "a"
- 'b'
- c"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar, r"a");
        expect! (receiver, (2, 3, 5), data, node, scalar, r"b");
        expect! (receiver, (1, 2, 3), node, sequence);

        expect! (receiver, (2, 6, 7), data, node, scalar, r"a");
        expect! (receiver, (2, 6, 8), data, node, scalar, r"b");
        expect! (receiver, (1, 2, 6), data, node, mapping);

        expect! (receiver, (1, 2, 9), data, node, scalar, r#""a""#);
        expect! (receiver, (1, 2, 10), data, node, scalar, r"'b'");
        expect! (receiver, (1, 2, 11), data, node, scalar, r"c");

        expect! (receiver, (0, 0, 12), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (2, 4, 5), data, node, scalar, r#""a""#, !=r"!!str");
        expect! (receiver, (2, 4, 6), data, node, scalar, r#""b""#, !=r"!!str");
        expect! (receiver, (1, 3, 4), data, node, sequence, !=r"!!seq");

        expect! (receiver, (2, 7, 8), data, node, scalar, r#""a""#, !=r"!!str");
        expect! (receiver, (2, 7, 9), data, node, scalar, r#""b""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, mapping, !=r"!!map");

        expect! (receiver, (1, 3, 10), data, node, scalar, r#""a""#, !=r"!!str");
        expect! (receiver, (1, 3, 11), data, node, scalar, r#""b""#, !=r"!!str");
        expect! (receiver, (1, 3, 12), data, node, scalar, r#""c""#, !=r"!!str");

        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_07_24 () {
        let src =
r#"- !!str "a"
- 'b'
- &anchor "c"
- *anchor
- !!str"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r#""a""#, !=r"!!str");
        expect! (receiver, (1, 2, 4), data, node, scalar, r"'b'");
        expect! (receiver, (1, 2, 5), data, node, scalar, r#""c""#, &=r"anchor");
        expect! (receiver, (1, 2, 6), data, alias, r"anchor");
        expect! (receiver, (1, 2, 7), data, node, scalar !=r"!!str");
        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""a""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""b""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""c""#, !=r"!!str", &=r"A");
        expect! (receiver, (1, 3, 7), data, alias, r"A");
        expect! (receiver, (1, 3, 8), data, node, scalar, r#""""#, !=r"!!str");

        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 9), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);

        expect! (receiver, (1, 2, 3), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, literal, "literal");
        expect! (receiver, (2, 3, 5), data, literal, "\n");
        expect! (receiver, (1, 2, 3), node, block, close);

        expect! (receiver, (1, 2, 6), node, block, open);
        expect! (receiver, (2, 6, 7), data, literal, " folded");
        expect! (receiver, (2, 6, 8), data, literal, "\n");
        expect! (receiver, (1, 2, 6), node, block, close);

        expect! (receiver, (1, 2, 9), node, block, open);
        expect! (receiver, (2, 9, 10), data, literal, "keep");
        expect! (receiver, (2, 9, 11), data, literal, "\n");
        expect! (receiver, (2, 9, 12), data, literal, "\n");
        expect! (receiver, (1, 2, 9), node, block, close);

        expect! (receiver, (1, 2, 13), node, block, open);
        expect! (receiver, (2, 13, 14), data, literal, " strip");
        expect! (receiver, (1, 2, 13), node, block, close);

        expect! (receiver, (0, 0, 15), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""literal\n""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""·folded\n""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""keep\n\n""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""·strip""#, !=r"!!str");

        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);

        expect! (receiver, (1, 2, 3), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, literal, "detected");
        expect! (receiver, (2, 3, 5), data, literal, "\n");
        expect! (receiver, (1, 2, 3), node, block, close);

        expect! (receiver, (1, 2, 6), node, block, open);
        expect! (receiver, (2, 6, 7), data, literal, "\n");
        expect! (receiver, (2, 6, 8), data, literal, "\n");
        expect! (receiver, (2, 6, 9), data, literal, "# detected");
        expect! (receiver, (2, 6, 10), data, literal, "\n");
        expect! (receiver, (1, 2, 6), node, block, close);

        expect! (receiver, (1, 2, 11), node, block, open);
        expect! (receiver, (2, 11, 12), data, literal, " explicit");
        expect! (receiver, (2, 11, 13), data, literal, "\n");
        expect! (receiver, (1, 2, 11), node, block, close);

        expect! (receiver, (1, 2, 14), node, block, open);
        expect! (receiver, (2, 14, 15), data, literal, "\t");
        expect! (receiver, (2, 14, 16), data, literal, "\n");
        expect! (receiver, (2, 14, 17), data, literal, "detected");
        expect! (receiver, (2, 14, 18), data, literal, "\n");
        expect! (receiver, (1, 2, 14), node, block, close);

        expect! (receiver, (0, 0, 19), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_08_02_extra () {
        let src =
r#"- |
 detected
- |
 
  
  # detected
- |1
  explicit
- |
 	
 detected
"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);

        expect! (receiver, (1, 2, 3), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, literal, "detected");
        expect! (receiver, (2, 3, 5), data, literal, "\n");
        expect! (receiver, (1, 2, 3), node, block, close);

        expect! (receiver, (1, 2, 6), node, block, open);
        expect! (receiver, (2, 6, 7), data, literal, "\n");
        expect! (receiver, (2, 6, 8), data, literal, "\n");
        expect! (receiver, (2, 6, 9), data, literal, "# detected");
        expect! (receiver, (2, 6, 10), data, literal, "\n");
        expect! (receiver, (1, 2, 6), node, block, close);

        expect! (receiver, (1, 2, 11), node, block, open);
        expect! (receiver, (2, 11, 12), data, literal, " explicit");
        expect! (receiver, (2, 11, 13), data, literal, "\n");
        expect! (receiver, (1, 2, 11), node, block, close);

        expect! (receiver, (1, 2, 14), node, block, open);
        expect! (receiver, (2, 14, 15), data, literal, "\t");
        expect! (receiver, (2, 14, 16), data, literal, "\n");
        expect! (receiver, (2, 14, 17), data, literal, "detected");
        expect! (receiver, (2, 14, 18), data, literal, "\n");
        expect! (receiver, (1, 2, 14), node, block, close);

        expect! (receiver, (0, 0, 19), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_08_03_01 () {
        let src =
r#"- |
  
 text
"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, literal, "\n");
        expect! (receiver, (2, 3, 5), data, literal, r"text");
        expect! (receiver, (2, 3, 6), data, literal, "\n");
        expect! (receiver, (1, 2, 3), node, block, close);
        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_08_03_02 () {
        let src =
r#"- >
  text
 text
"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, literal, r"text");
        expect! (receiver, (2, 3, 5), data, literal, "\n");
        expect! (receiver, (1, 2, 3), node, block, close);
        expect! (receiver, (0, 0, 6), data, node, scalar, r"text");
        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_08_03_03 () {
        let src =
r#"- |2
 text
"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), node, block, open);
        expect! (receiver, (1, 2, 3), node, block, close);
        expect! (receiver, (0, 0, 4), data, node, scalar, r"text");
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"strip");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, block, open);
        expect! (receiver, (2, 4, 5), data, literal, r"text");
        expect! (receiver, (1, 3, 4), node, block, close);
        expect! (receiver, (1, 3, 6), data, node, scalar, r"clip");
        expect! (receiver, (1, 3, 7), node, block, open);
        expect! (receiver, (2, 7, 8), data, literal, r"text");
        expect! (receiver, (2, 7, 9), data, literal, "\n");
        expect! (receiver, (1, 3, 7), node, block, close);
        expect! (receiver, (1, 3, 10), data, node, scalar, r"keep");
        expect! (receiver, (1, 3, 11), node, block, open);
        expect! (receiver, (2, 11, 12), data, literal, r"text");
        expect! (receiver, (2, 11, 13), data, literal, "\n");
        expect! (receiver, (1, 3, 11), node, block, close);

        expect! (receiver, (0, 0, 14), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""strip""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""text""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""clip""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""text\n""#, !=r"!!str");
        expect! (receiver, (1, 3, 8), data, node, scalar, r#""keep""#, !=r"!!str");
        expect! (receiver, (1, 3, 9), data, node, scalar, r#""text\n""#, !=r"!!str");

        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"strip");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, block, open);
        expect! (receiver, (2, 4, 5), data, literal, r"# text");
        expect! (receiver, (1, 3, 4), node, block, close);
        expect! (receiver, (1, 3, 6), data, node, scalar, r"clip");
        expect! (receiver, (1, 3, 7), node, block, open);
        expect! (receiver, (2, 7, 8), data, literal, r"# text");
        expect! (receiver, (2, 7, 9), data, literal, "\n");
        expect! (receiver, (1, 3, 7), node, block, close);
        expect! (receiver, (1, 3, 10), data, node, scalar, r"keep");
        expect! (receiver, (1, 3, 11), node, block, open);
        expect! (receiver, (2, 11, 12), data, literal, r"# text");
        expect! (receiver, (2, 11, 13), data, literal, "\n");
        expect! (receiver, (2, 11, 14), data, literal, "\n");
        expect! (receiver, (1, 3, 11), node, block, close);

        expect! (receiver, (0, 0, 15), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""strip""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r##""# text""##, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""clip""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r##""# text\n""##, !=r"!!str");
        expect! (receiver, (1, 3, 8), data, node, scalar, r#""keep""#, !=r"!!str");
        expect! (receiver, (1, 3, 9), data, node, scalar, r##""# text\n""##, !=r"!!str");

        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_08_06 () {
        let src =
r#"strip: >-

clip: >

keep: |+

"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"strip");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, block, open);
        expect! (receiver, (1, 3, 4), node, block, close);

        expect! (receiver, (1, 3, 5), data, node, scalar, r"clip");
        expect! (receiver, (1, 3, 6), node, block, open);
        expect! (receiver, (1, 3, 6), node, block, close);

        expect! (receiver, (1, 3, 7), data, node, scalar, r"keep");
        expect! (receiver, (1, 3, 8), node, block, open);
        expect! (receiver, (2, 8, 9), data, literal, "\n");
        expect! (receiver, (1, 3, 8), node, block, close);

        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""strip""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""clip""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""""#, !=r"!!str");
        expect! (receiver, (1, 3, 8), data, node, scalar, r#""keep""#, !=r"!!str");
        expect! (receiver, (1, 3, 9), data, node, scalar, r#""\n""#, !=r"!!str");

        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_08_07 () {
        let src =
r#"|
 literal
 	text

"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, r"literal");
        expect! (receiver, (1, 2, 4), data, literal, "\n");
        expect! (receiver, (1, 2, 5), data, literal, "\ttext");
        expect! (receiver, (1, 2, 6), data, literal, "\n");
        expect! (receiver, (0, 0, 2), node, block, close);

        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_08_07_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "literal\n\ttext\n""#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (0, 0, 3), data, node, scalar, r#""literal\n\ttext\n""#, !=r"!!str");

        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_08_08 () {
        let src =
r#"|
 
  
  literal
   
  
  text

 # Comment"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, "\n");
        expect! (receiver, (1, 2, 4), data, literal, "\n");
        expect! (receiver, (1, 2, 5), data, literal, "literal");
        expect! (receiver, (1, 2, 6), data, literal, "\n");
        expect! (receiver, (1, 2, 7), data, literal, " ");
        expect! (receiver, (1, 2, 8), data, literal, "\n");
        expect! (receiver, (1, 2, 9), data, literal, "\n");
        expect! (receiver, (1, 2, 10), data, literal, "text");
        expect! (receiver, (1, 2, 11), data, literal, "\n");
        expect! (receiver, (0, 0, 2), node, block, close);

        expect! (receiver, (0, 0, 12), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_08_09 () {
        let src =
r#">
 folded
 text

"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, "folded");
        expect! (receiver, (1, 2, 4), data, literal, r" ");
        expect! (receiver, (1, 2, 5), data, literal, "text");
        expect! (receiver, (1, 2, 6), data, literal, "\n");
        expect! (receiver, (0, 0, 2), node, block, close);

        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, "\n");
        expect! (receiver, (1, 2, 4), data, literal, r"folded");
        expect! (receiver, (1, 2, 5), data, literal, r" ");
        expect! (receiver, (1, 2, 6), data, literal, r"line");
        expect! (receiver, (1, 2, 7), data, literal, "\n");
        expect! (receiver, (1, 2, 8), data, literal, r"next");
        expect! (receiver, (1, 2, 9), data, literal, r" ");
        expect! (receiver, (1, 2, 10), data, literal, r"line");
        expect! (receiver, (1, 2, 11), data, literal, "\n");
        expect! (receiver, (1, 2, 12), data, literal, r"  * bullet");
        expect! (receiver, (1, 2, 13), data, literal, "\n");
        expect! (receiver, (1, 2, 14), data, literal, "\n");
        expect! (receiver, (1, 2, 15), data, literal, r"  * list");
        expect! (receiver, (1, 2, 16), data, literal, "\n");
        expect! (receiver, (1, 2, 17), data, literal, r"  * lines");
        expect! (receiver, (1, 2, 18), data, literal, "\n");
        expect! (receiver, (1, 2, 19), data, literal, "\n");
        expect! (receiver, (1, 2, 20), data, literal, r"last");
        expect! (receiver, (1, 2, 21), data, literal, r" ");
        expect! (receiver, (1, 2, 22), data, literal, r"line");
        expect! (receiver, (1, 2, 23), data, literal, "\n");
        expect! (receiver, (0, 0, 2), node, block, close);

        expect! (receiver, (0, 0, 24), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (0, 0, 3), data, node, scalar, "\"\\n\\\n      folded line\\n\\\n      next line\\n\\\n      \\  * bullet\\n\n      \\n\\\n      \\  * list\\n\\\n      \\  * lines\\n\\\n      \\n\\\n      last line\\n\"", !=r"!!str");

        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, "\n");
        expect! (receiver, (1, 2, 4), data, literal, r"folded");
        expect! (receiver, (1, 2, 5), data, literal, r" ");
        expect! (receiver, (1, 2, 6), data, literal, r"line");
        expect! (receiver, (1, 2, 7), data, literal, "\n");
        expect! (receiver, (1, 2, 8), data, literal, r"next");
        expect! (receiver, (1, 2, 9), data, literal, r" ");
        expect! (receiver, (1, 2, 10), data, literal, r"line");
        expect! (receiver, (1, 2, 11), data, literal, "\n");
        expect! (receiver, (1, 2, 12), data, literal, r"  * bullet");
        expect! (receiver, (1, 2, 13), data, literal, "\n");
        expect! (receiver, (1, 2, 14), data, literal, "\n");
        expect! (receiver, (1, 2, 15), data, literal, r"  * list");
        expect! (receiver, (1, 2, 16), data, literal, "\n");
        expect! (receiver, (1, 2, 17), data, literal, r"  * lines");
        expect! (receiver, (1, 2, 18), data, literal, "\n");
        expect! (receiver, (1, 2, 19), data, literal, "\n");
        expect! (receiver, (1, 2, 20), data, literal, r"last");
        expect! (receiver, (1, 2, 21), data, literal, r" ");
        expect! (receiver, (1, 2, 22), data, literal, r"line");
        expect! (receiver, (1, 2, 23), data, literal, "\n");
        expect! (receiver, (0, 0, 2), node, block, close);

        expect! (receiver, (0, 0, 24), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_08_14 () {
        let src =
r#"block sequence:
  - one
  - two : three
"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"block sequence");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, sequence);
        expect! (receiver, (2, 4, 5), data, node, scalar, r"one");
        expect! (receiver, (2, 4, 6), data, node, scalar, r"two");
        expect! (receiver, (2, 4, 7), data, block, map, (2, 4, 6));
        expect! (receiver, (3, 7, 8), data, node, scalar, r"three");

        expect! (receiver, (0, 0, 9), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""block sequence""#, !=r"!!str");
        expect! (receiver, (2, 5, 6), data, node, scalar, r#""one""#, !=r"!!str");
        expect! (receiver, (3, 7, 8), data, node, scalar, r#""two""#, !=r"!!str");
        expect! (receiver, (3, 7, 9), data, node, scalar, r#""three""#, !=r"!!str");
        expect! (receiver, (2, 5, 7), data, node, mapping, !=r"!!map");
        expect! (receiver, (1, 3, 5), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), node, null);
        expect! (receiver, (1, 2, 4), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 4, 5), data, literal, r"block node");
        expect! (receiver, (2, 4, 6), data, literal, "\n");
        expect! (receiver, (1, 2, 4), node, block, close);
        expect! (receiver, (1, 2, 7), node, sequence);
        expect! (receiver, (2, 7, 8), data, node, scalar, r"one");
        expect! (receiver, (2, 7, 9), data, node, scalar, r"two");
        expect! (receiver, (1, 2, 10), data, node, scalar, r"one");
        expect! (receiver, (1, 2, 11), data, block, map, (1, 2, 10));
        expect! (receiver, (2, 11, 12), data, node, scalar, r"two");

        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, "\"\"", !=r"!!null");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""block node\n""#, !=r"!!str");

        expect! (receiver, (2, 6, 7), data, node, scalar, r#""one""#, !=r"!!str");
        expect! (receiver, (2, 6, 8), data, node, scalar, r#""two""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, sequence, !=r"!!seq");

        expect! (receiver, (2, 9, 10), data, node, scalar, r#""one""#, !=r"!!str");
        expect! (receiver, (2, 9, 11), data, node, scalar, r#""two""#, !=r"!!str");
        expect! (receiver, (1, 3, 9), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");

        expect! (receiver, (0, 0, 12), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_08_16 () {
        let src =
r#"block mapping:
 key: value
"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"block mapping");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"key");
        expect! (receiver, (1, 3, 5), data, block, map, (1, 3, 4));
        expect! (receiver, (2, 5, 6), data, node, scalar, r"value");

        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 2), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"explicit key");
        expect! (receiver, (1, 2, 4), node, null);
        expect! (receiver, (1, 2, 5), node, block, open);
        expect! (receiver, (2, 5, 6), data, literal, r"block key");
        expect! (receiver, (2, 5, 7), data, literal, "\n");
        expect! (receiver, (1, 2, 5), node, block, close);
        expect! (receiver, (1, 2, 8), node, sequence);
        expect! (receiver, (2, 8, 9), data, node, scalar, r"one");
        expect! (receiver, (2, 8, 10), data, node, scalar, r"two");

        expect! (receiver, (0, 0, 11), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_08_18 () {
        let src =
r#"plain key: in-line value
: # Both empty
"quoted key":
- entry
"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"plain key");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"in-line value");
        expect! (receiver, (1, 3, 5), node, null);
        expect! (receiver, (1, 3, 6), node, null);
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""quoted key""#);
        expect! (receiver, (1, 3, 8), node, sequence);
        expect! (receiver, (2, 8, 9), data, node, scalar, r"entry");
        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
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

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""plain key""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""in-line value""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""""#, !=r"!!null");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""""#, !=r"!!null");
        expect! (receiver, (1, 3, 8), data, node, scalar, r#""quoted key""#, !=r"!!str");
        expect! (receiver, (2, 9, 10), data, node, scalar, r#""entry""#, !=r"!!str");
        expect! (receiver, (1, 3, 9), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 11), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_08_19 () {
        let src =
r#"- sun: yellow
- ? earth: blue
  : moon: white
"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"sun");
        expect! (receiver, (1, 2, 4), data, block, map, (1, 2, 3));
        expect! (receiver, (2, 4, 5), data, node, scalar, r"yellow");
        expect! (receiver, (1, 2, 6), data, node, mapping);
        expect! (receiver, (2, 6, 7), data, node, scalar, r"earth");
        expect! (receiver, (2, 6, 8), data, block, map, (2, 6, 7));
        expect! (receiver, (3, 8, 9), data, node, scalar, r"blue");
        expect! (receiver, (2, 6, 10), data, node, scalar, r"moon");
        expect! (receiver, (2, 6, 11), data, block, map, (2, 6, 10));
        expect! (receiver, (3, 11, 12), data, node, scalar, r"white");
        expect! (receiver, (0, 0, 13), doc, end);
        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (2, 4, 5), data, node, scalar, r#""sun""#, !=r"!!str");
        expect! (receiver, (2, 4, 6), data, node, scalar, r#""yellow""#, !=r"!!str");
        expect! (receiver, (1, 3, 4), data, node, mapping, !=r"!!map");

        expect! (receiver, (3, 8, 9), data, node, scalar, r#""earth""#, !=r"!!str");
        expect! (receiver, (3, 8, 10), data, node, scalar, r#""blue""#, !=r"!!str");
        expect! (receiver, (2, 7, 8), data, node, mapping, !=r"!!map");

        expect! (receiver, (2, 7, 11), node, null);
        expect! (receiver, (2, 7, 12), node, null);

        expect! (receiver, (3, 13, 14), data, node, scalar, r#""moon""#, !=r"!!str");
        expect! (receiver, (3, 13, 15), data, node, scalar, r#""white""#, !=r"!!str");
        expect! (receiver, (2, 7, 13), data, node, mapping, !=r"!!map");

        expect! (receiver, (1, 3, 7), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");

        expect! (receiver, (0, 0, 16), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r#""flow in block""#);
        expect! (receiver, (1, 2, 4), node, block, open);
        expect! (receiver, (2, 4, 5), data, literal, r"Block scalar");
        expect! (receiver, (2, 4, 6), data, literal, "\n");
        expect! (receiver, (1, 2, 4), node, block, close);
        expect! (receiver, (1, 2, 7), data, node, scalar, r"foo");
        expect! (receiver, (1, 2, 8), data, block, map, (1, 2, 7), !="!!map");
        expect! (receiver, (2, 8, 9), data, node, scalar, r"bar");
        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""flow in block""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""Block scalar\n""#, !=r"!!str");
        expect! (receiver, (2, 6, 7), data, node, scalar, r#""foo""#, !="!!str");
        expect! (receiver, (2, 6, 8), data, node, scalar, r#""bar""#, !="!!str");
        expect! (receiver, (1, 3, 6), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 3), data, node, sequence, !=r"!!seq");

        expect! (receiver, (0, 0, 9), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"literal");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, block, open);
        expect! (receiver, (2, 4, 5), data, literal, r"value");
        expect! (receiver, (2, 4, 6), data, literal, "\n");
        expect! (receiver, (1, 3, 4), node, block, close);
        expect! (receiver, (1, 3, 7), data, node, scalar, r"folded");
        expect! (receiver, (1, 3, 8), node, block, open);
        expect! (receiver, (2, 8, 9), data, literal, r"value");
        expect! (receiver, (1, 3, 8), data, node, block, close, !="!foo");
        expect! (receiver, (0, 0, 10), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""literal""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""value""#, !=r"!!str");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""folded""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""value""#, !=r"!<!foo>");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"sequence");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, sequence, !=r"!!seq");
        expect! (receiver, (2, 4, 5), data, node, scalar, r"entry");
        expect! (receiver, (2, 4, 6), data, node, sequence, !=r"!!seq");
        expect! (receiver, (3, 6, 7), data, node, scalar, r"nested");
        expect! (receiver, (1, 3, 8), data, node, scalar, r"mapping");
        expect! (receiver, (1, 3, 9), data, node, scalar, r"foo");
        expect! (receiver, (1, 3, 10), data, block, map, (1, 3, 9), !=r"!!map");
        expect! (receiver, (2, 10, 11), data, node, scalar, r"bar");
        expect! (receiver, (0, 0, 12), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""sequence""#, !=r"!!str");
        expect! (receiver, (2, 5, 6), data, node, scalar, r#""entry""#, !=r"!!str");
        expect! (receiver, (3, 7, 8), data, node, scalar, r#""nested""#, !=r"!!str");
        expect! (receiver, (2, 5, 7), data, node, sequence, !=r"!!seq");
        expect! (receiver, (1, 3, 5), data, node, sequence, !=r"!!seq");
        expect! (receiver, (1, 3, 9), data, node, scalar, r#""mapping""#, !=r"!!str");
        expect! (receiver, (2, 10, 11), data, node, scalar, r#""foo""#, !=r"!!str");
        expect! (receiver, (2, 10, 12), data, node, scalar, r#""bar""#, !=r"!!str");
        expect! (receiver, (1, 3, 10), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_09_01 () {
        let src =
b"\xEF\xBB\xBF# Comment
# lines
Document";


        let receiver = read_bytes! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"Document");
        expect! (receiver, (0, 0, 3), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_09_01_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "Document""#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""Document""#, !=r"!!str");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_09_02 () {
        let src =
r#"%YAML 1.2
---
Document
... # Suffix"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#"Document"#);
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_09_02_canonical () {
        let src =
r#"%YAML 1.2
---
!!str "Document""#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""Document""#, !=r"!!str");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (1, 2, 3), data, literal, r"Bare");
        expect! (receiver, (1, 2, 4), data, literal, " ");
        expect! (receiver, (1, 2, 5), data, literal, r"document");
        expect! (receiver, (0, 0, 2), node, block, close);
        expect! (receiver, (0, 0, 6), doc, end);
        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), doc, end);
        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (1, 2, 3), data, literal, r"%!PS-Adobe-2.0 # Not the first line");
        expect! (receiver, (0, 0, 2), node, block, close);
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""Bare document""#, !=r"!!str");
        expect! (receiver, (0, 0, 4), doc, end);

        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, "\"%!PS-Adobe-2.0\\n\"", !=r"!!str");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), node, block, open);
        expect! (receiver, (2, 3, 4), data, literal, r"matches");
        expect! (receiver, (2, 3, 5), data, literal, r" ");
        expect! (receiver, (2, 3, 6), data, literal, r"%");
        expect! (receiver, (1, 2, 3), node, block, close);
        expect! (receiver, (1, 2, 7), data, node, scalar, r"20");
        expect! (receiver, (0, 0, 2), data, node, mapping);

        expect! (receiver, (0, 0, 8), doc, end);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (1, 3, 4), data, node, scalar, r#""matches %""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""20""#, !=r"!!int");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 6), doc, end);

        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""""#, !=r"!!null");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), node, block, open);
        expect! (receiver, (1, 3, 4), data, literal, r"%!PS-Adobe-2.0");
        expect! (receiver, (1, 3, 5), data, literal, "\n");
        expect! (receiver, (0, 0, 3), node, block, close);
        expect! (receiver, (0, 0, 6), doc, end);

        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""%!PS-Adobe-2.0\n""#, !=r"!!str");
        expect! (receiver, (0, 0, 4), doc, end);

        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""""#, !=r"!!null");
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"Document");
        expect! (receiver, (0, 0, 3), doc, end);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), doc, end);

        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r"matches %");
        expect! (receiver, (0, 0, 4), data, block, map, (0, 0, 3));
        expect! (receiver, (1, 4, 5), data, node, scalar, r"20");
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""Document""#, !=r"!!str");
        expect! (receiver, (0, 0, 4), doc, end);

        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (0, 0, 3), data, node, scalar, r#""""#, !=r"!!null");
        expect! (receiver, (0, 0, 4), doc, end);

        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);
        expect! (receiver, (1, 3, 4), data, node, scalar, r#""matches %""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""20""#, !=r"!!int");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_10_01 () {
        let src =
r#"Block style: !!map
  Clark : Evans
  Ingy  : döt Net
  Oren  : Ben-Kiki

Flow style: !!map { Clark: Evans, Ingy: döt Net, Oren: Ben-Kiki }"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"Block style");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"Clark");
        expect! (receiver, (1, 3, 5), data, block, map, (1, 3, 4), !=r"!!map");
        expect! (receiver, (2, 5, 6), data, node, scalar, r"Evans");
        expect! (receiver, (2, 5, 7), data, node, scalar, r"Ingy");
        expect! (receiver, (2, 5, 8), data, node, scalar, r"döt Net");
        expect! (receiver, (2, 5, 9), data, node, scalar, r"Oren");
        expect! (receiver, (2, 5, 10), data, node, scalar, r"Ben-Kiki");

        expect! (receiver, (1, 3, 11), data, node, scalar, r"Flow style");
        expect! (receiver, (2, 12, 13), data, node, scalar, r"Clark");
        expect! (receiver, (2, 12, 14), data, node, scalar, r"Evans");
        expect! (receiver, (2, 12, 15), data, node, scalar, r"Ingy");
        expect! (receiver, (2, 12, 16), data, node, scalar, r"döt Net");
        expect! (receiver, (2, 12, 17), data, node, scalar, r"Oren");
        expect! (receiver, (2, 12, 18), data, node, scalar, r"Ben-Kiki");
        expect! (receiver, (1, 3, 12), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 19), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_10_02 () {
        let src =
r#"Block style: !!seq
- Clark Evans
- Ingy döt Net
- Oren Ben-Kiki

Flow style: !!seq [ Clark Evans, Ingy döt Net, Oren Ben-Kiki ]"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"Block style");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, sequence, !=r"!!seq");
        expect! (receiver, (2, 4, 5), data, node, scalar, r"Clark Evans");
        expect! (receiver, (2, 4, 6), data, node, scalar, r"Ingy döt Net");
        expect! (receiver, (2, 4, 7), data, node, scalar, r"Oren Ben-Kiki");
        expect! (receiver, (1, 3, 8), data, node, scalar, r"Flow style");
        expect! (receiver, (2, 9, 10), data, node, scalar, r"Clark Evans");
        expect! (receiver, (2, 9, 11), data, node, scalar, r"Ingy döt Net");
        expect! (receiver, (2, 9, 12), data, node, scalar, r"Oren Ben-Kiki");
        expect! (receiver, (1, 3, 9), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_10_03 () {
        let src =
r#"Block style: !!str |-
  String: just a theory.

Flow style: !!str "String: just a theory.""#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"Block style");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), node, block, open);
        expect! (receiver, (2, 4, 5), data, literal, r"String: just a theory.");
        expect! (receiver, (1, 3, 4), data, node, block, close, !="!!str");

        expect! (receiver, (1, 3, 6), data, node, scalar, r"Flow style");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""String: just a theory.""#, !=r"!!str");

        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_10_04 () {
        let src =
r#"!!null null: value for null key
key with null value: !!null null"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"null", !=r"!!null");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"value for null key");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"key with null value");
        expect! (receiver, (1, 3, 6), data, node, scalar, r"null", !=r"!!null");
        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_10_05 () {
        let src =
r#"YAML is a superset of JSON: !!bool true
Pluto is a planet: !!bool false"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"YAML is a superset of JSON");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"true", !=r"!!bool");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"Pluto is a planet");
        expect! (receiver, (1, 3, 6), data, node, scalar, r"false", !=r"!!bool");
        expect! (receiver, (0, 0, 7), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_10_06 () {
        let src =
r#"negative: !!int -12
zero: !!int 0
positive: !!int 34"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"negative");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"-12", !=r"!!int");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"zero");
        expect! (receiver, (1, 3, 6), data, node, scalar, r"0", !=r"!!int");
        expect! (receiver, (1, 3, 7), data, node, scalar, r"positive");
        expect! (receiver, (1, 3, 8), data, node, scalar, r"34", !=r"!!int");
        expect! (receiver, (0, 0, 9), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_10_07 () {
        let src =
r#"negative: !!float -1
zero: !!float 0
positive: !!float 2.3e4
infinity: !!float .inf
not a number: !!float .nan"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"negative");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"-1", !=r"!!float");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"zero");
        expect! (receiver, (1, 3, 6), data, node, scalar, r"0", !=r"!!float");
        expect! (receiver, (1, 3, 7), data, node, scalar, r"positive");
        expect! (receiver, (1, 3, 8), data, node, scalar, r"2.3e4", !=r"!!float");
        expect! (receiver, (1, 3, 9), data, node, scalar, r"infinity");
        expect! (receiver, (1, 3, 10), data, node, scalar, r".inf", !=r"!!float");
        expect! (receiver, (1, 3, 11), data, node, scalar, r"not a number");
        expect! (receiver, (1, 3, 12), data, node, scalar, r".nan", !=r"!!float");
        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_10_08 () {
        let src =
r#"A null: null
Booleans: [ true, false ]
Integers: [ 0, -0, 3, -19 ]
Floats: [ 0., -0.0, 12e03, -2E+05 ]
Invalid: [ True, Null, 0o7, 0x3A, +12.3 ]"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"A null");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"null");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"Booleans");
        expect! (receiver, (2, 6, 7), data, node, scalar, r"true");
        expect! (receiver, (2, 6, 8), data, node, scalar, r"false");
        expect! (receiver, (1, 3, 6), node, sequence);
        expect! (receiver, (1, 3, 9), data, node, scalar, r"Integers");
        expect! (receiver, (2, 10, 11), data, node, scalar, r"0");
        expect! (receiver, (2, 10, 12), data, node, scalar, r"-0");
        expect! (receiver, (2, 10, 13), data, node, scalar, r"3");
        expect! (receiver, (2, 10, 14), data, node, scalar, r"-19");
        expect! (receiver, (1, 3, 10), node, sequence);
        expect! (receiver, (1, 3, 15), data, node, scalar, r"Floats");
        expect! (receiver, (2, 16, 17), data, node, scalar, r"0.");
        expect! (receiver, (2, 16, 18), data, node, scalar, r"-0.0");
        expect! (receiver, (2, 16, 19), data, node, scalar, r"12e03");
        expect! (receiver, (2, 16, 20), data, node, scalar, r"-2E+05");
        expect! (receiver, (1, 3, 16), node, sequence);
        expect! (receiver, (1, 3, 21), data, node, scalar, r"Invalid");
        expect! (receiver, (2, 22, 23), data, node, scalar, r"True");
        expect! (receiver, (2, 22, 24), data, node, scalar, r"Null");
        expect! (receiver, (2, 22, 25), data, node, scalar, r"0o7");
        expect! (receiver, (2, 22, 26), data, node, scalar, r"0x3A");
        expect! (receiver, (2, 22, 27), data, node, scalar, r"+12.3");
        expect! (receiver, (1, 3, 22), node, sequence);
        expect! (receiver, (0, 0, 28), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""A null""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""null""#, !=r"!!null");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""Booleans""#, !=r"!!str");

        expect! (receiver, (2, 7, 8), data, node, scalar, r#""true""#, !=r"!!bool");
        expect! (receiver, (2, 7, 9), data, node, scalar, r#""false""#, !=r"!!bool");
        expect! (receiver, (1, 3, 7), data, node, sequence, !=r"!!seq");

        expect! (receiver, (1, 3, 10), data, node, scalar, r#""Integers""#, !=r"!!str");
        expect! (receiver, (2, 11, 12), data, node, scalar, r#""0""#, !=r"!!int");
        expect! (receiver, (2, 11, 13), data, node, scalar, r#""-0""#, !=r"!!int");
        expect! (receiver, (2, 11, 14), data, node, scalar, r#""3""#, !=r"!!int");
        expect! (receiver, (2, 11, 15), data, node, scalar, r#""-19""#, !=r"!!int");
        expect! (receiver, (1, 3, 11), data, node, sequence, !=r"!!seq");

        expect! (receiver, (1, 3, 16), data, node, scalar, r#""Floats""#, !=r"!!str");
        expect! (receiver, (2, 17, 18), data, node, scalar, r#""0.""#, !=r"!!float");
        expect! (receiver, (2, 17, 19), data, node, scalar, r#""-0.0""#, !=r"!!float");
        expect! (receiver, (2, 17, 20), data, node, scalar, r#""12e03""#, !=r"!!float");
        expect! (receiver, (2, 17, 21), data, node, scalar, r#""-2E+05""#, !=r"!!float");
        expect! (receiver, (1, 3, 17), data, node, sequence, !=r"!!seq");

        expect! (receiver, (1, 3, 22), data, node, scalar, r#""Invalid""#, !=r"!!str");
        expect! (receiver, (2, 23, 24), data, node, scalar, r"True");
        expect! (receiver, (2, 23, 25), data, node, scalar, r"Null");
        expect! (receiver, (2, 23, 26), data, node, scalar, r"0o7");
        expect! (receiver, (2, 23, 27), data, node, scalar, r"0x3A");
        expect! (receiver, (2, 23, 28), data, node, scalar, r"+12.3");
        expect! (receiver, (1, 3, 23), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 29), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"A null");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"null");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"Also a null");
        expect! (receiver, (1, 3, 6), node, null);
        expect! (receiver, (1, 3, 7), data, node, scalar, r"Not a null");
        expect! (receiver, (1, 3, 8), data, node, scalar, r#""""#);
        expect! (receiver, (1, 3, 9), data, node, scalar, r"Booleans");
        expect! (receiver, (2, 10, 11), data, node, scalar, r"true");
        expect! (receiver, (2, 10, 12), data, node, scalar, r"True");
        expect! (receiver, (2, 10, 13), data, node, scalar, r"false");
        expect! (receiver, (2, 10, 14), data, node, scalar, r"FALSE");
        expect! (receiver, (1, 3, 10), node, sequence);
        expect! (receiver, (1, 3, 15), data, node, scalar, r"Integers");
        expect! (receiver, (2, 16, 17), data, node, scalar, r"0");
        expect! (receiver, (2, 16, 18), data, node, scalar, r"0o7");
        expect! (receiver, (2, 16, 19), data, node, scalar, r"0x3A");
        expect! (receiver, (2, 16, 20), data, node, scalar, r"-19");
        expect! (receiver, (1, 3, 16), node, sequence);
        expect! (receiver, (1, 3, 21), data, node, scalar, r"Floats");
        expect! (receiver, (2, 22, 23), data, node, scalar, r"0.");
        expect! (receiver, (2, 22, 24), data, node, scalar, r"-0.0");
        expect! (receiver, (2, 22, 25), data, node, scalar, r".5");
        expect! (receiver, (2, 22, 26), data, node, scalar, r"+12e03");
        expect! (receiver, (2, 22, 27), data, node, scalar, r"-2E+05");
        expect! (receiver, (1, 3, 22), node, sequence);
        expect! (receiver, (1, 3, 28), data, node, scalar, r"Also floats");
        expect! (receiver, (2, 29, 30), data, node, scalar, r".inf");
        expect! (receiver, (2, 29, 31), data, node, scalar, r"-.Inf");
        expect! (receiver, (2, 29, 32), data, node, scalar, r"+.INF");
        expect! (receiver, (2, 29, 33), data, node, scalar, r".NAN");
        expect! (receiver, (1, 3, 29), node, sequence);
        expect! (receiver, (0, 0, 34), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 1), dir, yaml, (1, 2));
        expect! (receiver, (0, 0, 2), doc, start);

        expect! (receiver, (1, 3, 4), data, node, scalar, r#""A null""#, !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r#""null""#, !=r"!!null");
        expect! (receiver, (1, 3, 6), data, node, scalar, r#""Also a null""#, !=r"!!str");
        expect! (receiver, (1, 3, 7), data, node, scalar, r#""""#, !=r"!!null");
        expect! (receiver, (1, 3, 8), data, node, scalar, r#""Not a null""#, !=r"!!str");
        expect! (receiver, (1, 3, 9), data, node, scalar, r#""""#, !=r"!!str");
        expect! (receiver, (1, 3, 10), data, node, scalar, r#""Booleans""#, !=r"!!str");
        expect! (receiver, (2, 11, 12), data, node, scalar, r#""true""#, !=r"!!bool");
        expect! (receiver, (2, 11, 13), data, node, scalar, r#""True""#, !=r"!!bool");
        expect! (receiver, (2, 11, 14), data, node, scalar, r#""false""#, !=r"!!bool");
        expect! (receiver, (2, 11, 15), data, node, scalar, r#""FALSE""#, !=r"!!bool");
        expect! (receiver, (1, 3, 11), data, node, sequence, !=r"!!seq");
        expect! (receiver, (1, 3, 16), data, node, scalar, r#""Integers""#, !=r"!!str");
        expect! (receiver, (2, 17, 18), data, node, scalar, r#""0""#, !=r"!!int");
        expect! (receiver, (2, 17, 19), data, node, scalar, r#""0o7""#, !=r"!!int");
        expect! (receiver, (2, 17, 20), data, node, scalar, r#""0x3A""#, !=r"!!int");
        expect! (receiver, (2, 17, 21), data, node, scalar, r#""-19""#, !=r"!!int");
        expect! (receiver, (1, 3, 17), data, node, sequence, !=r"!!seq");
        expect! (receiver, (1, 3, 22), data, node, scalar, r#""Floats""#, !=r"!!str");
        expect! (receiver, (2, 23, 24), data, node, scalar, r#""0.""#, !=r"!!float");
        expect! (receiver, (2, 23, 25), data, node, scalar, r#""-0.0""#, !=r"!!float");
        expect! (receiver, (2, 23, 26), data, node, scalar, r#"".5""#, !=r"!!float");
        expect! (receiver, (2, 23, 27), data, node, scalar, r#""+12e03""#, !=r"!!float");
        expect! (receiver, (2, 23, 28), data, node, scalar, r#""-2E+05""#, !=r"!!float");
        expect! (receiver, (1, 3, 23), data, node, sequence, !=r"!!seq");
        expect! (receiver, (1, 3, 29), data, node, scalar, r#""Also floats""#, !=r"!!str");
        expect! (receiver, (2, 30, 31), data, node, scalar, r#"".inf""#, !=r"!!float");
        expect! (receiver, (2, 30, 32), data, node, scalar, r#""-.Inf""#, !=r"!!float");
        expect! (receiver, (2, 30, 33), data, node, scalar, r#""+.INF""#, !=r"!!float");
        expect! (receiver, (2, 30, 34), data, node, scalar, r#"".NAN""#, !=r"!!float");
        expect! (receiver, (1, 3, 30), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 3), data, node, mapping, !=r"!!map");
        expect! (receiver, (0, 0, 35), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_1_01 () {
        let src =
r#"--- &mydict !!mydict
!!value =: val1
!!seq [flow, sequence]: val2
!!map {flow: map}: val3"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"=", !=r"!!value");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2), &=r"mydict", !=r"!!mydict");
        expect! (receiver, (1, 3, 4), data, node, scalar, r"val1");
        expect! (receiver, (2, 5, 6), data, node, scalar, r"flow");
        expect! (receiver, (2, 5, 7), data, node, scalar, r"sequence");
        expect! (receiver, (1, 3, 5), data, node, sequence, !=r"!!seq");
        expect! (receiver, (1, 3, 8), data, node, scalar, r"val2");
        expect! (receiver, (2, 9, 10), data, node, scalar, r"flow");
        expect! (receiver, (2, 9, 11), data, node, scalar, r"map");
        expect! (receiver, (1, 3, 9), data, node, mapping, !=r"!!map");
        expect! (receiver, (1, 3, 12), data, node, scalar, r"val3");
        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_1_02 () {
        let src =
r#"--- &mydict !!mydict
=: val1
[flow, sequence]: val2
{!!str flow: map}: val3"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"=");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2), &=r"mydict", !=r"!!mydict");
        expect! (receiver, (1, 3, 4), data, node, scalar, r"val1");
        expect! (receiver, (2, 5, 6), data, node, scalar, r"flow");
        expect! (receiver, (2, 5, 7), data, node, scalar, r"sequence");
        expect! (receiver, (1, 3, 5), node, sequence);
        expect! (receiver, (1, 3, 8), data, node, scalar, r"val2");
        expect! (receiver, (2, 9, 10), data, node, scalar, r"flow", !=r"!!str");
        expect! (receiver, (2, 9, 11), data, node, scalar, r"map");
        expect! (receiver, (1, 3, 9), data, node, mapping);
        expect! (receiver, (1, 3, 12), data, node, scalar, r"val3");
        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_1_03 () {
        let src =
r#"--- &mydict !!mydict
!!seq [flow, sequence]: val2
!!value =: val1
!!map {!!str flow: map}: val3"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"flow");
        expect! (receiver, (1, 2, 4), data, node, scalar, r"sequence");
        expect! (receiver, (0, 0, 2), data, node, sequence, !=r"!!seq");
        expect! (receiver, (0, 0, 5), data, block, map, (0, 0, 2), &=r"mydict", !=r"!!mydict");
        expect! (receiver, (1, 5, 6), data, node, scalar, r"val2");
        expect! (receiver, (1, 5, 7), data, node, scalar, r"=", !=r"!!value");
        expect! (receiver, (1, 5, 8), data, node, scalar, r"val1");
        expect! (receiver, (2, 9, 10), data, node, scalar, r"flow", !=r"!!str");
        expect! (receiver, (2, 9, 11), data, node, scalar, r"map");
        expect! (receiver, (1, 5, 9), data, node, mapping, !=r"!!map");
        expect! (receiver, (1, 5, 12), data, node, scalar, r"val3");
        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_1_04 () {
        let src =
r#"--- &mydict !!mydict
[flow, sequence]: val2
=: val1
{flow: map}: val3"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"flow");
        expect! (receiver, (1, 2, 4), data, node, scalar, r"sequence");
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 5), data, block, map, (0, 0, 2), &=r"mydict", !=r"!!mydict");
        expect! (receiver, (1, 5, 6), data, node, scalar, r"val2");
        expect! (receiver, (1, 5, 7), data, node, scalar, r"=");
        expect! (receiver, (1, 5, 8), data, node, scalar, r"val1");
        expect! (receiver, (2, 9, 10), data, node, scalar, r"flow");
        expect! (receiver, (2, 9, 11), data, node, scalar, r"map");
        expect! (receiver, (1, 5, 9), data, node, mapping);
        expect! (receiver, (1, 5, 12), data, node, scalar, r"val3");
        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_1_05 () {
        let src =
r#"--- &mydict !!mydict
!!map {!!str flow: map}: val3
!!seq [flow, sequence]: val2
!!value =: val1
"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"flow", !=r"!!str");
        expect! (receiver, (1, 2, 4), data, node, scalar, r"map");
        expect! (receiver, (0, 0, 2), data, node, mapping, !=r"!!map");

        expect! (receiver, (0, 0, 5), data, block, map, (0, 0, 2), &=r"mydict", !=r"!!mydict");
        expect! (receiver, (1, 5, 6), data, node, scalar, r"val3");

        expect! (receiver, (2, 7, 8), data, node, scalar, r"flow");
        expect! (receiver, (2, 7, 9), data, node, scalar, r"sequence");
        expect! (receiver, (1, 5, 7), data, node, sequence, !=r"!!seq");
        expect! (receiver, (1, 5, 10), data, node, scalar, r"val2");

        expect! (receiver, (1, 5, 11), data, node, scalar, r"=", !=r"!!value");
        expect! (receiver, (1, 5, 12), data, node, scalar, r"val1");

        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_1_06 () {
        let src =
r#"--- &mydict !!mydict
{!!str flow: map}: val3
!!seq [flow, sequence]: val2
!!value =: val1
"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"flow", !=r"!!str");
        expect! (receiver, (1, 2, 4), data, node, scalar, r"map");
        expect! (receiver, (0, 0, 2), data, node, mapping);

        expect! (receiver, (0, 0, 5), data, block, map, (0, 0, 2), &=r"mydict", !=r"!!mydict");
        expect! (receiver, (1, 5, 6), data, node, scalar, r"val3");

        expect! (receiver, (2, 7, 8), data, node, scalar, r"flow");
        expect! (receiver, (2, 7, 9), data, node, scalar, r"sequence");
        expect! (receiver, (1, 5, 7), data, node, sequence, !=r"!!seq");
        expect! (receiver, (1, 5, 10), data, node, scalar, r"val2");

        expect! (receiver, (1, 5, 11), data, node, scalar, r"=", !=r"!!value");
        expect! (receiver, (1, 5, 12), data, node, scalar, r"val1");

        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_1_07 () {
        let src =
r#"--- &mydict
!!value =: val1
!!seq [flow, sequence]: val2
!!map {flow: map}: val3"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"=", !=r"!!value");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2), &=r"mydict");
        expect! (receiver, (1, 3, 4), data, node, scalar, r"val1");
        expect! (receiver, (2, 5, 6), data, node, scalar, r"flow");
        expect! (receiver, (2, 5, 7), data, node, scalar, r"sequence");
        expect! (receiver, (1, 3, 5), data, node, sequence, !=r"!!seq");
        expect! (receiver, (1, 3, 8), data, node, scalar, r"val2");
        expect! (receiver, (2, 9, 10), data, node, scalar, r"flow");
        expect! (receiver, (2, 9, 11), data, node, scalar, r"map");
        expect! (receiver, (1, 3, 9), data, node, mapping, !=r"!!map");
        expect! (receiver, (1, 3, 12), data, node, scalar, r"val3");
        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_1_08 () {
        let src =
r#"---
&myval !!value =: val1
!!seq [flow, sequence]: val2
!!map {flow: map}: val3"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"=", !=r"!!value", &=r"myval");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2));
        expect! (receiver, (1, 3, 4), data, node, scalar, r"val1");
        expect! (receiver, (2, 5, 6), data, node, scalar, r"flow");
        expect! (receiver, (2, 5, 7), data, node, scalar, r"sequence");
        expect! (receiver, (1, 3, 5), data, node, sequence, !=r"!!seq");
        expect! (receiver, (1, 3, 8), data, node, scalar, r"val2");
        expect! (receiver, (2, 9, 10), data, node, scalar, r"flow");
        expect! (receiver, (2, 9, 11), data, node, scalar, r"map");
        expect! (receiver, (1, 3, 9), data, node, mapping, !=r"!!map");
        expect! (receiver, (1, 3, 12), data, node, scalar, r"val3");
        expect! (receiver, (0, 0, 13), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_1_09 () {
        let src =
r#"--- &a1 !!mydict1
key1: !!str val1
key2: &a3 !!mydict2
      key3: val3
      key4: &a4 !!mydict3
            &a5 !!str key5: val5
            key6: val6
            key7: &a7 !!mydict4
                  key8: val8
                  key9:
                        !!str key10: val10
"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (0, 0, 2), data, node, scalar, r"key1");
        expect! (receiver, (0, 0, 3), data, block, map, (0, 0, 2), &=r"a1", !=r"!!mydict1");
        expect! (receiver, (1, 3, 4), data, node, scalar, r"val1", !=r"!!str");
        expect! (receiver, (1, 3, 5), data, node, scalar, r"key2");
        expect! (receiver, (1, 3, 6), data, node, scalar, r"key3");
        expect! (receiver, (1, 3, 7), data, block, map, (1, 3, 6), &=r"a3", !=r"!!mydict2");
        expect! (receiver, (2, 7, 8), data, node, scalar, r"val3");
        expect! (receiver, (2, 7, 9), data, node, scalar, r"key4");
        expect! (receiver, (2, 7, 10), data, node, scalar, r"key5", !=r"!!str", &=r"a5");
        expect! (receiver, (2, 7, 11), data, block, map, (2, 7, 10), &=r"a4", !=r"!!mydict3");
        expect! (receiver, (3, 11, 12), data, node, scalar, r"val5");
        expect! (receiver, (3, 11, 13), data, node, scalar, r"key6");
        expect! (receiver, (3, 11, 14), data, node, scalar, r"val6");
        expect! (receiver, (3, 11, 15), data, node, scalar, r"key7");
        expect! (receiver, (3, 11, 16), data, node, scalar, r"key8");
        expect! (receiver, (3, 11, 17), data, block, map, (3, 11, 16), &=r"a7", !=r"!!mydict4");
        expect! (receiver, (4, 17, 18), data, node, scalar, r"val8");
        expect! (receiver, (4, 17, 19), data, node, scalar, r"key9");
        expect! (receiver, (4, 17, 20), data, node, scalar, r"key10", !=r"!!str");
        expect! (receiver, (4, 17, 21), data, block, map, (4, 17, 20));
        expect! (receiver, (5, 21, 22), data, node, scalar, r"val10");

        expect! (receiver, (0, 0, 23), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_01 () {
        let src =
r#"---
- [ !!str ]
"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 3), node, sequence);
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_02 () {
        let src =
r#"---
- [ !!str , ]
"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 3), node, sequence);
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_03 () {
        let src =
r#"---
- [ ]
"#;

        let receiver = read! (src);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), node, sequence);
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_04 () {
        let src =
r#"---
- { !!str : !!str , }
"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar !=r"!!str" );
        expect! (receiver, (2, 3, 5), data, node, scalar !=r"!!str" );
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_05 () {
        let src =
r#"---
- { !!str : !!str }
"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar !=r"!!str" );
        expect! (receiver, (2, 3, 5), data, node, scalar !=r"!!str" );
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_06 () {
        let src =
r#"---
- { !!str : }
"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar !=r"!!str" );
        expect! (receiver, (2, 3, 5), node, null );
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_07 () {
        let src =
r#"---
- { !!str }
"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar !=r"!!str" );
        expect! (receiver, (2, 3, 5), node, null );
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_08 () {
        let src =
r#"---
- { !!str : !!str , !!str }
"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar !=r"!!str" );
        expect! (receiver, (2, 3, 5), data, node, scalar !=r"!!str");
        expect! (receiver, (2, 3, 6), data, node, scalar !=r"!!str" );
        expect! (receiver, (2, 3, 7), node, null );
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_09 () {
        let src =
r#"---
- { }
"#;

        let receiver = read! (src);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_10 () {
        let src =
r#"---
- !!str"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar !=r"!!str" );
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_11 () {
        let src =
r#"---
- !!str
"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar !=r"!!str" );
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_12 () {
        let src =
r#"---
- !!str
-"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar !=r"!!str" );
        expect! (receiver, (1, 2, 4), node, null);

        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_13 () {
        let src =
r#"---
- !!str :
    !!str"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 4), data, block, map, (1, 2, 3));
        expect! (receiver, (2, 4, 5), data, node, scalar !=r"!!str");
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_14 () {
        let src =
r#"---
- !!str :
    !!str

"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 4), data, block, map, (1, 2, 3));
        expect! (receiver, (2, 4, 5), data, node, scalar !=r"!!str");
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_15 () {
        let src =
r#"---
- !!str :"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 4), data, block, map, (1, 2, 3));
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_16 () {
        let src =
r#"---
- !!str :

"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 4), data, block, map, (1, 2, 3));
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_17 () {
        let src =
r#"---
- ? !!str
  : !!str"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar !=r"!!str");
        expect! (receiver, (2, 3, 5), data, node, scalar !=r"!!str");
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_18 () {
        let src =
r#"---
- ? !!str
  : !!str

"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar !=r"!!str");
        expect! (receiver, (2, 3, 5), data, node, scalar !=r"!!str");
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_19 () {
        let src =
r#"---
- ? !!str
  :"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar !=r"!!str");
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_20 () {
        let src =
r#"---
- ? !!str
  :

"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar !=r"!!str");
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_21 () {
        let src =
r#"---
- ? !!str"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar !=r"!!str");
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_22 () {
        let src =
r#"---
- ? !!str

"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar !=r"!!str");
        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_23 () {
        let src =
r#"---
-"#;

        let receiver = read! (src);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), node, null);
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_24 () {
        let src =
r#"---
-

"#;

        let receiver = read! (src);

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), node, null);
        expect! (receiver, (0, 0, 4), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_25 () {
        let src =
r#"---
- !!str :
-"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 4), data, block, map, (1, 2, 3));
        expect! (receiver, (1, 2, 5), node, null);
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_desolation_26 () {
        let src =
r#"---
- !!str :
-

"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 4), data, block, map, (1, 2, 3));
        expect! (receiver, (1, 2, 5), node, null);
        expect! (receiver, (0, 0, 6), doc, end);

        the_end! (receiver);
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


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);

        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 3), node, sequence);

        expect! (receiver, (2, 5, 6), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 5), node, sequence);

        expect! (receiver, (1, 2, 7), node, sequence);

        expect! (receiver, (2, 8, 9), data, node, scalar !=r"!!str");
        expect! (receiver, (2, 8, 10), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 8), data, node, mapping);

        expect! (receiver, (2, 11, 12), data, node, scalar !=r"!!str");
        expect! (receiver, (2, 11, 13), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 11), data, node, mapping);

        expect! (receiver, (2, 14, 15), data, node, scalar !=r"!!str");
        expect! (receiver, (2, 14, 16), node, null);
        expect! (receiver, (1, 2, 14), data, node, mapping);

        expect! (receiver, (2, 17, 18), data, node, scalar !=r"!!str");
        expect! (receiver, (2, 17, 19), node, null);
        expect! (receiver, (1, 2, 17), data, node, mapping);

        expect! (receiver, (1, 2, 20), data, node, mapping);

        expect! (receiver, (1, 2, 21), data, node, scalar !=r"!!str");

        expect! (receiver, (1, 2, 22), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 23), data, block, map, (1, 2, 22));
        expect! (receiver, (2, 23, 24), data, node, scalar !=r"!!str");

        expect! (receiver, (1, 2, 25), data, node, scalar !=r"!!str");
        expect! (receiver, (1, 2, 26), data, block, map, (1, 2, 25));

        expect! (receiver, (1, 2, 27), data, node, mapping);
        expect! (receiver, (2, 27, 28), data, node, scalar !=r"!!str");
        expect! (receiver, (2, 27, 29), data, node, scalar !=r"!!str");

        expect! (receiver, (1, 2, 30), data, node, mapping);
        expect! (receiver, (2, 30, 31), data, node, scalar !=r"!!str");

        expect! (receiver, (1, 2, 32), data, node, mapping);
        expect! (receiver, (2, 32, 33), data, node, scalar !=r"!!str");

        expect! (receiver, (1, 2, 34), node, null);

        expect! (receiver, (0, 0, 35), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_07_extra () {
        let src =
r#"|
  foo 
 
  	 bar

  baz
"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, r"foo ");
        expect! (receiver, (1, 2, 4), data, literal, "\n");
        expect! (receiver, (1, 2, 5), data, literal, "\n");
        expect! (receiver, (1, 2, 6), data, literal, "\t bar");
        expect! (receiver, (1, 2, 7), data, literal, "\n");
        expect! (receiver, (1, 2, 8), data, literal, "\n");
        expect! (receiver, (1, 2, 9), data, literal, r"baz");
        expect! (receiver, (1, 2, 10), data, literal, "\n");
        expect! (receiver, (0, 0, 2), node, block, close);
        expect! (receiver, (0, 0, 11), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_06_06_extra () {
        let src =
r#"|-
  trimmed
  
 

  as
  space"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, r"trimmed");
        expect! (receiver, (1, 2, 4), data, literal, "\n");
        expect! (receiver, (1, 2, 5), data, literal, "\n\n\n");
        expect! (receiver, (1, 2, 6), data, literal, r"as");
        expect! (receiver, (1, 2, 7), data, literal, "\n");
        expect! (receiver, (1, 2, 8), data, literal, r"space");
        expect! (receiver, (0, 0, 2), node, block, close);
        expect! (receiver, (0, 0, 9), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_02_15_extra () {
        let src =
r"|
 Sammy Sosa completed another
 fine season with great stats.

   63 Home Runs
   0.288 Batting Average

 What a year!";

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, r"Sammy Sosa completed another");
        expect! (receiver, (1, 2, 4), data, literal, "\n");
        expect! (receiver, (1, 2, 5), data, literal, r"fine season with great stats.");
        expect! (receiver, (1, 2, 6), data, literal, "\n");
        expect! (receiver, (1, 2, 7), data, literal, "\n");
        expect! (receiver, (1, 2, 8), data, literal, r"  63 Home Runs");
        expect! (receiver, (1, 2, 9), data, literal, "\n");
        expect! (receiver, (1, 2, 10), data, literal, "  0.288 Batting Average");
        expect! (receiver, (1, 2, 11), data, literal, "\n");
        expect! (receiver, (1, 2, 12), data, literal, "\n");
        expect! (receiver, (1, 2, 13), data, literal, "What a year!");
        expect! (receiver, (0, 0, 2), node, block, close);
        expect! (receiver, (0, 0, 14), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn example_02_14_extra () {
        let src =
r"--- |
  Mark McGwire's
  year was crippled
  by a knee injury.";


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, block, open);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, literal, r"Mark McGwire's");
        expect! (receiver, (1, 2, 4), data, literal, "\n");
        expect! (receiver, (1, 2, 5), data, literal, r"year was crippled");
        expect! (receiver, (1, 2, 6), data, literal, "\n");
        expect! (receiver, (1, 2, 7), data, literal, r"by a knee injury.");
        expect! (receiver, (0, 0, 2), node, block, close);
        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }


    #[test]
    fn extra_question_01 () {
        let src =
r#"? key : value"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"key");
        expect! (receiver, (1, 2, 4), data, node, scalar, r"value");

        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_question_02 () {
        let src =
r#"? key
: value"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (1, 2, 3), data, node, scalar, r"key");
        expect! (receiver, (1, 2, 4), data, node, scalar, r"value");

        expect! (receiver, (0, 0, 5), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_question_03 () {
        let src =
r#"
- ? key1 : value1
  ? key2 : value2"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar, r"key1");
        expect! (receiver, (2, 3, 5), data, node, scalar, r"value1");
        expect! (receiver, (2, 3, 6), data, node, scalar, r"key2");
        expect! (receiver, (2, 3, 7), data, node, scalar, r"value2");

        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_question_04 () {
        let src =
r#"
- ? key1
  : value1
  ? key2
  : value2"#;

        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar, r"key1");
        expect! (receiver, (2, 3, 5), data, node, scalar, r"value1");
        expect! (receiver, (2, 3, 6), data, node, scalar, r"key2");
        expect! (receiver, (2, 3, 7), data, node, scalar, r"value2");

        expect! (receiver, (0, 0, 8), doc, end);

        the_end! (receiver);
    }



    #[test]
    fn extra_question_05 () {
        let src =
r#"
- ? dict1_key: dict1_val
  : dict2_key: dict2_val
"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar, r"dict1_key");
        expect! (receiver, (2, 3, 5), data, block, map, (2, 3, 4));
        expect! (receiver, (3, 5, 6), data, node, scalar, r"dict1_val");
        expect! (receiver, (2, 3, 7), data, node, scalar, r"dict2_key");
        expect! (receiver, (2, 3, 8), data, block, map, (2, 3, 7));
        expect! (receiver, (3, 8, 9), data, node, scalar, r"dict2_val");
        expect! (receiver, (0, 0, 10), doc, end);
        the_end! (receiver);
    }



    #[test]
    fn extra_question_06 () {
        let src =
r#"
- ? dict1_key: dict1_val
  : dict2_key: dict2_val
  ? dict3_key: dict3_val
  : dict4_key: dict4_val
"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);

        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar, r"dict1_key");
        expect! (receiver, (2, 3, 5), data, block, map, (2, 3, 4));
        expect! (receiver, (3, 5, 6), data, node, scalar, r"dict1_val");
        expect! (receiver, (2, 3, 7), data, node, scalar, r"dict2_key");
        expect! (receiver, (2, 3, 8), data, block, map, (2, 3, 7));
        expect! (receiver, (3, 8, 9), data, node, scalar, r"dict2_val");
        expect! (receiver, (2, 3, 10), data, node, scalar, r"dict3_key");
        expect! (receiver, (2, 3, 11), data, block, map, (2, 3, 10));
        expect! (receiver, (3, 11, 12), data, node, scalar, r"dict3_val");
        expect! (receiver, (2, 3, 13), data, node, scalar, r"dict4_key");
        expect! (receiver, (2, 3, 14), data, block, map, (2, 3, 13));
        expect! (receiver, (3, 14, 15), data, node, scalar, r"dict4_val");
        expect! (receiver, (0, 0, 16), doc, end);
        the_end! (receiver);
    }



    #[test]
    fn extra_question_07 () {
        let src =
r#"
- ? dict1_key:
  : dict2_key:
"#;


        let receiver = read! (src);
        let mut data = data! ();

        expect! (receiver, (0, 0, 1), doc, start);
        expect! (receiver, (0, 0, 2), node, sequence);
        expect! (receiver, (1, 2, 3), data, node, mapping);
        expect! (receiver, (0, 0, 0), datum, data);
        expect! (receiver, (2, 3, 4), data, node, scalar, r"dict1_key");
        expect! (receiver, (2, 3, 5), data, block, map, (2, 3, 4));
        expect! (receiver, (3, 5, 6), node, null);
        expect! (receiver, (2, 3, 7), data, node, scalar, r"dict2_key");
        expect! (receiver, (2, 3, 8), data, block, map, (2, 3, 7));
        expect! (receiver, (3, 8, 9), node, null);
        expect! (receiver, (0, 0, 10), doc, end);
        the_end! (receiver);
    }
}
