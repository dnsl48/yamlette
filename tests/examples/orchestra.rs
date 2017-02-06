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



macro_rules! check {
    ( $expect:ident ; $rules:tt ) => {{
        let result = yamlette! ( write ; $rules );

        assert! (result.is_ok ());
        assert_eq! ($expect, result.ok ().unwrap ());
    }};
}



macro_rules! ex {
    ( $title:ident ; $expect:expr ; $rules:tt ; $( $import:path ),* ) => {
        #[test]
        fn $title () {
            $( use $import; )*

            let expect = $expect;

            let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

            yamlette_compose! ( orchestra ; orc ; $rules );
            let result = unsafe { String::from_utf8_unchecked (orc.listen ().ok ().unwrap ()) };

            assert_eq! (expect, result);
        }
    };

    ( $title:ident ; $expect:expr ; $rules:tt ) => {
        #[test]
        fn $title () {
            let expect = $expect;

            let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

            yamlette_compose! ( orchestra ; orc ; $rules );
            let result = unsafe { String::from_utf8_unchecked (orc.listen ().ok ().unwrap ()) };

            assert_eq! (expect, result);
        }
    };
}


#[cfg (all (test, not (feature = "dev")))]
mod stable {

extern crate skimmer;
extern crate yamlette;

use self::yamlette::model::schema::core::Core;
use self::yamlette::txt::get_charset_utf8;

use self::yamlette::orchestra::Orchestra;
use self::yamlette::orchestra::chord::{ Omap, Set };

use std::collections::{ BTreeMap, HashMap };



fn get_schema () -> Core { Core::new () }



#[test]
fn example_02_01_block () {
    let should_be = 
r#"- Mark McGwire
- Sammy Sosa
- Ken Griffey
"#;

    let mark = "Mark McGwire";
    let sammy = "Sammy Sosa";
    let ken = "Ken Griffey";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    yamlette_compose! ( orchestra ; orc ; [[ [ mark, sammy, ken ] ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn example_02_01_vec () {
    let should_be = 
r#"- Mark McGwire
- Sammy Sosa
- Ken Griffey
"#;

    let seq = vec! ["Mark McGwire", "Sammy Sosa", "Ken Griffey"];

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    yamlette_compose! ( orchestra ; orc ; [[ seq ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn example_02_01_block_quoted () {
    let should_be = 
r#"- 'Mark McGwire'
- 'Sammy Sosa'
- 'Ken Griffey'
"#;

    let mark = "Mark McGwire";
    let sammy = "Sammy Sosa";
    let ken = "Ken Griffey";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::yaml::str::ForceQuotes;

    yamlette_compose! ( orchestra ; orc ; [[ [# ForceQuotes (true) => mark, sammy, ken ] ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn example_02_01_block_doublequoted () {
    let should_be = 
r#"- "Mark McGwire"
- "Sammy Sosa"
- "Ken Griffey"
"#;

    let mark = "Mark McGwire";
    let sammy = "Sammy Sosa";
    let ken = "Ken Griffey";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::yaml::str::{ ForceQuotes, PreferDoubleQuotes };

    yamlette_compose! ( orchestra ; orc ; [[ [# ForceQuotes (true), PreferDoubleQuotes (true) => mark, sammy, ken ] ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn example_02_01_flow () {
    let should_be = 
r#"[ Mark McGwire, Sammy Sosa, Ken Griffey ]"#;

    let mark = "Mark McGwire";
    let sammy = "Sammy Sosa";
    let ken = "Ken Griffey";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::style::Flow;

    yamlette_compose! ( orchestra ; orc ; [[ # Flow (true) =>
        [
            mark,
            sammy,
            ken
        ]
    ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}




#[test]
fn example_02_01_flow_compact () {
    let should_be = 
r#"[Mark McGwire,Sammy Sosa,Ken Griffey]"#;

    let mark = "Mark McGwire";
    let sammy = "Sammy Sosa";
    let ken = "Ken Griffey";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::style::{ Flow, Compact };

    yamlette_compose! ( orchestra ; orc ; [ # Flow (true), Compact (true) => [ [ mark, sammy, ken ] ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn example_02_01_flow_multiline () {
    let should_be = 
r#"[
  Mark McGwire,
  Sammy Sosa,
  Ken Griffey
]"#;

    let mark = "Mark McGwire";
    let sammy = "Sammy Sosa";
    let ken = "Ken Griffey";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::style::{ Flow, Multiline };

    yamlette_compose! ( orchestra ; orc ; [ # Flow (true) => [ # Multiline (true) => [ mark, sammy, ken ] ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn example_02_01_flow_multiline_indent4 () {
    let should_be = 
r#"[
    Mark McGwire,
    Sammy Sosa,
    Ken Griffey
]"#;

    let mark = "Mark McGwire";
    let sammy = "Sammy Sosa";
    let ken = "Ken Griffey";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::style::{ Flow, Multiline, Indent };

    yamlette_compose! ( orchestra ; orc ; [ # Flow (true) => [ # Multiline (true) => ( # Indent (4) => [ mark, sammy, ken ] ) ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn extra_01_empty () {
    let should_be = 
r#"[]"#;

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    yamlette_compose! ( orchestra ; orc ; [[
        []
    ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn extra_01_flow_threshold () {
    let should_be = 
r#"[ abcd, abcd, abcd,
abcd, abcd, abcd, abcd,
abcd, abcd ]"#;

    let data = "abcd";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::style::{ Flow, RespectThreshold, Threshold };

    yamlette_compose! ( orchestra ; orc ; [[ # Flow (true), RespectThreshold (true), Threshold (24) =>
        [
            data, data, data, data, data, data,
            data, data, data
        ]
    ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn extra_01_flow_threshold_compact () {
    let should_be = 
r#"[abcd,abcd,abcd,abcd,
abcd,abcd,abcd,abcd,abcd]"#;

    let data = "abcd";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::style::{ Flow, Compact, RespectThreshold, Threshold };

    yamlette_compose! ( orchestra ; orc ; [[ # Flow (true), Compact (true), RespectThreshold (true), Threshold (24) =>
        [
            data, data, data, data, data, data,
            data, data, data
        ]
    ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn extra_01_volume_flow_threshold_compact () {
    let should_be = 
r#"[abcd,abcd,abcd,abcd,
abcd,abcd,abcd,abcd,abcd]"#;

    let data = "abcd";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::style::{ Flow, Compact, RespectThreshold, Threshold };

    yamlette_compose! ( orchestra ; orc ; [[# Flow (true), Compact (true), RespectThreshold (true), Threshold (24) =>
        [
            data, data, data, data, data, data,
            data, data, data
        ]
    ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn example_02_02_block () {
    let should_be = 
r"hr: 65
avg: 0.278
rbi: 147
";

    let hr = 65;
    let avg = 0.278;
    let rbi = 147;

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    yamlette_compose! ( orchestra ; orc ; [[
        {
            "hr" : hr,
            "avg": avg,
            "rbi": rbi
        }
    ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn example_02_02_map () {
    let should_be_1 =
r"hr: 65
avg: 0.278
rbi: 147
";

    let should_be_2 =
r"avg: 0.278
hr: 65
rbi: 147
";

    let should_be_3 =
r"rbi: 147
hr: 65
avg: 0.278
";

    let should_be_4 =
r"avg: 0.278
rbi: 147
hr: 65
";

    let should_be_5 =
r"rbi: 147
avg: 0.278
hr: 65
";

    let should_be_6 =
r"hr: 65
rbi: 147
avg: 0.278
";

    let mut map = HashMap::with_capacity (3);

    map.insert ("hr", "65");
    map.insert ("avg", "0.278");
    map.insert ("rbi", "147");

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    yamlette_compose! ( orchestra ; orc ; [[ map ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    if result != should_be_1 && result != should_be_2 && result != should_be_3 && result != should_be_4 && result != should_be_5 && result != should_be_6 {
        assert_eq! ("", result);
    }
}



#[test]
fn extra_02_empty () {
    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    yamlette_compose! ( orchestra ; orc ; [[ {} ]] );
    let maybe_music = orc.listen ();
    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };
    assert_eq! ("{}", result);


    use yamlette::model::style::{ IssueTag };

    yamlette_compose! ( orchestra ; orc ; [[ # IssueTag (true) => {} ]] );
    let maybe_music = orc.listen ();
    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };
    assert_eq! ("!!map {}", result);
}



#[test]
fn extra_03_more_emptiness_flow_multiline () {
    let should_be = 
r#"[
  {},
  [],
  {
    {}: {},
    {}: [],
    []: {},
    []: []
  }
]"#;

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::style::{ Flow, Multiline };

    yamlette_compose! ( orchestra ; orc ; [[# Flow (true), Multiline (true) =>
        [
            {},
            [],
            {
                {} : {},
                {} : [],
                [] : {},
                [] : []
            }
        ]
    ]] );


    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn extra_03_more_emptiness_flow () {
    let should_be = 
r#"[ {}, [], { {}: {}, {}: [], []: {}, []: [] } ]"#;

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::style::{ Flow };

    yamlette_compose! ( orchestra ; orc ; [[# Flow (true) =>
        [
            {},
            [],
            {
                {} : {},
                {} : [],
                [] : {},
                [] : []
            }
        ]
    ]] );


    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn extra_03_more_emptiness_flow_compact () {
    let should_be = 
r#"[{},[],{{}: {},{}: [],[]: {},[]: []}]"#;

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::style::{ Flow, Compact };

    yamlette_compose! ( orchestra ; orc ; [[# Flow (true), Compact (true) =>
        [
            {},
            [],
            {
                {} : {},
                {} : [],
                [] : {},
                [] : []
            }
        ]
    ]] );


    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}




#[test]
fn extra_04_flow_respect_threshold () {
    let should_be = 
r#"{ a: b, c: d, e: f,
g: h, i: j, k: l, m:
n, o: p, q: r, s: t,
u: v, w: x, y: z }"#;

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::style::{ Flow, RespectThreshold, Threshold };

    yamlette_compose! ( orchestra ; orc ; [[# Flow (true), RespectThreshold (true), Threshold (20) =>
        { "a": "b", "c": "d", "e": "f", "g": "h", "i": "j", "k": "l", "m": "n", "o": "p", "q": "r", "s": "t", "u": "v", "w": "x", "y": "z" }
    ]] );


    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn extra_04_compact_flow_respect_threshold () {
    let should_be = 
r#"{a: b,c: d,e: f,g:
h,i: j,k: l,m: n,o:
p,q: r,s: t,u: v,w:
x,y: z}"#;

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::style::{ Compact, Flow, RespectThreshold, Threshold };

    yamlette_compose! ( orchestra ; orc ; [[# Compact (true), Flow (true), RespectThreshold (true), Threshold (20) =>
        { "a": "b", "c": "d", "e": "f", "g": "h", "i": "j", "k": "l", "m": "n", "o": "p", "q": "r", "s": "t", "u": "v", "w": "x", "y": "z" }
    ]] );


    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}


#[test]
fn example_02_01_block_diryaml () {
    let should_be = 
r#"%YAML 1.2
---
- Mark McGwire
- Sammy Sosa
- Ken Griffey
...
"#;

    let mark = "Mark McGwire";
    let sammy = "Sammy Sosa";
    let ken = "Ken Griffey";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();


    yamlette_compose! ( orchestra ; orc ; [ % YAML => [
                                    [ mark, sammy, ken ]
                                 ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}


#[test]
fn example_02_01_block_volumes_03 () {
    let should_be = 
r#"- Mark McGwire
- Sammy Sosa
- Ken Griffey
...
- Sammy Sosa
- Ken Griffey
...
- Ken Griffey
...
- Sammy Sosa
...
- Mark McGwire
- Sammy Sosa
...
- Mark McGwire
"#;

    let mark = "Mark McGwire";
    let sammy = "Sammy Sosa";
    let ken = "Ken Griffey";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();


    yamlette_compose! ( orchestra ; orc ; [
        [
            [ mark, sammy, ken ]
        ],
        [
            [ sammy, ken ]
        ],
        [
            [ken]
        ],
        [
            [sammy]
        ],
        [
            [mark, sammy]
        ],
        [
            [mark]
        ]
    ] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn example_02_01_block_volumes_02 () {
    let should_be = 
r#"- Mark McGwire
...
- Sammy Sosa
"#;

    let mark = "Mark McGwire";
    let sammy = "Sammy Sosa";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    yamlette_compose! ( orchestra ; orc ; [
        [
            [mark]
        ],
        [
            [sammy]
        ]
    ] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}


#[test]
fn example_02_01_block_volumes_01 () {
    let should_be = 
r#"- Mark McGwire
"#;

    let mark = "Mark McGwire";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    yamlette_compose! ( orchestra ; orc ; [
        [
            [mark]
        ]
    ] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn example_02_01_block_dir_tags_01 () {
    let should_be = 
r#"%TAG !aloha! http://yamlette.org,2015:aloha/
---
- Mark McGwire
- Sammy Sosa
- Ken Griffey
...
"#;

    let mark = "Mark McGwire";
    let sammy = "Sammy Sosa";
    let ken = "Ken Griffey";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    yamlette_compose! ( orchestra ; orc ; [[ % (TAG ; "!aloha!", "http://yamlette.org,2015:aloha/") => [ mark, sammy, ken ] ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}




#[test]
fn example_02_01_block_dirs_01 () {
    let should_be = 
r#"%YAML 1.2
%TAG !test! http://yamlette.org,2015:test/
%TAG !aloha! http://yamlette.org,2015:aloha/
---
- Mark McGwire
- Sammy Sosa
- Ken Griffey
...
%YAML 1.2
%TAG !test! http://yamlette.org,2015:test/
---
- Sammy Sosa
- Ken Griffey
...
%TAG !test! http://yamlette.org,2015:test/
%TAG !hola! http://yamlette.org,2015:hola/
---
- Mark McGwire
"#;

    let mark = "Mark McGwire";
    let sammy = "Sammy Sosa";
    let ken = "Ken Griffey";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    yamlette_compose! ( orchestra ; orc ; [ % YAML , (TAG ; "!test!", "http://yamlette.org,2015:test/") =>
        [ % (TAG ; "!aloha!", "http://yamlette.org,2015:aloha/") => [ mark, sammy, ken ] ],
        [ [ sammy, ken ] ],
        [ % NO_YAML, NO_BORDER_BOT, (TAG ; "!hola!", "http://yamlette.org,2015:hola/") => [ mark ] ]
    ] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



#[test]
fn example_02_01_block_tagged () {
    let should_be = 
r#"!!seq
- !!str Mark McGwire
- !!str Sammy Sosa
- !!str Ken Griffey
"#;

    let mark = "Mark McGwire";
    let sammy = "Sammy Sosa";
    let ken = "Ken Griffey";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::style::{ IssueTag };

    yamlette_compose! ( orchestra ; orc ; [[ # IssueTag (true) => [ mark, sammy, ken ] ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}


#[test]
fn example_02_01_block_tagged_no_tagged () {
    let should_be = 
r#"!!seq
- Mark McGwire
- !!str "Sammy Sosa"
- Ken Griffey
"#;

    let mark = "Mark McGwire";
    let sammy = "Sammy Sosa";
    let ken = "Ken Griffey";

    let orc = Orchestra::new (get_charset_utf8 (), get_schema ()).ok ().unwrap ();

    use yamlette::model::style::{ IssueTag };
    use yamlette::model::yaml::str::{ ForceQuotes, PreferDoubleQuotes };


    yamlette_compose! ( orchestra ; orc ; [[ ( # IssueTag (true) => [ # IssueTag (false) =>
        mark,
        ( # IssueTag (true), ForceQuotes (true), PreferDoubleQuotes (true) => sammy ),
        ken
    ]) ]] );

    let maybe_music = orc.listen ();

    let result = unsafe { String::from_utf8_unchecked (maybe_music.ok ().unwrap ()) };

    assert_eq! (should_be, result);
}



ex! (
    example_02_03;

r#"american:
  - Boston Red Sox
  - Detroit Tigers
  - New York Yankees
national:
  - New York Mets
  - Chicago Cubs
  - Atlanta Braves
"#;

    [[ {
        "american": [ "Boston Red Sox", "Detroit Tigers", "New York Yankees" ],
        "national": [ "New York Mets", "Chicago Cubs", "Atlanta Braves" ]
    } ]]
);



ex! (
    example_02_04_simplified;

r#"- name: Mark McGwire
  hr: 65
  avg: 0.278
- name: Sammy Sosa
  hr: 63
  avg: 0.288
"#;

    [[
        [
            {
                "name": "Mark McGwire",
                "hr": 65,
                "avg": 0.278
            },
            {
                "name": "Sammy Sosa",
                "hr": 63,
                "avg": 0.288
            }
        ]
    ]]
);



ex! (
    example_02_05_simplified;

r#"- [ name, hr, avg ]
- [ Mark McGwire, 65, 0.278 ]
- [ Sammy Sosa, 63, 0.288 ]
"#;

    [[[ # Flow (true) =>
        ["name", "hr", "avg"],
        ["Mark McGwire", 65, 0.278],
        ["Sammy Sosa", 63, 0.288]
    ]]] ;

    yamlette::model::style::Flow
);



ex! (
    example_02_06_simplified ;

r#"Mark McGwire: { hr: 65, avg: 0.278 }
Sammy Sosa: {
  hr: 63,
  avg: 0.288
}
"# ;

    [[
        { # Flow (true) =>
            "Mark McGwire": { "hr": 65, "avg": 0.278 },
            "Sammy Sosa": ( # Multiline (true) =>  { "hr": 63, "avg": 0.288 } )
        }
    ]] ;

    yamlette::model::style::Flow,
    yamlette::model::style::Multiline
);



ex! (
    example_02_07_simplified ;

r#"---
- Mark McGwire
- Sammy Sosa
- Ken Griffey
---
- Chicago Cubs
- St Louis Cardinals
"# ;

    [ % BORDER_TOP, NO_BORDER_BOT =>
        [ [ "Mark McGwire", "Sammy Sosa", "Ken Griffey" ] ],
        [ [ "Chicago Cubs", "St Louis Cardinals" ] ]
    ]
);



ex! (
    example_02_08 ;

r#"---
time: 20:03:20
player: Sammy Sosa
action: strike (miss)
...
---
time: 20:03:47
player: Sammy Sosa
action: grand slam
...
"# ;

    [ % BORDER_TOP =>
        [
            {
                "time": "20:03:20",
                "player": "Sammy Sosa",
                "action": "strike (miss)"
            }
        ],
        [
            {
                "time": "20:03:47",
                "player": "Sammy Sosa",
                "action": "grand slam"
            }
        ]
    ]
);



ex! (
    example_02_09_simplified ;

r#"---
hr:
  - Mark McGwire
  - Sammy Sosa
rbi:
  - Sammy Sosa
  - Ken Griffey
"# ;

    [ % BORDER_TOP, NO_BORDER_BOT => [
        {
            "hr": [ "Mark McGwire", "Sammy Sosa" ],
            "rbi": [ "Sammy Sosa", "Ken Griffey" ]
        }
    ]]
);



ex! (
    example_02_10_simplified ;

r#"---
hr:
  - Mark McGwire
  - &SS Sammy Sosa
rbi:
  - *SS
  - Ken Griffey
"# ;

    [[ % BORDER_TOP, NO_BORDER_BOT =>
        {
            "hr": [ "Mark McGwire", (&SS "Sammy Sosa") ],
            "rbi": [ (*SS), "Ken Griffey" ]
        }
    ]]
);



ex! (
    example_02_11_simplified ;

r#"?
  - Detroit Tigers
  - Chicago cubs
: - 2001-07-23

? [ New York Yankees,
Atlanta Braves ]
: [ 2001-07-02, 2001-08-12,
2001-08-14 ]
"# ;

    [[
        {
            [ "Detroit Tigers", "Chicago cubs" ]: [ "2001-07-23" ],
            ( # Flow (true), RespectThreshold (true), Threshold (20) => [ "New York Yankees", "Atlanta Braves" ] ): ( # Flow (true), RespectThreshold (true), Threshold (30) => [ "2001-07-02", "2001-08-12", "2001-08-14" ] )
        }
    ]] ;

    yamlette::model::style::Flow,
    yamlette::model::style::RespectThreshold,
    yamlette::model::style::Threshold
);



ex! (
    example_02_12_simplified ;

r#"---
- item: Super Hoop
  quantity: 1
- item: Basketball
  quantity: 4
- item: Big Shoes
  quantity: 1
"# ;

    [[ % BORDER_TOP, NO_BORDER_BOT =>
        [
            { "item": "Super Hoop", "quantity": 1 },
            { "item": "Basketball", "quantity": 4 },
            { "item": "Big Shoes", "quantity": 1 }
        ]
    ]]
);



ex! (
    example_02_21_simplified ;

r#"null: ~
booleans: [ true, false ]
string: '012345'
"# ;

    [[{
        "null": (),
        "booleans": (# Flow (true) => [ true, false ]),
        "string": (# ForceQuotes (true) => "012345")
    }]] ;

    yamlette::model::style::Flow ,
    yamlette::model::yaml::str::ForceQuotes
);



#[test]
fn example_02_25_simplified () {
    let should_be =
r#"---
!!set
? !!str Mark McGwire
? !!str Sammy Sosa
? !!str Ken Griffy
"#;

    let mut set = Vec::new ();

    set.push ("Mark McGwire");
    set.push ("Sammy Sosa");
    set.push ("Ken Griffy");

    let set = Set (set);

    use yamlette::model::style::IssueTag;

    check! (should_be ; [[ # IssueTag (true) => % BORDER_TOP, NO_BORDER_BOT => set ]]);
}


#[test]
fn example_02_26_simplified () {
    let should_be =
r#"---
!!omap
- !!str Ken Griffy: !!int 58
- !!str Mark McGwire: !!int 65
- !!str Sammy Sosa: !!int 63
"#;

    let mut map = BTreeMap::new ();

    map.insert ("Mark McGwire", 65);
    map.insert ("Sammy Sosa", 63);
    map.insert ("Ken Griffy", 58);

    let omap = Omap (map);

    use yamlette::model::style::IssueTag;

    check! (should_be ; [[ # IssueTag (true) => % BORDER_TOP, NO_BORDER_BOT => omap ]]);
}


ex! (
    example_02_27_simplified ;

r#"---
invoice: 34843
date: 2001-01-23
bill-to:
    &id001
    given: Chris
    family: Dumars
    address:
        lines: "458 Walkman Dr.\nSuite #292"
        city: Royal Oak
        state: MI
        postal: 48046
ship-to: *id001
product:
    - sku: BL394D
      quantity: 4
      description: Basketball
      price: 450.00
    - sku: BL4438H
      quantity: 1
      description: Super Hoop
      price: 2392.00
tax: 251.42
total: 4443.52
comments: "Late afternoon is best.\nBackup contact is Nancy\nBillsmer @ 338-4338."
"# ;

    [[ # Indent(4) => % BORDER_TOP, NO_BORDER_BOT => {
        "invoice": 34843,
        "date": "2001-01-23",
        "bill-to": (&id001 {
            "given": "Chris",
            "family": "Dumars",
            "address": {
                "lines": "458 Walkman Dr.\nSuite #292",
                "city": "Royal Oak",
                "state": "MI",
                "postal": 48046
            }
        }),
        "ship-to": (*id001),
        "product": ( # Indent (2) =>[
            {
                "sku": "BL394D",
                "quantity": 4,
                "description": "Basketball",
                "price": "450.00"
            },
            {
                "sku": "BL4438H",
                "quantity": 1,
                "description": "Super Hoop",
                "price": "2392.00"
            }
        ]),
        "tax": 251.42,
        "total": 4443.52,
        "comments": "Late afternoon is best.\nBackup contact is Nancy\nBillsmer @ 338-4338."
    }]] ;

    yamlette::model::style::Indent
);

}
