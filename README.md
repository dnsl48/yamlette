# Yamlette

Comprehensive YAML 1.2 processor implemented in Rust
------

[![Current Version on crates.io](https://img.shields.io/crates/v/yamlette.svg)](https://crates.io/crates/yamlette/) [![MIT / Apache2 License](https://img.shields.io/badge/license-MIT%20/%20Apache2-blue.svg)]() [![Build Status](https://travis-ci.org/dnsl48/yamlette.svg?branch=master)](https://travis-ci.org/dnsl48/yamlette) [![Documentation](https://docs.rs/yamlette/badge.svg)](https://docs.rs/yamlette/)
------


## Features

 - complete support of the YAML 1.2 specification
 - test coverage for every example of YAML 1.2 specification
 - handy macros for YAML reading and writing
 - control over output formatting in writing mode
 - easy to deserialize your own types through a trait implementation (`FromPointer<'a>`)
 - somewhat unstable although possible to serialize your own types through a trait implementation (orchestra::chord::Chord)
 - experimental multithreading model (although at the moment it's much slower than single-threaded implementation and not recommended)


# Examples

The `yamlette!` macro performs all the job so that you can use library without knowing how it works internally.

The main idea is that you describe the data structure you work with instead of juggling with objects and methods.

There is a format explanation after the examples, since both Reader and Writer have it similarly.


## Basic reader example

```rust
#[macro_use]
extern crate yamlette;

const SRC_YAML: &'static str = r#"
sequence:
- one
- two
mapping:
  ? sky
  : blue
  sea : green
"#;

fn main () {
    yamlette! ( read ; SRC_YAML ; [[
        {
            "sequence" => (list seq:Vec<String>),
            "mapping" => {
                "sky" => (sky_color:&str),
                "sea" => (sea_color:String)
            }
        }
    ]] );

    assert! (seq.is_some ());
    let seq = seq.unwrap ();

    assert_eq! (seq.len (), 2);
    assert_eq! (seq[0], "one");
    assert_eq! (seq[1], "two");

    assert_eq! (sky_color, Some ("blue"));
    assert_eq! (sea_color, Some (String::from ("green")));
}
```

Its first argument is not a variable, but rather a literal that says we need to call reader functionality here.

The second argument is the data source for reading. It can be of the next types:
 - `&'static string`
 - `String`
 - `Vec<u8>`
 - `std::fs::File`
 - any object implementing `skimmer::reader::IntoReader` trait

The third argument is the data structure description. Its format is described below (after the examples).

`yamlette!` macro does not return anything by default when called as a reader.


## Basic writer example


```rust
#[macro_use]
extern crate yamlette;

use std::collections::BTreeMap;

const TGT_YAML: &'static str =
r#"name: Martin D'vloper
job: Developer
employed: true
foods:
  - Apple
  - Orange
  - Strawberry
  - Mango
languages:
  pascal: Lame
  perl: Elite
  python: Elite
education: "4 GCSEs\n3 A-Levels\nBSc in the Internet of Things"
"#;

fn main () {
    let name = "Martin D'vloper";
    let employed = true;

    let foods = vec! ["Apple", "Orange", "Strawberry", "Mango"];

    let mut languages = BTreeMap::new ();
    languages.insert ("pascal", "Lame");
    languages.insert ("perl", "Elite");
    languages.insert ("python", "Elite");

    let education = "4 GCSEs\n3 A-Levels\nBSc in the Internet of Things";

    let string = yamlette! ( write ; [[ {
        "name": name,
        "job": "Developer",
        "employed": employed,
        "foods": foods,
        "languages": languages,
        "education": education
    } ]] ).ok ().unwrap ();

    assert_eq! (string, TGT_YAML);
}
```

Its first argument is not a variable, but rather a literal that says we need to call writer functionality here.

The second argument is the data structure description. Its format is described below (after the examples).

`yamlette!` macro returns `Result<String, yamlette::orchestra::OrchError>` instance by default when called as a writer.


## Custom type read example

You simply need to implement `FromPointer<'a>` trait for your type

```rust
extern crate yamlette;

use self::yamlette::book::extractor::pointer::Pointer;
use self::yamlette::book::extractor::traits::FromPointer;

#[derive (PartialEq, Eq, Debug)]
struct State {
    pub state: bool,
    pub transition: u8,
    pub brightness: u8
}

impl State {
    pub fn new (state: bool, transition: u8, brightness: u8) -> State {
        State {
            state: state,
            transition: transition,
            brightness: brightness
        }
    }
}

impl<'a> FromPointer<'a> for State {
    fn from_pointer (pointer: Pointer<'a>) -> Option<Self> {
        yamlette_reckon! ( ptr ; Some (pointer) ; {
            (state:bool),
            (transition:u8),
            (brightness:u8)
        } );

        Some (State {
            state: if let Some (s) = state { s } else { false },
            transition: if let Some (t) = transition { t } else { 0u8 },
            brightness: if let Some (b) = brightness { b } else { 0u8 }
        })
    }
}
```


## The format description

##### Common things
The main idea behind the format is that you can express in JSON-ish format the structure of your data.
However, stream of YAML may contain several documents, so that we enumerate them too as a top-level list, embraced with square brackets.
There is another top level of square brackets so that Rust macro engine can read all the data description as a single token tree.

That's why we always have two square brackets at the top:
 - the first one for Rust macro engine
 - the second one embracing a YAML document (we can have many of these, but we always must have at least one)

Meaning of that is we can describe an empty YAML document with the next structure: `[[]]`.
Two empty YAML document would be: `[ [], [] ]` etc.

Within the document we start describing the actual data nodes.
There are three possible node types:
 - Dictionary (Map), which can be described with curly brackets: `{}`
 - Sequence (List), which can be described with square brackets: `[]`
 - Scalar (Node), which can be described with parentheses (or sometimes even without): `()`

Particularities between data types are different between reading and writing.

##### Reader specifics of the format
 When we read a dictionary, we enumerate its keys and values (which are nodes by themselves).

 There are two ways to enumerate a dictionary:
  - `{ key => value }` - look up the value by the key value (applying `<key as PartialEq>::eq`)
  - `{ key > value }` - simply go through dictionary keys and values and assign their values in the original order

 The JSONish form `{ key : value }` is intentionally left unimplemented, since it's not obvious at this stage of the project which behaviour should be default.

 When it comes to a scalar value, there are several options: we embrace it with parentheses
  - `(var:type)`  - variable name with its type into which reader should try to cast the read value
  - `(list var:type)` - handle the node as a collection; the type should implement `yamlette::book::extractor::traits::List` trait
  - `(dict var:type)` - handle the node as a collection; the type should implement `yamlette::book::extractor::traits::Dict` trait
  - `(call FnOnce [, FnOnce[, ...]]])` - for advanced users; call a number of custom callback functions over the node
  - `(foreach FnOnce [, FnOnce[, ...]]]` - for advanced users; call a number of custom callback functions over all siblings of the node

 There is nothing specific about sequences, fortunatelly.

##### Write specifics of the format
 When we write a dictionary, we just enumerate its keys and values through a colon with comma, the same way as in JSON:
  - `{ key : value }`

 When it comes to a scalar value, we just need to provide a value, without any syntactic sugar.
 It is a thumb-up rule to avoid passing expressions or blocks to the macro, since it might
 interfere with the macro format itself.
 However, if you need to provide an expression - you might want to embrace it with parentheses.

 When we write stuff, we might want to choose some styling around how the processor should generate output.

 There are two additional things to know:
  - Directives
  - Styles

 Styles are applicable to everything, and they can be put:
  - right after the macro token opening square bracket (this will apply rules for all documents)
  - right after a document opening square bracket (this will apply rules for all nodes in the document)
  - right after a seq opening square bracket (this will apply rules for all nodes within the list, but not the list itself)
  - right after a map opening curly bracket (this will apply rules for all nodes within the dict, but not the dict itself)
  - right after a round bracket (for a node)

 Directives are applicable to a YAML document, and they can be put:
  - after the macro token opening square bracket (this will apply rules for all documents).
  - right after the document opening square bracket (this will apply rules for the particular document)

 If you need to set up styles and directives on the same level, then Styles go first, and then Directives:
  - `[ % YAML, BORDER_TOP, NO_BORDER_BOT => # FLOW => [ ... ]]`

 The list of directives should start with a percent sign (%) and end with a equal+greater_than signs, which had to be chosen because of
 some Rust macro engine limitations and not to interfere with existing Rust language syntax.

 The list of styles should start with a hashtag sign (#) and end with a equal+greater_than signs.

 Possible directives:
  - YAML - print out %YAML directive
  - BORDER_TOP - print out top border of the document (---)
  - BORDER_BOT - print out bottom border of the document (...), which is automatically generated in case of several YAML documents
  - NO_YAML - discard YAML directive
  - NO_BORDER_TOP - discard BORDER_TOP directive
  - NO_BORDER_BOT - discard BORDER_BOT directive
  - (TAG ; handle , prefix ) - issue your custom %TAG directive

 There are two types of styles: CommonStyles and Model styles. Model styles are once applied for only a single data type (data model), like !!str or !!int.
 Common styles are applicable for more than one data type (data model).

 There are some CommonStyles implemented already:
  - `yamlette::model::style::Indent` - allows to change indent size (2 spaces by default)
  - `yamlette::model::style::Flow`   - flow mode (JSONish mode)
  - `yamlette::model::style::Compact` - perform as compact output as possible (eyes bleeding mode)
  - `yamlette::model::style::Multiline` - do as much newlines as possible (useful with FLOW mode)
  - `yamlette::model::style::IssueTag` - issue node tags (yes you're right, !!str, !!int, !!map, !!timezone etc)
  - `yamlette::model::style::RespectThreshold` - make a newline in case the line gets too long
  - `yamlette::model::style::Threshold` - max number of characters per line for RespectThreshold mode

 Model styles are supposedly change some formatting and mostly depend on a use case. There are already some of them implemented, though:
  - `yamlette::model::yaml::str::ForceQuotes` - embrace a string with quotes (even if there are no any special chars or line feeds)
  - `yamlette::model::yaml::str::PreferDoubleQuotes` - if ForceQuotes, use `"` instead of `'`
  - other ones probably will be implemented in new versions of the library (e.g. number formatters for !!int, !!float and !!timestamp)

 Here is an example of the described above:

```rust
#[macro_use]
extern crate yamlette;

use std::collections::BTreeMap;

const TGT_YAML: &'static str =
r#"%YAML 1.2
%TAG !custom-bool-tag! tag:yaml.org,2002:boo
---
name: Martin D'vloper
job: 'Developer'
employed: !custom-bool-tag!l true
foods: [ Apple, Orange, Strawberry, Mango ]
languages: {
    pascal: Lame,
    perl: Elite,
    python: Elite
}
...
"#;

fn main () {
    let name = "Martin D'vloper";
    let employed = true;

    let foods = vec! ["Apple", "Orange", "Strawberry", "Mango"];

    let mut languages = BTreeMap::new ();
    languages.insert ("pascal", "Lame");
    languages.insert ("perl", "Elite");
    languages.insert ("python", "Elite");

    use yamlette::model::style::{ FLOW, MULTILINE, ISSUE_TAG, Indent };
    use yamlette::model::yaml::str::FORCE_QUOTES;

    let string = yamlette! ( write ; [
        % YAML, ( TAG ; "!custom-bool-tag!" , "tag:yaml.org,2002:boo" ) =>
        [
            { # FLOW =>
                "name": name,
                "job": ( # FORCE_QUOTES => "Developer" ),
                "employed": ( # ISSUE_TAG => employed ),
                "foods": foods,
                "languages": ( # MULTILINE, Indent (4) => languages )
            }
        ]
    ] ).ok ().unwrap ();

    assert_eq! (string, TGT_YAML);
}
```

# License

License: `Double: MIT / Apache License, Version 2.0`