pub mod server;
pub mod message;

pub fn hello_world(from: &str) {
    println!("Hello, world! I'm a {}!", from);
}