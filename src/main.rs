mod prime_checker;
use crate::prime_checker::is_prime_number;

#[tokio::main]
async fn main() {
    let number = 29;
    let is_prime = is_prime_number(number).await;
    println!(
        "{} is {}a prime number!",
        number,
        if is_prime { "" } else { "not " }
    );
}
