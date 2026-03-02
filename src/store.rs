use serde_json::Value;
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

/// An in-memory document store holding a single JSON document.
/// Loaded once at startup and shared across all requests.
#[derive(Debug, Clone)]
pub struct Store {
    document: Value,
}

#[derive(Debug)]
pub enum StoreError {
    /// The file could not be read.
    IoError(std::io::Error),
    /// The file contents are not valid JSON.
    ParseError(serde_json::Error),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::IoError(e)    => write!(f, "Could not read store file: {e}"),
            StoreError::ParseError(e) => write!(f, "Store file is not valid JSON: {e}"),
        }
    }
}

impl Store {
    /// Load a JSON file from disk and hold it in memory.
    pub fn load(path: &Path) -> Result<Self, StoreError> {
        let contents = fs::read_to_string(path)
            .map_err(StoreError::IoError)?;
        let document = serde_json::from_str(&contents)
            .map_err(StoreError::ParseError)?;
        Ok(Store { document })
    }

    /// Create a store directly from a JSON value.
    /// Useful for testing.
    pub fn from_value(document: Value) -> Self {
        Store { document }
    }

    /// Return a reference to the root document.
    pub fn document(&self) -> &Value {
        &self.document
    }
}
