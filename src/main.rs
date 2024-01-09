use manager::*;

mod accounts;
mod collections;
mod cryptography;
mod manager;
mod utils;

fn main() {
    let mut manager = Manager::new();
    match manager.display_menu() {
        _ => {}
    }
    println!("Good Bye! :)");
}
