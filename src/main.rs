use futures::task::{waker, ArcWake};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

/// Shared state between the future and the executor
struct SharedState {
    number: u64,
    current: u64,
    is_prime: Option<bool>,
    waker: Option<Waker>,
}

impl SharedState {
    fn new(number: u64) -> Self {
        Self {
            number,
            current: 2,
            is_prime: None,
            waker: None,
        }
    }
}

/// Custom future for checking if a number is prime
struct PrimeCheckFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

impl PrimeCheckFuture {
    fn new(number: u64) -> Self {
        Self {
            shared_state: Arc::new(Mutex::new(SharedState::new(number))),
        }
    }
}

impl Future for PrimeCheckFuture {
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.shared_state.lock().unwrap();

        if let Some(is_prime) = state.is_prime {
            return Poll::Ready(is_prime);
        }

        if state.current * state.current <= state.number {
            if state.number % state.current == 0 {
                state.is_prime = Some(false);
                return Poll::Ready(false);
            }
            state.current += 1;

            // Store the waker to be used for waking up the task
            state.waker = Some(cx.waker().clone());
            return Poll::Pending;
        }

        state.is_prime = Some(true);
        Poll::Ready(true)
    }
}

/// Executor that polls the future
struct Executor {
    shared_state: Arc<Mutex<SharedState>>,
}

impl Executor {
    fn new(shared_state: Arc<Mutex<SharedState>>) -> Self {
        Self { shared_state }
    }

    fn run(&self) {
        let waker = waker(Arc::new(ExecutorWaker {
            shared_state: self.shared_state.clone(),
        }));
        let mut context = Context::from_waker(&waker);

        let mut future = PrimeCheckFuture {
            shared_state: self.shared_state.clone(),
        };
        let mut future = Pin::new(&mut future);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(is_prime) => {
                    let state = self.shared_state.lock().unwrap();
                    println!(
                        "{} is {}a prime number!",
                        state.number,
                        if is_prime { "" } else { "not " }
                    );
                    break;
                }
                Poll::Pending => {
                    // Simulate asynchronous work by yielding the thread
                    std::thread::yield_now();
                }
            }
        }
    }
}

/// Waker implementation using ArcWake
struct ExecutorWaker {
    shared_state: Arc<Mutex<SharedState>>,
}

impl ArcWake for ExecutorWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let state = arc_self.shared_state.lock().unwrap();
        if let Some(waker) = &state.waker {
            waker.wake_by_ref();
        }
    }
}

fn main() {
    let number = 29;
    let shared_state = Arc::new(Mutex::new(SharedState::new(number)));
    let executor = Executor::new(shared_state.clone());

    // Start the executor
    executor.run();
}
