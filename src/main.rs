use futures::future::{AbortHandle, Abortable};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::time::Duration;

struct Resource {
    id: usize,
    data: HashMap<usize, Vec<u8>>,
}

impl Resource {
    fn file_name(&self) -> String {
        format!("resource-{}", self.id)
    }

    fn new(id: usize) -> Self {
        Self {
            id,
            data: HashMap::new(),
        }
    }

    fn get(&mut self, k: usize) -> Option<Vec<u8>> {
        self.data.remove(&k)
    }

    fn insert(&mut self, k: usize, v: Vec<u8>) {
        self.data.insert(k, v);
    }

    fn is_resumable(&self) -> bool {
        std::path::Path::new(&self.file_name()).exists()
    }

    fn serialize(&mut self) {
        println!("Serializing...");
        dbg!(&self.data);
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(self.file_name())
            .unwrap();
        let data = bincode::serialize(&self.data).unwrap();
        file.write_all(&data).unwrap();
    }

    fn deserialize(&mut self) {
        println!("Deserializing...");
        let file = OpenOptions::new()
            .read(true)
            .open(self.file_name())
            .unwrap();
        self.data = bincode::deserialize_from(file).unwrap();
        dbg!(&self.data);
        fs::remove_file(self.file_name()).unwrap();
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        self.serialize();
    }
}

async fn some_function(mut resource: Resource) {
    let (mut valx, mut valy) = if resource.is_resumable() {
        resource.deserialize();
        let x = bincode::deserialize(&resource.get(1).unwrap()).unwrap();
        let y = bincode::deserialize(&resource.get(2).unwrap()).unwrap();
        println!("deserialized: (valx, valy) = ({}, {})", &x, &y);
        (x, y)
    } else {
        (1, 1)
    };

    loop {
        valx += 1;
        valy += 2;
        tokio::time::sleep(Duration::from_millis(100)).await;
        resource.insert(1, bincode::serialize(&valx).unwrap());
        resource.insert(2, bincode::serialize(&valy).unwrap());
    }
}

#[tokio::main]
async fn main() {
    let resource = Resource::new(479324290734);
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let result_fut =
        tokio::task::spawn(Abortable::new(some_function(resource), abort_registration));

    tokio::time::sleep(Duration::from_secs(2)).await;
    abort_handle.abort();
}
