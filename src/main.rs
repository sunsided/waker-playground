use std::task::{Context, Poll, Waker, RawWaker, RawWakerVTable};
use std::future::Future;
use std::pin::Pin;

/// Custom future for checking whether a number is a prime.
struct PrimeCheckFuture {
    number: u64,
    current: u64,
}

impl PrimeCheckFuture {
    fn new(number: u64) -> Self {
        Self {
            number,
            current: 2,
        }
    }
}

impl Future for PrimeCheckFuture {
    type Output = bool;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Artificially simulate work for checking primality
        if self.current * self.current > self.number {
            return Poll::Ready(true);
        }
        if self.number % self.current == 0 {
            return Poll::Ready(false);
        }

        // Increment current divisor to continue checking
        self.current += 1;
        Poll::Pending
    }
}

// Manual Waker Implementation
fn create_waker() -> Waker {
    fn clone(_: *const ()) -> RawWaker {
        unsafe { create_raw_waker() }
    }

    unsafe fn wake(_: *const ()) {}

    unsafe fn wake_by_ref(_: *const ()) {}

    unsafe fn drop(_: *const ()) {}

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    unsafe fn create_raw_waker() -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }

    let raw_waker = unsafe { create_raw_waker() };
    unsafe { Waker::from_raw(raw_waker) }
}

// Executor to manually poll the future
fn main() {
    let number = 29;

    let future = PrimeCheckFuture::new(number);
    let waker = create_waker();
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(future);

    println!("Checking if {} is a prime number...", number);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(is_prime) => {
                if is_prime {
                    println!("{} is a prime number!", number);
                } else {
                    println!("{} is NOT a prime number!", number);
                }
                break;
            }
            Poll::Pending => {
                println!("Still checking...");
            }
        }
    }
}