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
    fn new(id: usize, data: T) -> Self {
        Self {
            completed: false,
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
    resuming_position: usize,
    valx: i32,
    valy: i32,
}

async fn some_function(mut state: State<MyState>) {
    state.deserialize_data();
    let valx = &mut state.data.valx;
    let valy = &mut state.data.valy;
    println!("start: (valx, valy) = ({}, {})", *valx, *valy);

    loop {
        match state.data.resuming_position {
            0 => {
                let my_other_state = MyOtherState {
                    resuming_position: 0,
                    a: vec![],
                };
                let nested_state = State::new(160182641, my_other_state);
                some_function2(nested_state).await;
                state.data.resuming_position = 1;
            }
            1 => {
                *valx += 1;
                *valy += 2;
                if *valx > 100 && *valy > 200 {
                    println!("completed: (valx, valy) = ({}, {})", *valx, *valy);
                    state.data.resuming_position = 2;
                    continue;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            2 => {
                state.completed = true;
                break;
            }
            _ => {
                panic!("undefined state");
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct MyOtherState {
    resuming_position: usize,
    a: Vec<usize>,
}

async fn some_function2(mut state: State<MyOtherState>) {
    state.deserialize_data();
    let a = &mut state.data.a;
    println!("start: a: {:?}", *a);
    let mut i = a.len();

    loop {
        match state.data.resuming_position {
            0 => {
                if a.len() > 100 {
                    state.data.resuming_position = 1;
                    continue;
                }
                a.push(i);
                i = i + 1;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            1 => {
                state.completed = true;
                break;
            }
            _ => {
                panic!("undefined state");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();

    let my_state = MyState {
        resuming_position: 0,
        valx: 0,
        valy: 0,
    };
    let state = State::new(973298479, my_state);

    let result_fut = tokio::task::spawn(Abortable::new(some_function(state), abort_registration));

    tokio::time::sleep(Duration::from_secs(2)).await;
    abort_handle.abort();
}
