use std::{thread, time};
use std::io;
use rand::Rng;

pub mod models;
pub mod prompts;

pub fn wait_about_one_second() {
    // Sleep for a random amount of time between 700 and 1700 milliseconds
    thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(700, 1700)));
}

pub fn get_user_input(input: &mut String) {
    // Read line of input
    io::stdin()
        .read_line(input)
        .expect("Something went wrong...");
}

pub fn get_user_selection() -> Option<i32> {
    // Get input
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => match input.trim().parse() {
            Ok(num) => Some(num),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

pub fn roll(min: i32, max: i32) -> i32 {
    // Generate random number between min and max + 1 (exclusive)
    rand::thread_rng().gen_range(min, max + 1)
}