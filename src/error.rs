pub enum Error {
    WalkerError(dt_walker::Error),
    NotDirectory(String),
    NotFile(String),
}
