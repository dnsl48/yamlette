#[cfg (all (test, not (feature = "dev")))]
mod stable {
    extern crate yamlette;

    #[test]
    fn example_extra_yaml_rust () {
        let should_be = Some (r#"foo:
    - list1
    - list2
bar:
    - 1
    - 2.0"#);

        yamlette! ( read ; should_be.as_ref ().unwrap ().clone () ; [[ {
                "foo" => [ (list1:&str), (list2:&str) ],
                "bar" => [ (unu:u8), (dua:f32) ]
        } ]] ; { book: book, result: result } );

        assert! (result.is_ok ());
        assert_eq! (1, book.volumes.len ());

        assert_eq! (list1, Some ("list1"));
        assert_eq! (list2, Some ("list2"));
        assert_eq! (unu, Some (1u8));
        assert_eq! (dua, Some (2.0));
    }


    #[test]
    fn example_extra_yaml_rust_lazy () {
        let should_be = r#"foo:
    - list1
    - list2
bar:
    - 1
    - 2.0"#;

        let mut payload = yamlette! ( init ; reader );

        yamlette! ( read ; warm ; &mut payload ; should_be ; [[ {
            "foo" => [ (list1:&str), (list2:&str) ],
            "bar" => [ (unu:u8), (dua:f32) ]
        } ]] ; {} );

        assert_eq! (list1, Some ("list1"));
        assert_eq! (list2, Some ("list2"));
        assert_eq! (unu, Some (1u8));
        assert_eq! (dua, Some (2.0));

        yamlette! ( read ; warm ; &mut payload ; should_be ; [[ {
            "foo" => [ (two_list1:&str), (two_list2:&str) ],
            "bar" => [ (two_unu:u8), (two_dua:f32) ]
        } ]] ; {} );

        assert_eq! (two_list1, Some ("list1"));
        assert_eq! (two_list2, Some ("list2"));
        assert_eq! (two_unu, Some (1u8));
        assert_eq! (two_dua, Some (2.0));
    }

    #[test]
    fn example_02_01_block () {
        let should_be = 
            Some (r#"- Mark McGwire
- Sammy Sosa
- Ken Griffey
"#);

        yamlette! ( read ; should_be.as_ref ().unwrap ().clone () ; [[ [ (mark:&str), (sammy:&str), (ken:&str) ] ]] ; { book: book, result: result } );

        assert! (result.is_ok ());
        assert_eq! (1, book.volumes.len ());

        assert_eq! (mark, Some ("Mark McGwire"));
        assert_eq! (sammy, Some ("Sammy Sosa"));
        assert_eq! (ken, Some ("Ken Griffey"));

        let result = yamlette! ( write ; [[ [ (mark.unwrap ().to_string ()), (sammy.unwrap ().to_string ()), (ken.unwrap ().to_string ()) ] ]] ).ok ();

        assert_eq! (should_be.map (|b| b.to_string ()), result);
    }


    #[test]
    fn example_02_01_block_sage () {
        let should_be = 
            Some (r#"- Mark McGwire
- Sammy Sosa
- Ken Griffey
"#);

        yamlette! ( sage ; should_be.as_ref ().unwrap ().clone () ; [[ [ (mark:&str), (sammy:&str), (ken:&str) ] ]] ; { book: book, result: result } );

        assert! (result.is_ok ());
        assert_eq! (1, book.volumes.len ());

        assert_eq! (mark, Some ("Mark McGwire"));
        assert_eq! (sammy, Some ("Sammy Sosa"));
        assert_eq! (ken, Some ("Ken Griffey"));

        let result = yamlette! ( write ; [[ [ (mark.unwrap ().to_string ()), (sammy.unwrap ().to_string ()), (ken.unwrap ().to_string ()) ] ]] ).ok ();

        assert_eq! (should_be.map (|b| b.to_string ()), result);
    }


    #[test]
    fn example_02_01_block_ignore_result () {
        let should_be = 
            Some (r#"- Mark McGwire
- Sammy Sosa
- Ken Griffey
"#);

        yamlette! ( read ; should_be.as_ref ().unwrap ().clone () ; [[ [ (mark:&str), (sammy:&str), (ken:&str) ] ]] );

        assert_eq! (mark, Some ("Mark McGwire"));
        assert_eq! (sammy, Some ("Sammy Sosa"));
        assert_eq! (ken, Some ("Ken Griffey"));

        let result = yamlette! ( write ; [[ [ (mark.unwrap ().to_string ()), (sammy.unwrap ().to_string ()), (ken.unwrap ().to_string ()) ] ]] ).ok ();

        assert_eq! (should_be.map (|b| b.to_string ()), result);
    }


    #[test]
    fn example_02_01_block_ignore_result_sage () {
        let should_be = 
            Some (r#"- Mark McGwire
- Sammy Sosa
- Ken Griffey
"#);

        yamlette! ( sage ; should_be.as_ref ().unwrap ().clone () ; [[ [ (mark:&str), (sammy:&str), (ken:&str) ] ]] );

        assert_eq! (mark, Some ("Mark McGwire"));
        assert_eq! (sammy, Some ("Sammy Sosa"));
        assert_eq! (ken, Some ("Ken Griffey"));

        let result = yamlette! ( write ; [[ [ (mark.unwrap ().to_string ()), (sammy.unwrap ().to_string ()), (ken.unwrap ().to_string ()) ] ]] ).ok ();

        assert_eq! (should_be.map (|b| b.to_string ()), result);
    }


/*
    #[test]
    fn example_02_01_block_custom_charset () {
        let should_be = 
            Some (r#"- Mark McGwire
- Sammy Sosa
- Ken Griffey
"#.to_string ());

        let result = yamlette! ( write ; [[ [ "Mark McGwire", "Sammy Sosa", "Ken Griffey" ] ]] ; { charset: yamlette::txt::get_charset_utf8 () } ).ok ();

        assert_eq! (should_be, result);
    }
*/


/*
    #[test]
    fn example_02_01_block_custom_encoding () {
        let should_be = 
            Some (r#"- Mark McGwire
- Sammy Sosa
- Ken Griffey
"#.to_string ());

        let result = yamlette! ( write ; [[ [ "Mark McGwire", "Sammy Sosa", "Ken Griffey" ] ]] ; { encoding: UTF8 } ).ok ();

        assert_eq! (should_be, result);
    }
*/


    #[test]
    fn example_02_01_block_custom_schema () {
        let should_be = 
            Some (r#"- Mark McGwire
- Sammy Sosa
- Ken Griffey
"#);

        // let cset = yamlette::txt::charset::get_charset_utf8 ();
        let schema = yamlette::model::schema::core::Core::new ();
        let result = yamlette! ( write ; [[ [ "Mark McGwire", "Sammy Sosa", "Ken Griffey" ] ]] ; { schema: schema } ).ok ();

        assert_eq! (should_be.map (|b| b.to_string ()), result);
    }
}
