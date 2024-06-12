use std::io;
use std::io::ErrorKind;
use std::path::Path;

use derive_more::{Display, Error};
use tora::{ReadStruct, WriteStruct};

#[derive(Display, Debug, Error)]
pub enum LoadLibraryError {
    #[display("The .mw file was invalid.")]
    InvalidData,

    #[display("An IO error occurred: {_0}")]
    Io(io::Error),
}

impl From<io::Error> for LoadLibraryError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(ReadStruct, WriteStruct, Hash, PartialEq, Eq, Debug, Clone)]
pub struct Library {
    name: String,
    description: String,
    author: String,
    path_to_local_dll: String,
}

impl Library {
    pub fn load_library<P>(path: P) -> Result<Library, LoadLibraryError>
    where
        P: AsRef<Path>,
    {
        tora::read_from_file(path).map_err(|e| match e.kind() {
            ErrorKind::InvalidData => LoadLibraryError::InvalidData,
            _ => LoadLibraryError::Io(e),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn path_to_local_dll(&self) -> &str {
        &self.path_to_local_dll
    }

    pub fn from(name: &str, description: &str, author: &str, path_to_local_dll: &str) -> Self {
        Self::new(
            name.to_string(),
            description.to_string(),
            author.to_string(),
            path_to_local_dll.to_string(),
        )
    }

    pub const fn new(
        name: String,
        description: String,
        author: String,
        path_to_local_dll: String,
    ) -> Self {
        Self {
            name,
            description,
            author,
            path_to_local_dll,
        }
    }
}
