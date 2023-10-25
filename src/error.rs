use std::io;

#[derive(Debug)]
pub enum Error {
    WalkerError(dt_walker::Error),
    NotDirectory(String),
    NotFile(String),
    RenameError(io::Error),
    CanonicalizeError(io::Error),
    ReadDirError(io::Error),
    ReadDirEntryError(io::Error)
}
