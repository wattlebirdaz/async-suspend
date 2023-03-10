mod state;
use crate::state::State;
use futures::future::{AbortHandle, Abortable};
use serde::{Deserialize, Serialize};
use std::time::Duration;

async fn some_function() {
    #[derive(Serialize, Deserialize)]
    struct MyState {
        resuming_position: usize,
        valx: i32,
        valy: i32,
    }
    let mut state = State::new(
        973298479,
        MyState {
            resuming_position: 0,
            valx: 0,
            valy: 0,
        },
    );
    state.deserialize_data();
    let valx = &mut state.data.valx;
    let valy = &mut state.data.valy;
    println!("start: (valx, valy) = ({}, {})", *valx, *valy);

    loop {
        match state.data.resuming_position {
            0 => {
                some_function2().await;
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

async fn some_function2() {
    #[derive(Serialize, Deserialize)]
    struct MyOtherState {
        resuming_position: usize,
        a: Vec<usize>,
    }
    let mut state = State::new(
        160182641,
        MyOtherState {
            resuming_position: 0,
            a: vec![],
        },
    );
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

    let result_fut = tokio::task::spawn(Abortable::new(some_function(), abort_registration));

    tokio::time::sleep(Duration::from_secs(2)).await;
    abort_handle.abort();
}
