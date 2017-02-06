macro_rules! composer {
    ($src:expr) => {{
        let charset = get_charset_utf8 ();
        let (sender, receiver) = channel ();

        let composer = Composer::new (&charset, receiver, build_complete_schema ());
        let mut reader = Reader::new (SliceReader::new ($src.as_bytes ()), Tokenizer::new (charset), sender);

        reader.read ().unwrap_or_else (|err| { assert! (false, format! ("Unexpected result: {}, :{}", err, err.position)); });

        let composer = composer.unwrap ();

        composer
    }}
}




macro_rules! wait {
    ($composer:expr) => {{
        let result = $composer.join ();
        assert! (result.is_ok ()); // check the thread didn't panic

        let result = result.unwrap ();
        if let Err (ref err) = result { println! ("Composing error: {:?}", err); }

        assert! (result.is_ok ()); // documents have been built

        let book = result.ok ().unwrap ();

        book
    }}
}




macro_rules! chapter {
    ($book:expr, $chapter_idx:expr, collection, seq, $collection_len:expr) => {{
        if let Some (chapter) = $book.get_chapter ($chapter_idx) {
            assert! (!chapter.is_empty ());

            let root = chapter.get_root_node ();
            assert! (root.is_some ());

            let &(_, ref root_node) = root.unwrap ();

            if let &Node::Collection (tag_id, Collection::Seq (ref collection)) = root_node {} else {
                assert_eq! (tag_id, "tag:yaml.org,2002:seq");
                assert_eq! (collection.len (), $collection_len);
            } else { assert! (false, "Not a Collection::Seq") }
        } else { assert! (false, format ("Undefined chapter {}", $chapter_idx)) }
    }}
}




macro_rules! book {

    ($book:expr, chapter = $chapter:expr, children = $children:expr, children_cnt = $children_cnt:expr, value = ($tag:expr => $value:expr)) => {{
        match $tag {
            "tag:yaml.org,2002:str" => {
                if let Some (children_index) = $children.get ($children_cnt) {
                    if let Some ( &(_, Node::Scalar ("tag:yaml.org,2002:str", Scalar::Str (ref string))) ) = $chapter.get_node_by_index (*children_index) {
                        assert_eq! (string, $value);
                    } else { assert! (false, format! ("cannot extract a child. Expected !str, got {:?}", $chapter.get_node_by_index (*children_index))) }
                } else { assert! (false, "not enough children") }
            }

            _ => assert! (false, format! ("Incorrect tag '{}'", $tag))
        };

        $children_cnt += 1;
    }};


/*
    ($book:expr, chapter = $chapter:expr, children = $children:expr, children_cnt = $children_cnt:expr, value = (($tag1:expr => $val1:expr), ($tag2:expr => $value2:expr))) => {{
        if let Some (children_index) = $children.get ($children_cnt) {

            

            if let Some ( &(_, Node::Scalar ("tag:yaml.org,2002:str", Scalar::Str (ref string))) ) = $chapter.get_node_by_index (*children_index) {
                assert_eq! (string, $value);
            } else { assert! (false, format! ("cannot extract a child. Expected !str, got {:?}", $chapter.get_node_by_index (*children_index))) }
        } else { assert! (false, "not enough children") }

        $children_cnt += 1;
    }};
*/



    ($book:expr, chapter = $chapter:expr, children = $children:expr, children_cnt = $children_cnt:expr, value = [$($value:tt),*]) => {{
        $(book! ($book, chapter = $chapter, children = $children, children_cnt = $children_cnt, value = $value);)*
    }};



    ($book:expr, chapter = $chapter:expr, chapter_tree = ($tag:expr => $value:tt)) => {{
        match $tag {
            "tag:yaml.org,2002:seq" => {
                if let Some ( &(_, Node::Collection ("tag:yaml.org,2002:seq", Collection::Seq (ref children))) ) = $chapter.get_root_node () {
                    let mut children_cnt = 0;

                    book! ($book, chapter = $chapter, children = children, children_cnt = children_cnt, value = $value);

                    assert_eq! (children_cnt, children.len ());
                } else { assert! (false, "Incorrect root node of the chapter") }
            }
/*
            "tag:yaml.org,2002:map" => {
                if let Some ( &(_, Node::Collection ("tag:yaml.org,2002:map", Collection::Map (ref children))) ) = $chapter.get_root_node () {
                    let mut children_cnt = 0;

                    book! ($book, chapter = $chapter, children = children, children_cnt = children_cnt, value = $value);

                    assert_eq! (children_cnt, children.len ());
                } else { assert! (false, "Incorrect root node of the chapter") }
            }
*/

            _ => assert! (false, format! ("Unimplemented tag type for a chapter '{}'", $tag))
        }
    }};

    ($book:expr, chapter_index = $chapter_index:expr, $chapter_tree:tt) => {{
        if let Some (chapter) = $book.get_chapter ($chapter_index) {
            book! ($book, chapter = chapter, chapter_tree = $chapter_tree)
        } else { assert! (false, format! ("Chapter {} does not exist", $chapter_index)) }

        $chapter_index += 1;
    }};

    ($book:expr, $($chapters:tt)+) => {{
        let mut chapter_index: usize = 0;

        $(book! ($book, chapter_index = chapter_index, $chapters);)*;

        assert_eq! ($book.len (), chapter_index);
    }};
}




#[cfg (test)]
mod stable {
    extern crate fraction;
    extern crate skimmer;
    extern crate yamlette;

    use self::fraction::Fraction;

    use self::skimmer::reader::SliceReader;

    use self::yamlette::txt::get_charset_utf8;
    use self::yamlette::tokenizer::Tokenizer;
    use self::yamlette::reader::Reader;
    use self::yamlette::composer::Composer;
    use self::yamlette::book::Node;
    use self::yamlette::schema::{ Collection, Scalar };

    use self::yamlette::schema::build_complete_schema;

    use std::sync::mpsc::channel;



    #[test]
    fn example_2_01 () {
                let src =
r"- Mark McGwire
- Sammy Sosa
- Ken Griffey";

        let composer = composer! (src);
        let book = wait! (composer);

        book! ( &book,
            ("tag:yaml.org,2002:seq" => [
                ("tag:yaml.org,2002:str" => "Mark McGwire"),
                ("tag:yaml.org,2002:str" => "Sammy Sosa"),
                ("tag:yaml.org,2002:str" => "Ken Griffey")
            ])
        )
    }


/*
    #[test]
    fn example_2_02 () {
        let src =
r"hr:  65    # Home runs
avg: 0.278 # Batting average
rbi: 147   # Runs Batted In";

        let composer = composer! (src);
        let book = wait! (composer);


        book! ( &book,
            ("tag:yaml.org,2002:map", [
                (("tag:yaml.org,2002:str" => "hr"), ("tag:yaml.org,2002:int" => 65)),
                (("tag:yaml.org,2002:str" => "avg"), ("tag:yaml.org,2002:float" => Fraction::new (278, 1000))),
                (("tag:yaml.org,2002:str" => "rbi"), ("tag:yaml.org,2002:int" => 147))
            ])
        )
    }
*/
}
