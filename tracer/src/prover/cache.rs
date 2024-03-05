use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
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
    cache: Mutex<HashMap<PathBuf, Vec<u8>>>,
}

impl DiskCache {
    pub fn new() -> Self {
        DiskCache {
            dir: TempDir::new("prove_cache").unwrap(),
            cache: Default::default(),
        }
    }
}

impl Cache for DiskCache {
    fn put<T: Serialize>(&self, key: &str, item: &T) {
        let serialized = bincode::serialize(item).unwrap();
        let path = self.dir.path().join(key);
        println!(
            "saving {} bytes to {}",
            calculate_hash(&serialized),
            path.display()
        );
        // let mut file = std::fs::File::create(path).unwrap();
        // file.write_all(&serialized).unwrap();
        self.cache.lock().unwrap().insert(path, serialized);
    }

    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let path = self.dir.path().join(key);

        if !self.cache.lock().unwrap().get(&path).is_some() {
            return None;
        }

        println!(
            "loaded {} from  {}",
            calculate_hash(self.cache.lock().unwrap().get(&path).unwrap()),
            path.display()
        );

        // let file = std::fs::File::open(path).unwrap();
        // let reader = std::io::BufReader::new(file);
        let item = bincode::deserialize(self.cache.lock().unwrap().get(&path).unwrap()).unwrap();
        Some(item)
    }
}

impl Drop for DiskCache {
    fn drop(&mut self) {
        println!("DROPPPING");
    }
}
use std::hash::{DefaultHasher, Hash, Hasher};

fn calculate_hash(x: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}
