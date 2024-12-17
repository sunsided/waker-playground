use async_mutex::Mutex;
use futures::task::{waker_ref, ArcWake};
use std::sync::{Arc, Mutex as StdMutex};
use std::task::{Context, Poll};
use std::thread;
use std::time::Duration;
use yield_now::yield_now;

pub struct PrimeChecker {
    number: u64,
    current: u64,
    is_prime: Option<bool>,
}

impl PrimeChecker {
    pub fn new(number: u64) -> Self {
        Self {
            number,
            current: 2,
            is_prime: None,
        }
    }

    pub fn poll_prime(&mut self, cx: &mut Context<'_>) -> Poll<bool> {
        if let Some(result) = self.is_prime {
            return Poll::Ready(result);
        }

        if self.current * self.current <= self.number {
            if self.number % self.current == 0 {
                self.is_prime = Some(false);
                return Poll::Ready(false);
            }
            self.current += 1;

            // Simulate asynchronous work by scheduling a wake-up
            let waker = cx.waker().clone();
            let current = self.current;
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(10));
                println!("Checked up to {}", current);
                waker.wake();
            });

            return Poll::Pending;
        }

        self.is_prime = Some(true);
        Poll::Ready(true)
    }
}

struct PrimeWaker {
    shared_state: Arc<StdMutex<SharedState>>,
}

impl ArcWake for PrimeWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let mut shared_state = arc_self.shared_state.lock().unwrap();
        shared_state.woken = true;
    }
}

struct SharedState {
    woken: bool,
}

pub async fn is_prime_number(number: u64) -> bool {
    let prime_checker = Arc::new(Mutex::new(PrimeChecker::new(number)));
    let shared_state = Arc::new(StdMutex::new(SharedState { woken: false }));

    // Create an Arc for PrimeWaker and store it in a variable
    let prime_waker = Arc::new(PrimeWaker {
        shared_state: shared_state.clone(),
    });

    // Obtain a waker reference from the Arc
    let waker = waker_ref(&prime_waker);

    // Create a Context from the waker reference
    let mut cx = Context::from_waker(&waker);

    loop {
        let mut checker = prime_checker.lock().await;
        match checker.poll_prime(&mut cx) {
            Poll::Ready(result) => return result,
            Poll::Pending => {
                drop(checker);
                // Wait until woken
                loop {
                    let mut state = shared_state.lock().unwrap();
                    if state.woken {
                        state.woken = false;
                        break;
                    }
                    // Release the lock before sleeping to avoid deadlock
                    drop(state);
                    yield_now().await;
                }
            }
        }
    }
}
