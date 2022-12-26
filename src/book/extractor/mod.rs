pub mod pointer;
pub mod traits;

pub use self::pointer::Pointer;
pub use self::traits::FromPointer;

#[macro_export]
macro_rules! yamlette_reckon {
    ( book ; $book:expr ; [ $( $rules:tt ),* ] ) => {
        let mut _counter: usize = 0;
        $(
            let volume = $book.volumes.get (_counter);
            $crate::yamlette_reckon! ( volume ; volume ; $rules );
            _counter += 1;
        )*
    };


    ( volume ; $volume:expr ; [ $( $rules:tt ),* ] ) => {
        let _pointer = if let Some (volume) = $volume { $crate::book::extractor::pointer::Pointer::new (volume) } else { None };
        $(
            $crate::yamlette_reckon! ( ptr ; _pointer ; $rules );
            let _pointer = if let Some (p) = _pointer { p.next_sibling () } else { None };
        )*
    };


    ( ptr ; $pointer:expr ; [ $( $v:tt ),* ] ) => {
        let _pointer = if let Some (p) = $pointer { p.into_seq () } else { None };
        $(
            $crate::yamlette_reckon! ( ptr ; _pointer ; $v );
            let _pointer = if let Some (p) = _pointer { p.next_sibling () } else { None };
        )*
    };


    ( ptr ; $pointer:expr ; { $( $k:tt > $v:tt ),* } ) => {
        let _pointer = if let Some (p) = $pointer { p.into_map () } else { None };
        $(
            $crate::yamlette_reckon! ( ptr ; _pointer ; $k );
            let _pointer = if let Some (p) = _pointer { p.next_sibling () } else { None };
            $crate::yamlette_reckon! ( ptr ; _pointer ; $v );
            let _pointer = if let Some (p) = _pointer { p.next_sibling () } else { None };
        )*
    };


    ( ptr ; $pointer:expr ; { $( $k:expr => $v:tt ),* } ) => {
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
            $crate::yamlette_reckon! ( ptr ; _pointer ; $v );
        )*
    };


    ( ptr ; $pointer:expr ; { $( ( $v:ident:$t:ty ) ),* } ) => {
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


    ( ptr ; $pointer:expr ; ( $($v:ident:$t:ty),* ) ) => {
        $(
            let $v: Option<$t> = if let Some (p) = $pointer {
                use $crate::book::extractor::traits::FromPointer;
                <$t as FromPointer>::from_pointer (p)
            } else { None };
        )*
    };


    ( ptr ; $pointer:expr ; (list $($v:ident:$t:ty),* ) ) => {
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


    ( ptr ; $pointer:expr ; (dict $($v:ident:$t:ty),* ) ) => {
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


    ( ptr ; $pointer:expr ; (call $($f:expr),*) ) => {
        use $crate::book::extractor::pointer::Pointer;
        fn _clo<'a, F>(clo: &mut F, ptr: Pointer<'a>) where F: FnMut(Pointer<'a>) -> () { clo (ptr); }

        $(
            if $pointer.is_some () {
                _clo ($f, $pointer.unwrap ());
            }
        )*
    };


    ( ptr ; $pointer:expr ; (foreach $($f:expr),*) ) => {
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
}
