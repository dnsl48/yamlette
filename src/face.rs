pub extern crate skimmer;

use crate::model::schema::Schema;

pub struct Options<S>
where
    S: Schema + 'static,
{
    pub schema: Option<S>,
}

impl<S> Options<S>
where
    S: Schema + Clone + 'static,
{
    pub fn new() -> Options<S> {
        Options { schema: None }
    }
}

impl<S, O> From<(S, Options<O>)> for Options<S>
where
    S: Schema + Clone + 'static,
    O: Schema + Clone + 'static,
{
    fn from(val: (S, Options<O>)) -> Options<S> {
        Options {
            schema: Some(val.0),
        }
    }
}

#[macro_export]
macro_rules! yamlette {
    ( read ; $source:expr ; $rules:tt ) => { $crate::yamlette! ( read ; $source ; $rules ; {} ) };

    ( read ; $source:expr ; $rules:tt ; $options:tt ) => {
        let mut rs = $crate::yamlette! ( init ; reader ; $options );

        $crate::yamlette! ( read ; warm ; &mut rs ; $source ; $rules ; $options );
    };

    ( read ; warm ; $rs:expr ; $source:expr ; $rules:tt ; $options:tt ) => {
        let mut _book = $crate::book::Book::new ();

        let _result/*: Result<(), Result<SageError, ReadError>>*/ = match *$rs {
            Ok ( (ref mut reader, ref mut savant) ) => {
                match reader.read (
                    $crate::face::skimmer::reader::IntoReader::into_reader ($source),
                    &mut |block| { match savant.think (block) {
                        Ok (maybe_idea) => { if let Some (idea) = maybe_idea { _book.stamp (idea); }; Ok ( () ) },
                        Err (_) => Err (::std::borrow::Cow::from ("Cannot think of a block"))
                    } }
                ) {
                    Ok (_) => Ok ( () ),
                    Err (err) => Err (Err (err))
                }
            }
            Err (ref mut err) => Err (Ok (::std::mem::replace (err, $crate::sage::SageError::Error (::std::borrow::Cow::from (String::with_capacity (0))))))
        };

        $crate::yamlette_reckon! ( book ; _book ; $rules );

        $crate::yamlette! ( options moveout ; _book ; _result ; $options );
    };

    ( sage ; $source:expr ; $rules:tt ) => { $crate::yamlette! ( sage ; $source ; $rules ; {} ) };

    ( sage ; $source:expr ; $rules:tt ; $options:tt ) => {
        let mut rs = $crate::yamlette! ( init ; sage ; $options );

        $crate::yamlette! ( sage ; warm ; &mut rs ; $source ; $rules ; $options );
    };

    ( sage ; warm ; $rs:expr ; $source:expr ; $rules:tt ; $options:tt ) => {
        let mut _book = $crate::book::Book::new ();

        let _result/*: Result<(), Result<SageError, ReadError>>*/ = match *$rs {
            Ok ( (ref mut reader, ref mut sender, ref sage) ) => {
                match reader.read (
                    $crate::face::skimmer::reader::IntoReader::into_reader ($source),
                    &mut |block| { if let Err (_) = sender.send (block) { Err (::std::borrow::Cow::from ("Cannot yield a block")) } else { Ok ( () ) } }
                ) {
                    Ok (_) => {
                        _book.get_written (sage);
                        Ok ( () )
                    }
                    Err (err) => Err (Err (err))
                }
            }
            Err (ref mut err) => Err (Ok (::std::mem::replace (err, $crate::sage::SageError::Error (::std::borrow::Cow::from (String::with_capacity (0))))))
        };

        $crate::yamlette_reckon! ( book ; _book ; $rules );

        $crate::yamlette! ( options moveout ; _book ; _result ; $options );
    };

    ( write ; $rules:tt ) => {{ $crate::yamlette! ( write ; $rules ; {} ) }};

    ( write ; $rules:tt ; $options:tt ) => {{
        match $crate::yamlette! ( init ; writer ; $options ) {
            Ok ( mut orch ) => yamlette! ( write ; warm ; &mut orch ; $rules ),
            Err ( err ) => Err ( err )
        }
    }};

    ( write ; warm ; $orchestra:expr ; $rules:tt ) => {{
        $crate::yamlette_compose! ( orchestra ; $orchestra ; $rules );

        match $orchestra.listen () {
            Ok (music) => Ok (unsafe { String::from_utf8_unchecked (music) }),
            Err (error) => Err (error)
        }
    }};


    ( init ; reader ) => {{ $crate::yamlette! ( init ; reader ; {} ) }};

    ( init ; reader ; $options:tt ) => {{
        $crate::yamlette! ( options ; $options ; options );

        let schema = options.schema.take ().unwrap ();

        let reader = $crate::reader::Reader::new ();
        let savant = $crate::savant::Savant::new (schema);

        Ok ( (reader, savant) )
    }};


    ( init ; sage ) => {{ $crate::yamlette! ( init ; sage ; {} ) }};

    ( init ; sage ; $options:tt ) => {{
        $crate::yamlette! ( options ; $options ; options );

        let schema = options.schema.take ().unwrap ();

        let (sender, receiver) = ::std::sync::mpsc::channel ();

        let reader = $crate::reader::Reader::new ();

        match $crate::sage::Sage::new (receiver, schema) {
            Ok (sage) => Ok ( (reader, sender, sage) ),
            Err ( err ) => Err ( $crate::sage::SageError::IoError (err) )
        }
    }};

    ( init ; writer ) => {{ $crate::yamlette! ( init ; writer ; {} ) }};

    ( init ; writer ; $options:tt ) => {{
        $crate::yamlette! ( options ; $options ; options );

        match $crate::orchestra::Orchestra::new (options.schema.take ().unwrap ()) {
            Ok ( orch ) => Ok ( orch ),
            Err ( err ) => Err ( $crate::orchestra::OrchError::IoError ( err ) )
        }
    }};

    ( options ; { $( $key:ident : $val:expr ),* } ; $var:ident ) => {
        let mut $var: $crate::face::Options<$crate::model::schema::core::Core> = $crate::face::Options::new ();

        $(
            $var = $crate::yamlette! ( option ; $var ; $key ; $val );
        )*

        $var = if $var.schema.is_none () {
            let schema = $crate::model::schema::core::Core::new ();
            $crate::face::Options::from ((schema, $var))
        } else {
            $var
        };
    };

    ( option ; $options:expr ; schema ; $schema:expr ) => {{ $crate::face::Options::from (($schema, $options)) }};

    ( option ; $options:expr ; $unu:tt ; $dua:tt ) => {{ $options }};

    ( option ; $options:expr ; $unu:expr ; $dua:expr ) => {{ $options }};

    ( option ; $options:expr ; $unu:ident ; $dua:ident ) => {{ $options }};

    ( options moveout ; $book:expr ; $result:expr ; { $( $key:ident : $val:ident ),* } ) => {
        $(
            $crate::yamlette! ( option moveout ; $book ; $result ; $key ; $val );
        )*
    };

    ( option moveout ; $book:expr ; $result:expr ; book ; $var:ident ) => { let $var = &$book; };

    ( option moveout ; $book:expr ; $result:expr ; result ; $var:ident ) => { let $var = $result; };

    ( option moveout ; $book:expr ; $result:expr ; $unu:tt ; $dua:tt ) => {{ }};

    ( option moveout ; $book:expr ; $result:expr ; $unu:expr ; $dua:expr ) => {{ }};

    ( option moveout ; $book:expr ; $result:expr ; $unu:ident ; $dua:ident ) => {{ }};
}
