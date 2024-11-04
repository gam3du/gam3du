//! Provides mechanisms to retrieve file-like data from several backends
#![allow(missing_docs, reason = "TODO")]
#![expect(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    reason = "TODO"
)]

use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
};

pub trait FileStorage {
    fn get_content(&self, path: &Path) -> Result<Cow<'_, [u8]>, String>;
}

#[derive(Default)]
pub struct StaticStorage {
    files: HashMap<PathBuf, Vec<u8>>,
}

impl StaticStorage {
    pub fn store(&mut self, path: impl Into<PathBuf>, content: Vec<u8>) {
        assert!(
            self.files.insert(path.into(), content).is_none(),
            "duplicate path"
        );
    }
}

impl FileStorage for StaticStorage {
    fn get_content(&self, path: &Path) -> Result<Cow<'_, [u8]>, String> {
        match self.files.get(path) {
            Some(file) => Ok(Cow::Borrowed(file)),
            None => Err(format!(
                "File not found in static storage: {}",
                path.display()
            )),
        }
    }
}
