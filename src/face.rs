pub extern crate skimmer;

use txt::CharSet;
use model::schema::Schema;


pub struct Options<S> {
    pub cset: Option<CharSet>,
    pub schema: Option<S>
}


impl<S: Schema> Options<S> {
    pub fn new () -> Options<S> { Options { cset: None, schema: None } }

    pub fn set_charset (&mut self, cset: CharSet) { self.cset = Some (cset); }
}


impl<S: Schema, O: Schema> From<(S, Options<O>)> for Options<S> {
    fn from (mut val: (S, Options<O>)) -> Options<S> { Options {
        cset: val.1.cset.take (),
        schema: Some (val.0)
    }}
}



#[macro_export]
macro_rules! yamlette {
    ( read ; $source:expr ; $rules:tt ) => { yamlette! ( read ; $source ; $rules ; {} ) };

    ( read ; $source:expr ; $rules:tt ; $options:tt ) => {
        let mut rs = yamlette! ( init ; reader ; $options );

        yamlette! ( read ; warm ; &mut rs ; $source ; $rules ; $options );
    };

    ( read ; warm ; $rs:expr ; $source:expr ; $rules:tt ; $options:tt ) => {
        use std::mem;
        use $crate::txt::Twine;

        let mut _book = $crate::book::Book::new ();

        let _result/*: Result<(), Result<SageError, ReadError>>*/ = match *$rs {
            Ok ( (ref mut reader, ref mut sender, ref sage) ) => {
                match reader.read (
                    $crate::face::skimmer::reader::IntoReader::into_reader ($source),
                    &mut |block| { if let Err (_) = sender.send (block) { Err (Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
                ) {
                    Ok (_) => {
                        _book.get_written (sage);
                        Ok ( () )
                    }
                    Err (err) => Err (Err (err))
                }
            }
            Err (ref mut err) => Err (Ok (mem::replace (err, $crate::sage::SageError::Error (Twine::empty ()))))
        };

        yamlette_reckon! ( book ; _book ; $rules );

        yamlette! ( options moveout ; _book ; _result ; $options );
    };

    ( write ; $rules:tt ) => {{ yamlette! ( write ; $rules ; {} ) }};

    ( write ; $rules:tt ; $options:tt ) => {{
        match yamlette! ( init ; writer ; $options ) {
            Ok ( mut orch ) => yamlette! ( write ; warm ; &mut orch ; $rules ),
            Err ( err ) => Err ( err )
        }
    }};

    ( write ; warm ; $orchestra:expr ; $rules:tt ) => {{ 
        yamlette_compose! ( orchestra ; $orchestra ; $rules );

        match $orchestra.listen () {
            Ok (music) => Ok (unsafe { String::from_utf8_unchecked (music) }),
            Err (error) => Err (error)
        }
    }};


    ( init ; reader ) => {{ yamlette! ( init ; reader ; {} ) }};

    ( init ; reader ; $options:tt ) => {{
        use $crate::reader::Reader;
        use $crate::tokenizer::Tokenizer;
        use $crate::sage::{ Sage, SageError };
        use std::sync::mpsc::channel;

        yamlette! ( options ; $options ; options );

        let cset = options.cset.take ().unwrap ();
        let schema = options.schema.take ().unwrap ();

        let (sender, receiver) = channel ();

        let reader = Reader::new (Tokenizer::new (cset.clone ()));

        match Sage::new (cset, receiver, schema) {
            Ok (sage) => Ok ( (reader, sender, sage) ),
            Err ( err ) => Err ( SageError::IoError (err) )
        }
    }};

    ( init ; writer ) => {{ yamlette! ( init ; writer ; {} ) }};

    ( init ; writer ; $options:tt ) => {{
        use $crate::orchestra::{ Orchestra, OrchError };

        yamlette! ( options ; $options ; options );

        match Orchestra::new (options.cset.take ().unwrap (), options.schema.take ().unwrap ()) {
            Ok ( orch ) => Ok ( orch ),
            Err ( err ) => Err ( OrchError::IoError ( err ) )
        }
    }};

    ( options ; { $( $key:ident : $val:expr ),* } ; $var:ident ) => {
        use $crate::face::Options;

        let mut $var: Options<$crate::model::schema::core::Core> = Options::new ();

        $(
            $var = yamlette! ( option ; $var ; $key ; $val );
        )*

        if $var.cset.is_none () { $var.set_charset ($crate::txt::get_charset_utf8 ()); }

        $var = if $var.schema.is_none () {
            Options::from (($crate::model::schema::core::Core::new (), $var))
        } else {
            $var
        };
    };

    ( option ; $options:expr ; schema ; $schema:expr ) => {{ Options::from (($schema, $options)) }};

    ( option ; $options:expr ; encoding ; $encoding:tt ) => {{
        let cset = match stringify! ($encoding) {
            "UTF8" |
            "utf8" |
            "UTF-8" |
            "utf-8" => $crate::txt::get_charset_utf8 (),
            enc @ _ => panic! ("unknown encoding: {}", enc)
        };

        $options.set_charset (cset); $options
    }};

    ( option ; $options:expr ; charset ; $cset:expr ) => {{ $options.set_charset ($cset); $options }};

    ( option ; $options:expr ; $unu:tt ; $dua:tt ) => {{ $options }};

    ( option ; $options:expr ; $unu:expr ; $dua:expr ) => {{ $options }};

    ( option ; $options:expr ; $unu:ident ; $dua:ident ) => {{ $options }};

    ( options moveout ; $book:expr ; $result:expr ; { $( $key:ident : $val:ident ),* } ) => {
        $(
            yamlette! ( option moveout ; $book ; $result ; $key ; $val );
        )*
    };

    ( option moveout ; $book:expr ; $result:expr ; book ; $var:ident ) => { let $var = &$book; };

    ( option moveout ; $book:expr ; $result:expr ; result ; $var:ident ) => { let $var = $result; };

    ( option moveout ; $book:expr ; $result:expr ; $unu:tt ; $dua:tt ) => {{ }};

    ( option moveout ; $book:expr ; $result:expr ; $unu:expr ; $dua:expr ) => {{ }};

    ( option moveout ; $book:expr ; $result:expr ; $unu:ident ; $dua:ident ) => {{ }};
}
