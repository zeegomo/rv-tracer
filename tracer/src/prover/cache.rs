use serde::{de::DeserializeOwned, Serialize};
use std::{io::Write};
use tempdir::TempDir;

pub trait Cache {
    fn put<T: Serialize>(&self, key: &str, item: &T);
    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T>;
}

pub struct NoCache;

impl Cache for NoCache {
    fn put<T: Serialize>(&self, _key: &str, _item: &T) {}
    fn get<T: DeserializeOwned>(&self, _key: &str) -> Option<T> {
        None
    }
}

pub struct DiskCache {
    dir: TempDir,
}

impl DiskCache {
    pub fn new() -> Self {
        DiskCache {
            dir: TempDir::new("prove_cache").unwrap(),
        }
    }
}

impl Default for DiskCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache for DiskCache {
    fn put<T: Serialize>(&self, key: &str, item: &T) {
        let serialized = bincode::serialize(item).unwrap();
        let path = self.dir.path().join(key);
        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(&serialized).unwrap();
    }

    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let path = self.dir.path().join(key);

        if !path.exists() {
            return None;
        }

        let file = std::fs::File::open(path).unwrap();
        let reader = std::io::BufReader::new(file);
        let item = bincode::deserialize_from(reader).unwrap();
        Some(item)
    }
}