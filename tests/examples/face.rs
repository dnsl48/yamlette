#[cfg (all (test, not (feature = "dev")))]
mod stable {
    extern crate yamlette;

    #[test]
    fn example_02_01_block () {
        let should_be = 
            Some (r#"- Mark McGwire
- Sammy Sosa
- Ken Griffey
"#.to_string ());

        yamlette! ( read ; should_be.as_ref ().unwrap ().clone () ; [[ [ (mark:&str), (sammy:&str), (ken:&str) ] ]] ; { book: book, result: result } );

        assert! (result.is_ok ());
        assert_eq! (1, book.volumes.len ());

        assert_eq! (mark, Some ("Mark McGwire"));
        assert_eq! (sammy, Some ("Sammy Sosa"));
        assert_eq! (ken, Some ("Ken Griffey"));

        let result = yamlette! ( write ; [[ [ (mark.unwrap ().to_string ()), (sammy.unwrap ().to_string ()), (ken.unwrap ().to_string ()) ] ]] ).ok ();

        assert_eq! (should_be, result);
    }


    #[test]
    fn example_02_01_block_ignore_result () {
        let should_be = 
            Some (r#"- Mark McGwire
- Sammy Sosa
- Ken Griffey
"#.to_string ());

        yamlette! ( read ; should_be.as_ref ().unwrap ().clone () ; [[ [ (mark:&str), (sammy:&str), (ken:&str) ] ]] );

        assert_eq! (mark, Some ("Mark McGwire"));
        assert_eq! (sammy, Some ("Sammy Sosa"));
        assert_eq! (ken, Some ("Ken Griffey"));

        let result = yamlette! ( write ; [[ [ (mark.unwrap ().to_string ()), (sammy.unwrap ().to_string ()), (ken.unwrap ().to_string ()) ] ]] ).ok ();

        assert_eq! (should_be, result);
    }


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


    #[test]
    fn example_02_01_block_custom_schema () {
        let should_be = 
            Some (r#"- Mark McGwire
- Sammy Sosa
- Ken Griffey
"#.to_string ());

        let result = yamlette! ( write ; [[ [ "Mark McGwire", "Sammy Sosa", "Ken Griffey" ] ]] ; { schema: yamlette::model::schema::core::Core::new () } ).ok ();

        assert_eq! (should_be, result);
    }
}
