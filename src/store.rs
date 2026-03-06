use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

// ---------------------------------------------------------------------------
// Store errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum StoreError {
    /// The file could not be read.
    IoError(std::io::Error),
    /// The file contents are not valid JSON.
    ParseError(serde_json::Error),
    /// The data root directory does not exist.
    RootNotFound(PathBuf),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::IoError(e)       => write!(f, "Could not read file: {e}"),
            StoreError::ParseError(e)    => write!(f, "File is not valid JSON: {e}"),
            StoreError::RootNotFound(p)  => write!(f, "Data directory not found: {}", p.display()),
        }
    }
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

/// A lazy, caching document store.
///
/// Documents are loaded from disk on first request and retained in memory.
/// The root directory mirrors the URL path structure — a request for
/// `/users/orders` loads `<root>/users/orders.json`.
pub struct Store {
    /// Root directory from which documents are loaded.
    root: PathBuf,
    /// In-memory cache of loaded documents, keyed by URL path.
    cache: RwLock<HashMap<String, Value>>,
}

impl Store {
    /// Create a store rooted at the given directory.
    pub fn new(root: &Path) -> Result<Self, StoreError> {
        if !root.exists() || !root.is_dir() {
            return Err(StoreError::RootNotFound(root.to_path_buf()));
        }
        Ok(Store {
            root: root.to_path_buf(),
            cache: RwLock::new(HashMap::new()),
        })
    }

    /// Create a store with a single in-memory document at the given path.
    /// Useful for testing and the built-in example.
    pub fn from_value(path: &str, document: Value) -> Self {
        let mut cache = HashMap::new();
        cache.insert(path.to_string(), document);
        Store {
            root: PathBuf::from("."),
            cache: RwLock::new(cache),
        }
    }

    /// Retrieve the document for the given URL path.
    ///
    /// Checks the cache first. On a cache miss, derives the filesystem path
    /// from the URL path, loads the file, caches it, and returns it.
    /// Returns None if no file exists at the derived path.
    pub fn get(&self, url_path: &str) -> Result<Option<Value>, StoreError> {
        // Normalise the path — strip leading slash.
        let key = url_path.trim_start_matches('/');

        // Check the cache first with a read lock.
        {
            let cache = self.cache.read().unwrap();
            if let Some(doc) = cache.get(key) {
                return Ok(Some(doc.clone()));
            }
        }

        // Cache miss — derive the filesystem path and load from disk.
        let file_path = self.resolve(key);

        if !file_path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&file_path)
            .map_err(StoreError::IoError)?;

        let document: Value = serde_json::from_str(&contents)
            .map_err(StoreError::ParseError)?;

        // Store in cache with a write lock.
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(key.to_string(), document.clone());
        }

        println!("Loaded and cached: {}", file_path.display());

        Ok(Some(document))
    }

    /// Derive the filesystem path from a URL path.
    /// `/users/orders` → `<root>/users/orders.json`
    fn resolve(&self, key: &str) -> PathBuf {
        let mut path = self.root.clone();
        for segment in key.split('/') {
            path.push(segment);
        }
        path.set_extension("json");
        path
    }
}
