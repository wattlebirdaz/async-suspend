use futures::future::{AbortHandle, Abortable};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::time::Duration;

struct State {
    done: bool,
    id: usize,
    data: HashMap<usize, Vec<u8>>,
}

impl State {
    fn file_name(&self) -> String {
        format!("state-{}", self.id)
    }

    fn new(id: usize) -> Self {
        Self {
            done: false,
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

    fn completed(&mut self) {
        self.done = true;
    }

    fn serialize(&mut self) {
        if !self.done {
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

impl Drop for State {
    fn drop(&mut self) {
        self.serialize();
    }
}

async fn some_function(mut state: State) {
    let (mut valx, mut valy) = if state.is_resumable() {
        state.deserialize();
        let x = bincode::deserialize(&state.get(1).unwrap()).unwrap();
        let y = bincode::deserialize(&state.get(2).unwrap()).unwrap();
        println!("deserialized: (valx, valy) = ({}, {})", &x, &y);
        (x, y)
    } else {
        (1, 1)
    };

    loop {
        valx += 1;
        valy += 2;
        if valx > 100 && valy > 200 {
            println!("completed: (valx, valy) = ({}, {})", &valx, &valy);
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        state.insert(1, bincode::serialize(&valx).unwrap());
        state.insert(2, bincode::serialize(&valy).unwrap());
    }

    state.completed();
}

#[tokio::main]
async fn main() {
    let state = State::new(479324290734);
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let result_fut = tokio::task::spawn(Abortable::new(some_function(state), abort_registration));

    tokio::time::sleep(Duration::from_secs(2)).await;
    abort_handle.abort();
}
