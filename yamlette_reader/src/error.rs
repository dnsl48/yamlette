use nom_supreme::error::ErrorTree;

pub type Error<I> = ErrorTree<I>;


// pub enum ErrorKind {
//     NomErrorKind(nom::error::ErrorKind)
// }

// impl From<nom::error::ErrorKind> for ErrorKind {
//     fn from(error_kind: nom::error::ErrorKind) -> Self {
//         ErrorKind::NomErrorKind(error_kind)
//     }
// }


// pub struct ParseError<I> {
//     input: I,
//     code: ErrorKind
// }

// #[derive(thiserror::Error, Debug)]
// pub enum Error<I> {
//     Parse(ParseError<I>)
// }

// // impl nom::error::ParseError for Error {

// // }