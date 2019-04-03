#[cfg (all (test, any (feature = "test_book", feature = "onetest")))]
pub mod book;

#[cfg (all (test, any (feature = "test_face", feature = "onetest")))]
pub mod face;

#[cfg (all (test, any (feature = "test_orchestra", feature = "onetest")))]
pub mod orchestra;

#[cfg (all (test, any (feature = "test_reader", feature = "onetest")))]
pub mod reader;

#[cfg (all (test, any (feature = "test_sage", feature = "onetest")))]
pub mod sage;

#[cfg (all (test, any (feature = "test_savant", feature = "onetest")))]
pub mod savant;
