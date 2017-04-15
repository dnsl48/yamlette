#[macro_export]
macro_rules! yamlette_compose {
    ( ignore ; $ignored:tt ; $expr:tt ) => { $expr };


    ( orchestra ; $orchestra:expr ; $volumes:tt ) => {{ yamlette_compose! ( volumes ; &$orchestra ; $volumes ; [ ] ; [ ] ) }};


    ( size ; [# $( $style:expr ),* => $( $element:tt ),* ] ) => {{ yamlette_compose! ( size ; [ $( $element ),* ] ) }};

    ( size ; [ $( $element:tt ),* ] ) => {{
        let mut _size = 1;

        $(
            _size += yamlette_compose! ( size ; $element );
        )*

        _size
    }};

    ( size ; {# $( $style:expr ),* => $( $key:tt : $val:tt ),* } ) => {{ yamlette_compose! ( size ; { $( $key : $val ),* } ) }};

    ( size ; { $( $key:tt : $val:tt ),* } ) => {{
        let mut _size = 1;

        $(
            _size += yamlette_compose! ( size ; $key );
            _size += yamlette_compose! ( size ; $val );
        )*

        _size
    }};

    ( size ; ( # $( $style:expr ),* => $elem:tt ) ) => {{ yamlette_compose! ( size ; $elem ) }};

    ( size ; ( # $( $style:expr ),* => $elem:expr ) ) => {{ yamlette_compose! ( size ; $elem ) }};

    ( size ; ( & $alias:ident $elem:tt ) ) => {{ yamlette_compose! ( size ; $elem ) }};

    ( size ; ( & $alias:ident $elem:expr ) ) => {{ yamlette_compose! ( size ; $elem ) }};

    ( size ; ( * $link:ident ) ) => {{ 1 }};

    ( size ; ( $elem:tt ) ) => {{ yamlette_compose! ( size ; $elem ) }};

    ( size ; ( $elem:expr ) ) => {{ yamlette_compose! ( size ; $elem ) }};

    ( size ; $element:expr ) => {{
        use $crate::orchestra::chord::Chord;
        Chord::chord_size (&$element)
    }};


    ( directives ; $orchestra:expr ; $directives:tt ) => {{
        let _tags_count = yamlette_compose! ( directives ; tags count ; $directives );

        if _tags_count > 0 {
            use std::borrow::Cow;

            let mut _tags: Vec<(Cow<'static, str>, Cow<'static, str>)> = Vec::with_capacity (_tags_count);
            yamlette_compose! ( directives ; collect tags ; _tags ; $directives ); 
            $orchestra.directive_tags (_tags).ok ().unwrap ();
        }

        yamlette_compose! ( directives ; others ; $orchestra ; $directives );
    }};

    ( directives ; tags count ; [ $( $directive:tt ),* ] ) => {{
        let mut _size = 0;
        $( _size += yamlette_compose! ( directives ; tag count ; $directive ); )*
        _size
    }};

    ( directives ; tag count ; (TAG ; $shortcut:expr , $handle:expr ) ) => { 1 };
    ( directives ; tag count ; $directive:tt ) => { 0 };

    ( directives ; collect tags ; $vec:expr ; [ $( $directive:tt ),* ] ) => { $( yamlette_compose! ( directive ; collect tags ; $vec ; $directive ); )* };
    ( directive ; collect tags ; $vec:expr ; (TAG ; $shortcut:expr , $handle:expr ) ) => { $vec.push ( (Cow::from ($shortcut) , Cow::from ($handle)) ); };
    ( directive ; collect tags ; $vec:expr ; $directive:tt ) => {{ }};

    ( directives ; others ; $orchestra:expr ; [ $( $directive:tt ),* ] ) => {{ $( yamlette_compose! ( directive ; others ; $orchestra ; $directive ); )* }};
    ( directive ; others ; $orchestra:expr ; YAML ) => {{ $orchestra.directive_yaml (true).ok ().unwrap (); }};
    ( directive ; others ; $orchestra:expr ; NO_YAML ) => {{ $orchestra.directive_yaml (false).ok ().unwrap (); }};
    ( directive ; others ; $orchestra:expr ; BORDER_TOP ) => {{ $orchestra.volume_border_top (true).ok ().unwrap (); }};
    ( directive ; others ; $orchestra:expr ; NO_BORDER_TOP ) => {{ $orchestra.volume_border_top (false).ok ().unwrap (); }};
    ( directive ; others ; $orchestra:expr ; BORDER_BOT ) => {{ $orchestra.volume_border_bot (true).ok ().unwrap (); }};
    ( directive ; others ; $orchestra:expr ; NO_BORDER_BOT ) => {{ $orchestra.volume_border_bot (false).ok ().unwrap (); }};
    ( directive ; others ; $orchestra:expr ; (TAG ; $shortcut:expr , $handle:expr ) ) => {};


    ( styles ; [ $( $style:expr ),* ] ) => { [ $( &mut $style as &mut $crate::model::style::Style ),* ] };

    ( styles ; apply to common ; $common_styles:expr ; $styles:tt ) => {{
        let mut cstyles = $common_styles;

        let styles: &mut [ &mut $crate::model::style::Style ] = &mut yamlette_compose! ( styles ; $styles );

        for style in styles {
            style.common_styles_apply (&mut cstyles);
        }

        cstyles
    }};


    ( volumes ; $orchestra:expr ; [ # $( $style:expr ),* => % $( $directive:tt ),* => $( $volume:tt ),* ] ; [ ] ; [ ] ) => {{
        yamlette_compose! ( volumes ; $orchestra ; [ $( $volume ),* ] ; [ $( $style ),* ] ; [ $( $directive ),* ] )
    }};


    ( volumes ; $orchestra:expr ; [ % $( $directive:tt ),* => $( $volume:tt ),* ] ; [ ] ; [ ] ) => {{
        yamlette_compose! ( volumes ; $orchestra ; [ $( $volume ),* ] ; [ ] ; [ $( $directive ),* ] )
    }};


    ( volumes ; $orchestra:expr ; [ # $( $style:expr ),* => $( $volume:tt ),* ] ; [ ] ; [ ] ) => {{
        yamlette_compose! ( volumes ; $orchestra ; [ $( $volume ),* ] ; [ $( $style ),* ] ; [ ] )
    }};


    ( volumes ; $orchestra:expr ; [ $( $volume:tt ),* ] ; $styles:tt ; $directives:tt ) => {{
        let mut _size = 0;

        $( yamlette_compose! ( ignore ; $volume ; { _size += 1; } ); )*

        $orchestra.volumes (_size).ok ().unwrap ();

        let _common_styles = $orchestra.get_styles ();

        $(
            yamlette_compose! ( volume ; $orchestra ; _common_styles ; $volume ; $styles ; $directives );

            $orchestra.vol_end ().ok ().unwrap ();
        )*

        $orchestra.the_end ().ok ().unwrap ();
    }};


    ( volume ; $orchestra:expr ; $common_styles:expr ; [ # $( $style:expr ),+ => % $( $directive:tt ),+ => $( $rule:tt ),* ] ; [ ] ; [ ] ) => {{
        yamlette_compose! ( volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; [ $( $style ),* ] ; [ $( $directive ),* ] );
    }};

    ( volume ; $orchestra:expr ; $common_styles:expr ; [ # $( $style:expr ),+ => % $( $directive:tt ),+ => $( $rule:tt ),* ] ; [ $( $parent_style:expr ),* ] ; [ ] ) => {{
        yamlette_compose! ( volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; [ $( $parent_style ),* , $( $style ),* ] ; [ $( $directive ),* ] );
    }};


    ( volume ; $orchestra:expr ; $common_styles:expr ; [ # $( $style:expr ),+ => % $( $directive:tt ),+ => $( $rule:tt ),* ] ; [ ] ; [ $( $parent_directive:tt ),* ] ) => {{
        yamlette_compose! ( volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; [ $( $style ),* ] ; [ $( $parent_directive ),* , $( $directive ),* ] );
    }};


    ( volume ; $orchestra:expr ; $common_styles:expr ; [ # $( $style:expr ),+ => % $( $directive:tt ),+ => $( $rule:tt ),* ] ; [ $( $parent_style:expr ),* ] ; [ $( $parent_directive:tt ),* ] ) => {{
        yamlette_compose! ( volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; [ $( $parent_style ),* , $( $style ),* ] ; [ $( $parent_directive ),* , $( $directive ),* ] );
    }};


    ( volume ; $orchestra:expr ; $common_styles:expr ; [ % $( $directive:tt ),* => $( $rule:tt ),* ] ; $styles:tt ; [ ] ) => {{
        yamlette_compose! ( volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; $styles ; [ $( $directive ),* ] );
    }};


    ( volume ; $orchestra:expr ; $common_styles:expr ; [ % $( $directive:tt ),* => $( $rule:tt ),* ] ; $styles:tt ; [ $( $parent_directive:tt ),* ] ) => {{
        yamlette_compose! ( volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; $styles ; [ $( $parent_directive ),* , $( $directive ),* ] );
    }};


    ( volume ; $orchestra:expr ; $common_styles:expr ; [ # $( $style:expr ),* => $( $rule:tt ),* ] ; [ ] ; $directives:tt ) => {{
        yamlette_compose! ( volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; [ $( $style ),* ] ; $directives );
    }};


    ( volume ; $orchestra:expr ; $common_styles:expr ; [ # $( $style:expr ),* => $( $rule:tt ),* ] ; [ $( $parent_style:expr ),* ] ; $directives:tt ) => {{
        yamlette_compose! ( volume ; $orchestra ; $common_styles ; [ $( $rule ),* ] ; [ $( $parent_style ),* , $( $style ),* ] ; $directives );
    }};


    ( volume ; $orchestra:expr ; $common_styles:expr ; [ $( $rules:tt ),* ] ; $styles:tt ; $directives:tt ) => {{
        let mut _size = 0;

        $orchestra.vol_next ().ok ().unwrap ();

        yamlette_compose! ( directives ; $orchestra ; $directives );

        $( yamlette_compose! ( ignore ; $rules ; { _size += yamlette_compose! ( size ; $rules ); } ); )*

        $orchestra.vol_reserve (_size).ok ().unwrap ();

        let _common_styles = yamlette_compose! ( styles ; apply to common ; $common_styles ; $styles );

        $(
            yamlette_compose! ( play ; $orchestra ; 0 ; $rules ; _common_styles ; $styles ; None );
        )*
    }};


    ( play ; $orchestra:expr ; $level:expr ; [ # $( $style:expr ),* => $( $element:tt ),* ] ; $common_styles:expr ; [] ; $alias:expr ) => {{
        yamlette_compose! ( play ; $orchestra ; $level ; [ $( $element ),* ] ; $common_styles ; [ $( $style ),* ] ; $alias )
    }};


    ( play ; $orchestra:expr ; $level:expr ; [ # $( $style:expr ),* => $( $element:tt ),* ] ; $common_styles:expr ; [ $( $parent_style:expr ),+ ] ; $alias:expr ) => {{
        yamlette_compose! ( play ; $orchestra ; $level ; [ $( $element ),* ] ; $common_styles ; [ $( $parent_style ),* , $( $style ),* ] ; $alias )
    }};


    ( play ; $orchestra:expr ; $level:expr ; [ $( $element:tt ),* ] ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        use $crate::orchestra::chord::{ Chord, EmptyList };

        let styles: &mut [ &mut $crate::model::style::Style ] = &mut yamlette_compose! ( styles ; $styles );

        Chord::play (EmptyList, $orchestra, $level, $alias, $common_styles, styles).ok ().unwrap ();

        let _common_styles = yamlette_compose! ( styles ; apply to common ; $common_styles ; $styles );

        $(
            yamlette_compose! ( play ; $orchestra ; $level + 1 ; $element ; _common_styles ; $styles ; None );
        )*
    }};


    ( play ; $orchestra:expr ; $level:expr ; { # $( $style:expr ),* => $( $key:tt : $val:tt ),* } ; $common_styles:expr ; [] ; $alias:expr ) => {{
        yamlette_compose! ( play ; $orchestra ; $level ; { $( $key : $val ),* } ; $common_styles ; [ $( $style ),* ] ; $alias )
    }};


    ( play ; $orchestra:expr ; $level:expr ; { # $( $style:expr ),* => $( $key:tt : $val:tt ),* } ; $common_styles:expr ; [ $( $parent_style:expr ),+ ] ; $alias:expr ) => {{
        yamlette_compose! ( play ; $orchestra ; $level ; { $( $key : $val ),* } ; $common_styles ; [ $( $parent_style ),* , $( $style ),* ] ; $alias )
    }};


    ( play ; $orchestra:expr ; $level:expr ; { $( $key:tt : $val:tt ),* } ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        use $crate::orchestra::chord::{ Chord, EmptyDict };

        let styles: &mut [ &mut $crate::model::style::Style ] = &mut yamlette_compose! ( styles ; $styles );

        Chord::play (EmptyDict, $orchestra, $level, $alias, $common_styles, styles).ok ().unwrap ();

        let _common_styles = yamlette_compose! ( styles ; apply to common ; $common_styles ; $styles );

        $(
            yamlette_compose! ( play ; $orchestra ; $level + 1 ; $key ; _common_styles ; $styles ; None );
            yamlette_compose! ( play ; $orchestra ; $level + 1 ; $val ; _common_styles ; $styles ; None );
        )*
    }};


    ( play ; $orchestra:expr ; $level:expr ; ( # $( $style:expr ),* => $element:tt ) ; $common_styles:expr ; [ ] ; $alias:expr ) => {{
        let _common_styles = yamlette_compose! ( styles ; apply to common ; $common_styles ; [ $( $style ),* ] );

        yamlette_compose! ( play ; $orchestra ; $level ; $element ; _common_styles ; [ $( $style ),* ] ; $alias );
    }};

    ( play ; $orchestra:expr ; $level:expr ; ( # $( $style:expr ),* => $element:tt ) ; $common_styles:expr ; [ $( $parent_style:expr ),* ] ; $alias:expr ) => {{
        let _common_styles = yamlette_compose! ( styles ; apply to common ; $common_styles ; [ $( $style ),* ] );

        yamlette_compose! ( play ; $orchestra ; $level ; $element ; _common_styles ; [ $( $parent_style ),* , $( $style ),* ] ; $alias );
    }};

    ( play ; $orchestra:expr ; $level:expr ; ( # $( $style:expr ),* => $element:expr ) ; $common_styles:expr ; [ ] ; $alias:expr ) => {{
        let _common_styles = yamlette_compose! ( styles ; apply to common ; $common_styles ; [ $( $style ),* ] );

        yamlette_compose! ( play ; $orchestra ; $level ; $element ; _common_styles ; [ $( $style ),* ] ; $alias );
    }};

    ( play ; $orchestra:expr ; $level:expr ; ( # $( $style:expr ),* => $element:expr ) ; $common_styles:expr ; [ $( $parent_style:expr ),* ] ; $alias:expr ) => {{
        let _common_styles = yamlette_compose! ( styles ; apply to common ; $common_styles ; [ $( $style ),* ] );

        yamlette_compose! ( unit ; $orchestra ; $level ; $element ; _common_styles ; [ $( $parent_style ),* , $( $style ),* ] ; $alias );
    }};

    ( play ; $orchestra:expr ; $level:expr ; ( & $new_alias:ident $element:tt ) ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        use std::borrow::Cow;
        yamlette_compose! ( play ; $orchestra ; $level ; $element ; $common_styles ; $styles ; Some (Cow::from (stringify! ($new_alias))) );
    }};

    ( play ; $orchestra:expr ; $level:expr ; ( & $new_alias:ident $element:expr ) ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        use std::borrow::Cow;
        yamlette_compose! ( play ; $orchestra ; $level ; $element ; $common_styles ; $styles ; Some (Cow::from (stringify! ($new_alias))) );
    }};

    ( play ; $orchestra:expr ; $level:expr ; ( * $link:ident ) ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        use $crate::model::yamlette::literal::LiteralValue;
        use $crate::model::TaggedValue;
        $orchestra.play ($level, TaggedValue::from (LiteralValue::from (format! ("*{}", stringify! ($link))))).ok ().unwrap ();
    }};

    ( play ; $orchestra:expr ; $level:expr ; ( $element:tt ) ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        yamlette_compose! ( play ; $orchestra ; $level ; $element ; $common_styles ; $styles ; $alias );
    }};

    ( play ; $orchestra:expr ; $level:expr ; ( $element:expr ) ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        yamlette_compose! ( unit ; $orchestra ; $level ; $element ; $common_styles ; $styles ; $alias );
    }};


    ( play ; $orchestra:expr ; $level:expr ; $element:expr ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        yamlette_compose! ( unit ; $orchestra ; $level ; $element ; $common_styles ; $styles ; $alias );
    }};


    ( unit ; $orchestra:expr ; $level:expr ; $element:expr ; $common_styles:expr ; $styles:tt ; $alias:expr ) => {{
        use $crate::orchestra::chord::Chord;

        let styles: &mut [ &mut $crate::model::style::Style ] = &mut yamlette_compose! ( styles ; $styles );

        Chord::play ($element, $orchestra, $level, $alias, $common_styles, styles).ok ().unwrap ()
    }};
}




#[cfg (all (test, not (feature = "dev")))]
mod tests {
    #[test]
    fn size () {
        let size = yamlette_compose! ( size ; "halo" );
        assert_eq! (1, size);

        let size = yamlette_compose! ( size ; () );
        assert_eq! (1, size);

        let size = yamlette_compose! ( size ; [ (), 1, "2", [ 4, { "a": 1, "b": 4 }, 3 ], () ] );
        assert_eq! (13, size);
    }
}
