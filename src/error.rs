use std::fmt;

#[derive(Debug)]
pub enum DynHashError {
    FailedToWriteUpdated,
    StdIoError(std::io::Error),
}
// Implement display trait for DynHashError
impl fmt::Display for DynHashError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Think of more meaningful display messages
        write!(f, "error")
    }
}
/// Implement error conversion (`std::io::Error` -> `DynHashError`)
impl From<std::io::Error> for DynHashError {
    fn from(err: std::io::Error) -> DynHashError {
        DynHashError::StdIoError(err)
    }
}
