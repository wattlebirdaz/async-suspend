use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::prelude::*;

pub struct State<T: Serialize + DeserializeOwned> {
    pub completed: bool,
    pub id: usize,
    pub data: T,
}

impl<T: Serialize + DeserializeOwned> State<T> {
    pub fn new(id: usize, data: T) -> Self {
        Self {
            completed: false,
            id,
            data,
        }
    }

    pub fn file_name(&self) -> String {
        format!("state-{}", self.id)
    }

    pub fn serialize_data(&self) {
        if !self.completed {
            let data = bincode::serialize(&self.data).unwrap();
            println!("Serializing...");
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(self.file_name())
                .unwrap();
            file.write_all(&data).unwrap();
        }
    }

    pub fn deserialize_data(&mut self) {
        if std::path::Path::new(&self.file_name()).exists() {
            println!("Deserializing...");
            let file = OpenOptions::new()
                .read(true)
                .open(self.file_name())
                .unwrap();
            self.data = bincode::deserialize_from(file).unwrap();
            fs::remove_file(self.file_name()).unwrap();
        }
    }
}

impl<T: Serialize + DeserializeOwned> Drop for State<T> {
    fn drop(&mut self) {
        self.serialize_data();
    }
}
