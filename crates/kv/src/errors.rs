#[derive(Debug)]
pub enum Error {
    OpenFileError,
    StreamError,
    FileNotFound(u64),
    InvalidData,
    ParseError
}
