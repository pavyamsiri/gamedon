use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur when reading a binary.
#[derive(Debug, Error)]
pub enum ReadBinaryError {
    /// IO related errors.
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Read a binary file into a vector.
pub fn read_binary_file(path: impl AsRef<Path>) -> Result<Vec<u8>, ReadBinaryError> {
    fn read(path: &Path) -> Result<Vec<u8>, ReadBinaryError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
    read(path.as_ref())
}
