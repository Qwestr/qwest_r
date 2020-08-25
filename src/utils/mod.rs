use std::io;
use rand::Rng;

pub mod prompts;

pub fn get_user_input(input: &mut String) {
    // Read line of input
    io::stdin()
        .read_line(input)
        .expect("I'm sorry, I didn't quite understand that.");
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