use std::io;

#[derive(Debug)]
pub enum Error {
    ReadDirError(io::Error),
    CanonicalizeError(io::Error),
    MaxDepthReached,
}
