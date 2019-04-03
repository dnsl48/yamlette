pub extern crate skimmer;

use model::schema::Schema;


pub struct Options<S>
  where
    S: Schema + 'static
{
    pub schema: Option<S>
}


impl<S> Options<S>
  where
    S: Schema + Clone + 'static
{
    pub fn new () -> Options<S> { Options { schema: None } }
}


impl<S, O> From<(S, Options<O>)> for Options<S>
  where
    S: Schema + Clone + 'static,
    O: Schema + Clone + 'static
{
    fn from (val: (S, Options<O>)) -> Options<S> { Options {
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
                        Err (_) => Err (::std::borrow::Cow::from ("Cannot think of a block"))
                    } }
                ) {
                    Ok (_) => Ok ( () ),
                    Err (err) => Err (Err (err))
                }
            }
            Err (ref mut err) => Err (Ok (::std::mem::replace (err, $crate::sage::SageError::Error (::std::borrow::Cow::from (String::with_capacity (0))))))
        };

        yamlette! ( reckon book ; _book ; $rules );

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

        yamlette! ( reckon book ; _book ; $rules );

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
        yamlette! ( compose orchestra ; $orchestra ; $rules );

        match $orchestra.listen () {
            Ok (music) => Ok (unsafe { String::from_utf8_unchecked (music) }),
            Err (error) => Err (error)
        }
    }};


    ( init ; reader ) => {{ yamlette! ( init ; reader ; {} ) }};

    ( init ; reader ; $options:tt ) => {{
        yamlette! ( options ; $options ; options );

        let schema = options.schema.take ().unwrap ();

        let reader = $crate::reader::Reader::new ();
        let savant = $crate::savant::Savant::new (schema);

        Ok ( (reader, savant) )
    }};


    ( init ; sage ) => {{ yamlette! ( init ; sage ; {} ) }};

    ( init ; sage ; $options:tt ) => {{
        yamlette! ( options ; $options ; options );

        let schema = options.schema.take ().unwrap ();

        let (sender, receiver) = ::std::sync::mpsc::channel ();

        let reader = $crate::reader::Reader::new ();

        match $crate::sage::Sage::new (receiver, schema) {
            Ok (sage) => Ok ( (reader, sender, sage) ),
            Err ( err ) => Err ( $crate::sage::SageError::IoError (err) )
        }
    }};

    ( init ; writer ) => {{ yamlette! ( init ; writer ; {} ) }};

    ( init ; writer ; $options:tt ) => {{
        yamlette! ( options ; $options ; options );

        match $crate::orchestra::Orchestra::new (options.schema.take ().unwrap ()) {
            Ok ( orch ) => Ok ( orch ),
            Err ( err ) => Err ( $crate::orchestra::OrchError::IoError ( err ) )
        }
    }};

    ( options ; { $( $key:ident : $val:expr ),* } ; $var:ident ) => {
        let mut $var: $crate::face::Options<$crate::model::schema::core::Core> = $crate::face::Options::new ();

        $(
            $var = yamlette! ( option ; $var ; $key ; $val );
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
            yamlette! ( option moveout ; $book ; $result ; $key ; $val );
        )*
    };

    ( option moveout ; $book:expr ; $result:expr ; book ; $var:ident ) => { let $var = &$book; };

    ( option moveout ; $book:expr ; $result:expr ; result ; $var:ident ) => { let $var = $result; };

    ( option moveout ; $book:expr ; $result:expr ; $unu:tt ; $dua:tt ) => {{ }};

    ( option moveout ; $book:expr ; $result:expr ; $unu:expr ; $dua:expr ) => {{ }};

    ( option moveout ; $book:expr ; $result:expr ; $unu:ident ; $dua:ident ) => {{ }};

    ( reckon book ; $book:expr ; [ $( $rules:tt ),* ] ) => {
        let mut _counter: usize = 0;
        $(
            let volume = $book.volumes.get (_counter);
            yamlette! ( reckon volume ; volume ; $rules );
            _counter += 1;
        )*
    };

    ( reckon volume ; $volume:expr ; [ $( $rules:tt ),* ] ) => {
        let _pointer = if let Some (volume) = $volume { $crate::book::extractor::pointer::Pointer::new (volume) } else { None };
        $(
            yamlette! ( reckon ptr ; _pointer ; $rules );
            let _pointer = if let Some (p) = _pointer { p.next_sibling () } else { None };
        )*
    };

    ( reckon ptr ; $pointer:expr ; [ $( $v:tt ),* ] ) => {
        let _pointer = if let Some (p) = $pointer { p.into_seq () } else { None };
        $(
            yamlette! ( reckon ptr ; _pointer ; $v );
            let _pointer = if let Some (p) = _pointer { p.next_sibling () } else { None };
        )*
    };

    ( reckon ptr ; $pointer:expr ; { $( $k:tt > $v:tt ),* } ) => {
        let _pointer = if let Some (p) = $pointer { p.into_map () } else { None };
        $(
            yamlette! ( reckon ptr ; _pointer ; $k );
            let _pointer = if let Some (p) = _pointer { p.next_sibling () } else { None };
            yamlette! ( reckon ptr ; _pointer ; $v );
            let _pointer = if let Some (p) = _pointer { p.next_sibling () } else { None };
        )*
    };

    ( reckon ptr ; $pointer:expr ; { $( $k:expr => $v:tt ),* } ) => {
        $(
            let mut _pointer = if let Some (p) = $pointer { p.into_map () } else { None };
            {
                let mut found = false;

                loop {
                    if _pointer.is_none () { break; }
                    let ptr = _pointer.unwrap ();

                    if ptr == $k {
                        found = true;
                        _pointer = ptr.next_sibling ();
                        break;
                    }

                    _pointer = ptr.next_sibling ();
                    _pointer = if let Some (p) = _pointer { p.next_sibling () } else { None };
                }

                if !found { _pointer = None; }
            }
            yamlette! ( reckon ptr ; _pointer ; $v );
        )*
    };

    ( reckon ptr ; $pointer:expr ; { $( ( $v:ident:$t:ty ) ),* } ) => {
        $( let mut $v: Option<$t> = None; )*

        {
            let mut _pointer = if let Some (p) = $pointer { p.into_map () } else { None };

            loop {
                if _pointer.is_none () { break; }

                let ptr = _pointer.unwrap ();

                $(
                    if ptr == stringify! ($v) {
                        if let Some (p) = ptr.next_sibling () {
                            use $crate::book::extractor::traits::FromPointer;
                            $v = <$t as FromPointer>::from_pointer (p);
                        }

                        _pointer = ptr.next_sibling ();
                        _pointer = if let Some (p) = _pointer { p.next_sibling () } else { None };

                        continue;
                    }
                )*

                _pointer = ptr.next_sibling ();
                _pointer = if let Some (p) = _pointer { p.next_sibling () } else { None };
            }
        }
    };

    ( reckon ptr ; $pointer:expr ; ( $($v:ident:$t:ty),* ) ) => {
        $(
            let $v: Option<$t> = if let Some (p) = $pointer {
                use $crate::book::extractor::traits::FromPointer;
                <$t as FromPointer>::from_pointer (p)
            } else { None };
        )*
    };

    ( reckon ptr ; $pointer:expr ; (list $($v:ident:$t:ty),* ) ) => {
        $(
            let $v: Option<$t> = if let Some (p) = $pointer {
                use $crate::book::extractor::traits::List;
                let _word = p.unalias ().to_word ();
                let mut some: Option<$t> = match *_word {
                    $crate::book::word::Word::Seq (_) => Some (List::list_new ()),
                    _ => None
                };

                if let Some (ref mut val) = some {
                    let mut _pointer = if let Some (p) = $pointer { p.into_seq () } else { None };
                    if let Some (p) = _pointer { val.list_reserve (p.count_siblings ()) };
                    loop {
                        _pointer = if let Some (p) = _pointer {
                            val.list_update (p);
                            p.next_sibling ()
                        } else { break };
                    }
                };
                some
            } else { None };
        )*
    };

    ( reckon ptr ; $pointer:expr ; (dict $($v:ident:$t:ty),* ) ) => {
        $(
            let $v: Option<$t> = if let Some (p) = $pointer {
                use $crate::book::extractor::traits::Dict;
                let _word = p.unalias ().to_word ();
                let mut some: Option<$t> = match *_word {
                    $crate::book::word::Word::Map (_) => Some (Dict::dict_new ()),
                    _ => None
                };

                if let Some (ref mut val) = some {
                    let mut _pointer = if let Some (p) = $pointer { p.into_map () } else { None };
                    if let Some (p) = _pointer { val.dict_reserve (p.count_siblings ()/2usize) };
                    loop {
                        _pointer = if let Some (p) = _pointer {
                            let key = p;
                            if let Some (p) = p.next_sibling () {
                                val.dict_update (key, p);
                                p.next_sibling ()
                            } else { break }
                        } else { break };
                    }
                };
                some
            } else { None };
        )*
    };

    ( reckon ptr ; $pointer:expr ; (call $($f:expr),*) ) => {
        use $crate::book::extractor::pointer::Pointer;
        fn _clo<'a, F>(clo: &mut F, ptr: Pointer<'a>) where F: FnMut(Pointer<'a>) -> () { clo (ptr); }

        $(
            if $pointer.is_some () {
                _clo ($f, $pointer.unwrap ());
            }
        )*
    };

    ( reckon ptr ; $pointer:expr ; (foreach $($f:expr),*) ) => {
        use $crate::book::extractor::pointer::Pointer;
        fn _clo<'a, F>(clo: &mut F, ptr: Pointer<'a>) where F: FnMut(Pointer<'a>) -> () { clo (ptr); }

        let mut _ptr = $pointer;

        loop {
            _ptr = match _ptr {
                Some (ptr) => {
                    $( _clo ($f, ptr); )*;
                    ptr.next_sibling ()
                }
                None => break
            };
        }
    };

    ( compose ignore ; $ignored:tt ; $expr:tt ) => { $expr };


    ( compose orchestra ; $orchestra:expr ; $volumes:tt ) => {{ yamlette!(compose volumes ; &$orchestra ; $volumes ; [ ] ; [ ] ) }};


    ( compose size ; [# $( $style:expr ),* => $( $element:tt ),* ] ) => {{ yamlette!(compose size ; [ $( $element ),* ] ) }};

    ( compose size ; [ $( $element:tt ),* ] ) => {{
        let mut _size = 1;

        $(
            _size += yamlette!(compose size ; $element );
        )*

        _size
    }};

    ( compose size ; {# $( $style:expr ),* => $( $key:tt : $val:tt ),* } ) => {{ yamlette!(compose size ; { $( $key : $val ),* } ) }};

    ( compose size ; { $( $key:tt : $val:tt ),* } ) => {{
        let mut _size = 1;

        $(
            _size += yamlette!(compose size ; $key );
            _size += yamlette!(compose size ; $val );
        )*

        _size
    }};

    ( compose size ; ( # $( $style:expr ),* => $elem:tt ) ) => {{ yamlette!(compose size ; $elem ) }};

    ( compose size ; ( # $( $style:expr ),* => $elem:expr ) ) => {{ yamlette!(compose size ; $elem ) }};

    ( compose size ; ( & $alias:ident $elem:tt ) ) => {{ yamlette!(compose size ; $elem ) }};

    ( compose size ; ( & $alias:ident $elem:expr ) ) => {{ yamlette!(compose size ; $elem ) }};

    ( compose size ; ( * $link:ident ) ) => {{ 1 }};

    ( compose size ; ( $elem:tt ) ) => {{ yamlette!(compose size ; $elem ) }};

    ( compose size ; ( $elem:expr ) ) => {{ yamlette!(compose size ; $elem ) }};

    ( compose size ; $element:expr ) => {{
        use $crate::orchestra::chord::Chord;
        Chord::chord_size (&$element)
    }};


    ( compose directives ; $orchestra:expr ; $directives:tt ) => {{
        let _tags_count = yamlette!(compose directives ; tags count ; $directives );

        if _tags_count > 0 {
            use std::borrow::Cow;

            let mut _tags: Vec<(Cow<'static, str>, Cow<'static, str>)> = Vec::with_capacity (_tags_count);
            yamlette!(compose directives ; collect tags ; _tags ; $directives );
            $orchestra.directive_tags (_tags).ok ().unwrap ();
        }

        yamlette!(compose directives ; others ; $orchestra ; $directives );
    }};

    ( compose directives ; tags count ; [ $( $directive:tt ),* ] ) => {{
        let mut _size = 0;
        $( _size += yamlette!(compose directives ; tag count ; $directive ); )*
        _size
    }};

    ( compose directives ; tag count ; (TAG ; $shortcut:expr , $handle:expr ) ) => { 1 };
    ( compose directives ; tag count ; $directive:tt ) => { 0 };

    ( compose directives ; collect tags ; $vec:expr ; [ $( $directive:tt ),* ] ) => { $( yamlette!(compose directive ; collect tags ; $vec ; $directive ); )* };
    ( compose directive ; collect tags ; $vec:expr ; (TAG ; $shortcut:expr , $handle:expr ) ) => { $vec.push ( (Cow::from ($shortcut) , Cow::from ($handle)) ); };
    ( compose directive ; collect tags ; $vec:expr ; $directive:tt ) => {{ }};

    ( compose directives ; others ; $orchestra:expr ; [ $( $directive:tt ),* ] ) => {{ $( yamlette!(compose directive ; others ; $orchestra ; $directive ); )* }};
    ( compose directive ; others ; $orchestra:expr ; YAML ) => {{ $orchestra.directive_yaml (true).ok ().unwrap (); }};
    ( compose directive ; others ; $orchestra:expr ; NO_YAML ) => {{ $orchestra.directive_yaml (false).ok ().unwrap (); }};
    ( compose directive ; others ; $orchestra:expr ; BORDER_TOP ) => {{ $orchestra.volume_border_top (true).ok ().unwrap (); }};
    ( compose directive ; others ; $orchestra:expr ; NO_BORDER_TOP ) => {{ $orchestra.volume_border_top (false).ok ().unwrap (); }};
    ( compose directive ; others ; $orchestra:expr ; BORDER_BOT ) => {{ $orchestra.volume_border_bot (true).ok ().unwrap (); }};
    ( compose directive ; others ; $orchestra:expr ; NO_BORDER_BOT ) => {{ $orchestra.volume_border_bot (false).ok ().unwrap (); }};
    ( compose directive ; others ; $orchestra:expr ; (TAG ; $shortcut:expr , $handle:expr ) ) => {};


    ( compose styles ; [ $( $style:expr ),* ] ) => { [ $( &mut $style as &mut $crate::model::style::Style ),* ] };

    ( compose styles ; apply to common ; $common_styles:expr ; $styles:tt ) => {{
        let mut cstyles = $common_styles;

        let styles: &mut [ &mut $crate::model::style::Style ] = &mut yamlette!(compose styles ; $styles );

        for style in styles {
            style.common_styles_apply (&mut cstyles);
        }

        cstyles
    }};


    ( compose volumes ; $orchestra:expr ; [ # $( $style:expr ),* => % $( $directive:tt ),* => $( $volume:tt ),* ] ; [ ] ; [ ] ) => {{
        yamlette!(compose volumes ; $orchestra ; [ $( $volume ),* ] ; [ $( $style ),* ] ; [ $( $directive ),* ] )
    }};


    ( compose volumes ; $orchestra:expr ; [ % $( $directive:tt ),* => $( $volume:tt ),* ] ; [ ] ; [ ] ) => {{
        yamlette!(compose volumes ; $orchestra ; [ $( $volume ),* ] ; [ ] ; [ $( $directive ),* ] )
    }};


    ( compose volumes ; $orchestra:expr ; [ # $( $style:expr ),* => $( $volume:tt ),* ] ; [ ] ; [ ] ) => {{
        yamlette!(compose volumes ; $orchestra ; [ $( $volume ),* ] ; [ $( $style ),* ] ; [ ] )
    }};


    ( compose volumes ; $orchestra:expr ; [ $( $volume:tt ),* ] ; $styles:tt ; $directives:tt ) => {{
        let mut _size = 0;

        $( yamlette!(compose ignore ; $volume ; { _size += 1; } ); )*

        $orchestra.volumes (_size).ok ().unwrap ();

        let _common_styles = $orchestra.get_styles ();

        $(
            yamlette!(compose volume ; $orchestra ; _common_styles ; $volume ; $styles ; $directives );

            $orchestra.vol_end ().ok ().unwrap ();
        )*

        $orchestra.the_end ().ok ().unwrap ();
    }};


    ( compose volume ; $orchestra:expr ; $common_styles:expr ; [ # $( $style:expr ),+ => % $( $directive:tt ),+ => $( $rule:tt ),* ] ; [ ] ; [ ] ) => {{
        yamlette!(compose volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; [ $( $style ),* ] ; [ $( $directive ),* ] );
    }};

    ( compose volume ; $orchestra:expr ; $common_styles:expr ; [ # $( $style:expr ),+ => % $( $directive:tt ),+ => $( $rule:tt ),* ] ; [ $( $parent_style:expr ),* ] ; [ ] ) => {{
        yamlette!(compose volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; [ $( $parent_style ),* , $( $style ),* ] ; [ $( $directive ),* ] );
    }};


    ( compose volume ; $orchestra:expr ; $common_styles:expr ; [ # $( $style:expr ),+ => % $( $directive:tt ),+ => $( $rule:tt ),* ] ; [ ] ; [ $( $parent_directive:tt ),* ] ) => {{
        yamlette!(compose volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; [ $( $style ),* ] ; [ $( $parent_directive ),* , $( $directive ),* ] );
    }};


    ( compose volume ; $orchestra:expr ; $common_styles:expr ; [ # $( $style:expr ),+ => % $( $directive:tt ),+ => $( $rule:tt ),* ] ; [ $( $parent_style:expr ),* ] ; [ $( $parent_directive:tt ),* ] ) => {{
        yamlette!(compose volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; [ $( $parent_style ),* , $( $style ),* ] ; [ $( $parent_directive ),* , $( $directive ),* ] );
    }};


    ( compose volume ; $orchestra:expr ; $common_styles:expr ; [ % $( $directive:tt ),* => $( $rule:tt ),* ] ; $styles:tt ; [ ] ) => {{
        yamlette!(compose volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; $styles ; [ $( $directive ),* ] );
    }};


    ( compose volume ; $orchestra:expr ; $common_styles:expr ; [ % $( $directive:tt ),* => $( $rule:tt ),* ] ; $styles:tt ; [ $( $parent_directive:tt ),* ] ) => {{
        yamlette!(compose volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; $styles ; [ $( $parent_directive ),* , $( $directive ),* ] );
    }};


    ( compose volume ; $orchestra:expr ; $common_styles:expr ; [ # $( $style:expr ),* => $( $rule:tt ),* ] ; [ ] ; $directives:tt ) => {{
        yamlette!(compose volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; [ $( $style ),* ] ; $directives );
    }};


    ( compose volume ; $orchestra:expr ; $common_styles:expr ; [ # $( $style:expr ),* => $( $rule:tt ),* ] ; [ $( $parent_style:expr ),* ] ; $directives:tt ) => {{
        yamlette!(compose volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; [ $( $parent_style ),* , $( $style ),* ] ; $directives );
    }};


    ( compose volume ; $orchestra:expr ; $common_styles:expr ; [ $( $rules:tt ),* ] ; $styles:tt ; $directives:tt ) => {{
        let mut _size = 0;

        $orchestra.vol_next ().ok ().unwrap ();

        yamlette!(compose directives ; $orchestra ; $directives );

        $( yamlette!(compose ignore ; $rules ; { _size += yamlette!(compose size ; $rules ); } ); )*

        $orchestra.vol_reserve (_size).ok ().unwrap ();

        let _common_styles = yamlette!(compose styles ; apply to common ; $common_styles ; $styles );

        $(
            yamlette!(compose play ; $orchestra ; 0 ; $rules ; _common_styles ; $styles ; None );
        )*
    }};


    ( compose play ; $orchestra:expr ; $level:expr ; [ # $( $style:expr ),* => $( $element:tt ),* ] ; $common_styles:expr ; [] ; $alias:expr ) => {{
        yamlette!(compose play ; $orchestra ; $level ; [ $( $element ),* ] ; $common_styles ; [ $( $style ),* ] ; $alias )
    }};


    ( compose play ; $orchestra:expr ; $level:expr ; [ # $( $style:expr ),* => $( $element:tt ),* ] ; $common_styles:expr ; [ $( $parent_style:expr ),+ ] ; $alias:expr ) => {{
        yamlette!(compose play ; $orchestra ; $level ; [ $( $element ),* ] ; $common_styles ; [ $( $parent_style ),* , $( $style ),* ] ; $alias )
    }};


    ( compose play ; $orchestra:expr ; $level:expr ; [ $( $element:tt ),* ] ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        use $crate::orchestra::chord::{ Chord, EmptyList };

        let styles: &mut [ &mut $crate::model::style::Style ] = &mut yamlette!(compose styles ; $styles );

        Chord::play (EmptyList, $orchestra, $level, $alias, $common_styles, styles).ok ().unwrap ();

        let _common_styles = yamlette!(compose styles ; apply to common ; $common_styles ; $styles );

        $(
            yamlette!(compose play ; $orchestra ; $level + 1 ; $element ; _common_styles ; $styles ; None );
        )*
    }};


    ( compose play ; $orchestra:expr ; $level:expr ; { # $( $style:expr ),* => $( $key:tt : $val:tt ),* } ; $common_styles:expr ; [] ; $alias:expr ) => {{
        yamlette!(compose play ; $orchestra ; $level ; { $( $key : $val ),* } ; $common_styles ; [ $( $style ),* ] ; $alias )
    }};


    ( compose play ; $orchestra:expr ; $level:expr ; { # $( $style:expr ),* => $( $key:tt : $val:tt ),* } ; $common_styles:expr ; [ $( $parent_style:expr ),+ ] ; $alias:expr ) => {{
        yamlette!(compose play ; $orchestra ; $level ; { $( $key : $val ),* } ; $common_styles ; [ $( $parent_style ),* , $( $style ),* ] ; $alias )
    }};


    ( compose play ; $orchestra:expr ; $level:expr ; { $( $key:tt : $val:tt ),* } ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        use $crate::orchestra::chord::{ Chord, EmptyDict };

        let styles: &mut [ &mut $crate::model::style::Style ] = &mut yamlette!(compose styles ; $styles );

        Chord::play (EmptyDict, $orchestra, $level, $alias, $common_styles, styles).ok ().unwrap ();

        let _common_styles = yamlette!(compose styles ; apply to common ; $common_styles ; $styles );

        $(
            yamlette!(compose play ; $orchestra ; $level + 1 ; $key ; _common_styles ; $styles ; None );
            yamlette!(compose play ; $orchestra ; $level + 1 ; $val ; _common_styles ; $styles ; None );
        )*
    }};


    ( compose play ; $orchestra:expr ; $level:expr ; ( # $( $style:expr ),* => $element:tt ) ; $common_styles:expr ; [ ] ; $alias:expr ) => {{
        let _common_styles = yamlette!(compose styles ; apply to common ; $common_styles ; [ $( $style ),* ] );

        yamlette!(compose play ; $orchestra ; $level ; $element ; _common_styles ; [ $( $style ),* ] ; $alias );
    }};

    ( compose play ; $orchestra:expr ; $level:expr ; ( # $( $style:expr ),* => $element:tt ) ; $common_styles:expr ; [ $( $parent_style:expr ),* ] ; $alias:expr ) => {{
        let _common_styles = yamlette!(compose styles ; apply to common ; $common_styles ; [ $( $style ),* ] );

        yamlette!(compose play ; $orchestra ; $level ; $element ; _common_styles ; [ $( $parent_style ),* , $( $style ),* ] ; $alias );
    }};

    ( compose play ; $orchestra:expr ; $level:expr ; ( # $( $style:expr ),* => $element:expr ) ; $common_styles:expr ; [ ] ; $alias:expr ) => {{
        let _common_styles = yamlette!(compose styles ; apply to common ; $common_styles ; [ $( $style ),* ] );

        yamlette!(compose play ; $orchestra ; $level ; $element ; _common_styles ; [ $( $style ),* ] ; $alias );
    }};

    ( compose play ; $orchestra:expr ; $level:expr ; ( # $( $style:expr ),* => $element:expr ) ; $common_styles:expr ; [ $( $parent_style:expr ),* ] ; $alias:expr ) => {{
        let _common_styles = yamlette!(compose styles ; apply to common ; $common_styles ; [ $( $style ),* ] );

        yamlette!(compose unit ; $orchestra ; $level ; $element ; _common_styles ; [ $( $parent_style ),* , $( $style ),* ] ; $alias );
    }};

    ( compose play ; $orchestra:expr ; $level:expr ; ( & $new_alias:ident $element:tt ) ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        use std::borrow::Cow;
        yamlette!(compose play ; $orchestra ; $level ; $element ; $common_styles ; $styles ; Some (Cow::from (stringify! ($new_alias))) );
    }};

    ( compose play ; $orchestra:expr ; $level:expr ; ( & $new_alias:ident $element:expr ) ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        use std::borrow::Cow;
        yamlette!(compose play ; $orchestra ; $level ; $element ; $common_styles ; $styles ; Some (Cow::from (stringify! ($new_alias))) );
    }};

    ( compose play ; $orchestra:expr ; $level:expr ; ( * $link:ident ) ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        use $crate::model::yamlette::literal::LiteralValue;
        use $crate::model::TaggedValue;
        $orchestra.play ($level, TaggedValue::from (LiteralValue::from (format! ("*{}", stringify! ($link))))).ok ().unwrap ();
    }};

    ( compose play ; $orchestra:expr ; $level:expr ; ( $element:tt ) ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        yamlette!(compose play ; $orchestra ; $level ; $element ; $common_styles ; $styles ; $alias );
    }};

    ( compose play ; $orchestra:expr ; $level:expr ; ( $element:expr ) ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        yamlette!(compose unit ; $orchestra ; $level ; $element ; $common_styles ; $styles ; $alias );
    }};


    ( compose play ; $orchestra:expr ; $level:expr ; $element:expr ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        yamlette!(compose unit ; $orchestra ; $level ; $element ; $common_styles ; $styles ; $alias );
    }};


    ( compose unit ; $orchestra:expr ; $level:expr ; $element:expr ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        use $crate::orchestra::chord::Chord;

        let styles: &mut [ &mut $crate::model::style::Style ] = &mut yamlette!(compose styles ; $styles );

        Chord::play ($element, $orchestra, $level, $alias, $common_styles, styles).ok ().unwrap ()
    }};
}
