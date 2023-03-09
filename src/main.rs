use futures::future::{AbortHandle, Abortable};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::time::Duration;

struct State<T: Serialize + DeserializeOwned> {
    completed: bool,
    id: usize,
    data: T,
}

impl<T: Serialize + DeserializeOwned> State<T> {
    fn new(completed: bool, id: usize, data: T) -> Self {
        Self {
            completed,
            id,
            data,
        }
    }

    fn file_name(&self) -> String {
        format!("state-{}", self.id)
    }

    fn serialize_data(&self) {
        if !self.completed {
            let data = bincode::serialize(&self.data).unwrap();
            println!("Serializing...");
            // dbg!(&self.data);
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(self.file_name())
                .unwrap();
            file.write_all(&data).unwrap();
        }
    }

    fn deserialize_data(&mut self) {
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

#[derive(Serialize, Deserialize)]
struct MyState {
    valx: i32,
    valy: i32,
}

async fn some_function(mut state: State<MyState>) {
    state.deserialize_data();
    let valx = &mut state.data.valx;
    let valy = &mut state.data.valy;
    println!("start: (valx, valy) = ({}, {})", *valx, *valy);

    loop {
        *valx += 1;
        *valy += 2;
        if *valx > 100 && *valy > 200 {
            println!("completed: (valx, valy) = ({}, {})", *valx, *valy);
            state.completed = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::main]
async fn main() {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();

    let my_state = MyState { valx: 0, valy: 0 };
    let state = State::new(false, 973298479, my_state);

    let result_fut = tokio::task::spawn(Abortable::new(some_function(state), abort_registration));

    tokio::time::sleep(Duration::from_secs(2)).await;
    abort_handle.abort();
}
