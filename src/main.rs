use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tokio::task;

/// Shared state between the future and the executor
struct SharedState {
    number: u64,
    current: u64,
    is_prime: Option<bool>,
    notify: Arc<Notify>,
}

impl SharedState {
    fn new(number: u64, notify: Arc<Notify>) -> Self {
        Self {
            number,
            current: 2,
            is_prime: None,
            notify,
        }
    }
}

/// Asynchronous function for checking if a number is prime
async fn is_prime_number(shared_state: Arc<Mutex<SharedState>>) -> bool {
    loop {
        let mut state = shared_state.lock().await;

        if let Some(is_prime) = state.is_prime {
            return is_prime;
        }

        if state.current * state.current > state.number {
            state.is_prime = Some(true);
            state.notify.notify_one();
            return true;
        }

        if state.number % state.current == 0 {
            state.is_prime = Some(false);
            state.notify.notify_one();
            return false;
        }

        state.current += 1;

        // Drop the lock before yielding to allow other tasks to proceed
        drop(state);

        // Yield to the Tokio scheduler to simulate asynchronous work
        task::yield_now().await;
    }
}

#[tokio::main]
async fn main() {
    let number = 29;
    let notify = Arc::new(Notify::new());
    let shared_state = Arc::new(Mutex::new(SharedState::new(number, notify.clone())));

    // Spawn the prime checking task
    let prime_check_handle = tokio::spawn(is_prime_number(shared_state.clone()));

    // Wait for the result
    notify.notified().await;

    // Retrieve the result
    let state = shared_state.lock().await;
    match state.is_prime {
        Some(true) => println!("{} is a prime number!", state.number),
        Some(false) => println!("{} is not a prime number!", state.number),
        None => println!("Prime check was not completed."),
    }

    // Ensure the spawned task completes
    let _ = prime_check_handle.await;
}
