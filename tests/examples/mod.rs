#[cfg (all (test, any ( not (feature = "onetest"), all (feature = "onetest", feature = "test_reader"))))]
pub mod reader;
#[cfg (all (test, any ( not (feature = "onetest"), all (feature = "onetest", feature = "test_savant"))))]
pub mod savant;
#[cfg (all (test, any ( not (feature = "onetest"), all (feature = "onetest", feature = "test_sage"))))]
pub mod sage;
#[cfg (all (test, any ( not (feature = "onetest"), all (feature = "onetest", feature = "test_book"))))]
pub mod book;
#[cfg (all (test, any ( not (feature = "onetest"), all (feature = "onetest", feature = "test_orchestra"))))]
pub mod orchestra;
#[cfg (all (test, any ( not (feature = "onetest"), all (feature = "onetest", feature = "test_face"))))]
pub mod face;
