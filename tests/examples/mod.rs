#[cfg (all (test, feature = "test_book"))]
pub mod book;

#[cfg (all (test, feature = "test_face"))]
pub mod face;

#[cfg (all (test, feature = "test_orchestra"))]
pub mod orchestra;

#[cfg (all (test, feature = "test_reader"))]
pub mod reader;

#[cfg (all (test, feature = "test_sage"))]
pub mod sage;

#[cfg (all (test, feature = "test_savant"))]
pub mod savant;
