use argon2::{self, Config};
use std::env;

fn main() {
    let password = env::args().nth(1).expect("No password provided");
    let salt = b"somesaltvalue"; // In a real app, use a unique, randomly generated salt for each user
    let config = Config::default();
    let hash = argon2::hash_encoded(password.as_bytes(), salt, &config).unwrap();
    println!("{}", hash);
}
