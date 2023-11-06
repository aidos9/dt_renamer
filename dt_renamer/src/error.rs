use std::io;

#[derive(Debug)]
pub enum Error {
    WalkerError(dt_walker::Error),
    NotDirectory(String),
    NotFile(String),
    DuplicateFileError(String),
    RenameError(io::Error),
    CanonicalizeError(io::Error),
    ReadDirError(io::Error),
    ReadDirEntryError(io::Error),
    CannotIdentifyFileName,
    InsertIndexTooLarge
}
