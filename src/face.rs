pub extern crate skimmer;

use self::skimmer::symbol::{ Combo, CopySymbol };

use txt::CharSet;
use model::schema::Schema;


pub struct Options<Char, DoubleChar, S>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static,
    S: Schema<Char, DoubleChar> + 'static
{
    pub cset: Option<CharSet<Char, DoubleChar>>,
    pub schema: Option<S>
}


impl<Char, DoubleChar, S> Options<Char, DoubleChar, S>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static,
    S: Schema<Char, DoubleChar> + 'static
{
    pub fn new () -> Options<Char, DoubleChar, S> { Options { cset: None, schema: None } }

    pub fn set_charset (&mut self, cset: CharSet<Char, DoubleChar>) { self.cset = Some (cset); }
}


impl<Char, DoubleChar, S, O> From<(S, Options<Char, DoubleChar, O>)> for Options<Char, DoubleChar, S>
  where
    Char: CopySymbol + 'static,
    DoubleChar: CopySymbol + Combo + 'static,
    S: Schema<Char, DoubleChar> + 'static,
    O: Schema<Char, DoubleChar> + 'static
{
    fn from (mut val: (S, Options<Char, DoubleChar, O>)) -> Options<Char, DoubleChar, S> { Options {
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
        let mut _book = $crate::book::Book::new ();

        let _result/*: Result<(), Result<SageError, ReadError>>*/ = match *$rs {
            Ok ( (ref mut reader, ref mut savant) ) => {
                match reader.read (
                    $crate::face::skimmer::reader::IntoReader::into_reader ($source),
                    &mut |block| { match savant.think (block) {
                        Ok (maybe_idea) => { if let Some (idea) = maybe_idea { _book.stamp (idea); }; Ok ( () ) },
                        Err (_) => Err ($crate::txt::Twine::from ("Cannot think of a block"))
                    } }
                ) {
                    Ok (_) => Ok ( () ),
                    Err (err) => Err (Err (err))
                }
            }
            Err (ref mut err) => Err (Ok (::std::mem::replace (err, $crate::sage::SageError::Error ($crate::txt::Twine::empty ()))))
        };

        yamlette_reckon! ( book ; _book ; $rules );

        yamlette! ( options moveout ; _book ; _result ; $options );
    };

    ( sage ; $source:expr ; $rules:tt ) => { yamlette! ( sage ; $source ; $rules ; {} ) };

    ( sage ; $source:expr ; $rules:tt ; $options:tt ) => {
        let mut rs = yamlette! ( init ; sage ; $options );

        yamlette! ( sage ; warm ; &mut rs ; $source ; $rules ; $options );
    };

    ( sage ; warm ; $rs:expr ; $source:expr ; $rules:tt ; $options:tt ) => {
        let mut _book = $crate::book::Book::new ();

        let _result/*: Result<(), Result<SageError, ReadError>>*/ = match *$rs {
            Ok ( (ref mut reader, ref mut sender, ref sage) ) => {
                match reader.read (
                    $crate::face::skimmer::reader::IntoReader::into_reader ($source),
                    &mut |block| { if let Err (_) = sender.send (block) { Err ($crate::txt::Twine::from ("Cannot yield a block")) } else { Ok ( () ) } }
                ) {
                    Ok (_) => {
                        _book.get_written (sage);
                        Ok ( () )
                    }
                    Err (err) => Err (Err (err))
                }
            }
            Err (ref mut err) => Err (Ok (::std::mem::replace (err, $crate::sage::SageError::Error ($crate::txt::Twine::empty ()))))
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
        yamlette! ( options ; $options ; options );

        let cset = options.cset.take ().unwrap ();
        let schema = options.schema.take ().unwrap ();

        let reader = $crate::reader::Reader::new ($crate::tokenizer::Tokenizer::new (cset));
        let savant = $crate::savant::Savant::new (schema);

        Ok ( (reader, savant) )
    }};


    ( init ; sage ) => {{ yamlette! ( init ; sage ; {} ) }};

    ( init ; sage ; $options:tt ) => {{
        yamlette! ( options ; $options ; options );

        let cset = options.cset.take ().unwrap ();
        let schema = options.schema.take ().unwrap ();

        let (sender, receiver) = ::std::sync::mpsc::channel ();

        let reader = $crate::reader::Reader::new ($crate::tokenizer::Tokenizer::new (cset.clone ()));

        match $crate::sage::Sage::new (cset, receiver, schema) {
            Ok (sage) => Ok ( (reader, sender, sage) ),
            Err ( err ) => Err ( $crate::sage::SageError::IoError (err) )
        }
    }};

    ( init ; writer ) => {{ yamlette! ( init ; writer ; {} ) }};

    ( init ; writer ; $options:tt ) => {{
        yamlette! ( options ; $options ; options );

        match $crate::orchestra::Orchestra::new (options.cset.take ().unwrap (), options.schema.take ().unwrap ()) {
            Ok ( orch ) => Ok ( orch ),
            Err ( err ) => Err ( $crate::orchestra::OrchError::IoError ( err ) )
        }
    }};

    ( options ; { $( $key:ident : $val:expr ),* } ; $var:ident ) => {
        let mut $var: $crate::face::Options<$crate::txt::charset::utf8::Char1, $crate::txt::charset::utf8::Char2, $crate::model::schema::core::Core<$crate::txt::charset::utf8::Char1, $crate::txt::charset::utf8::Char2>> = $crate::face::Options::new ();

        $(
            $var = yamlette! ( option ; $var ; $key ; $val );
        )*

        if $var.cset.is_none () { $var.set_charset ($crate::txt::get_charset_utf8 ()); }

        $var = if $var.schema.is_none () {
            let schema = $crate::model::schema::core::Core::new ($var.cset.as_ref ().unwrap ());
            $crate::face::Options::from ((schema, $var))
        } else {
            $var
        };
    };

    ( option ; $options:expr ; schema ; $schema:expr ) => {{ $crate::face::Options::from (($schema, $options)) }};

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
