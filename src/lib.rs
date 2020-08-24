use rand::Rng;
use std::io;

// Structs
struct Player {
    name: String,
    max_health: i32,
    current_health: i32,
}

struct Enemy {
    max_health: i32,
    current_health: i32,
}

// Functions
fn get_user_input(input: &mut String) {
    // Read line of input
    io::stdin()
        .read_line(input)
        .expect("I'm sorry, I didn't quite understand that.");
}

fn roll(min: i32, max: i32) -> i32 {
    // Generate random number between min and max + 1 (exclusive)
    rand::thread_rng().gen_range(min, max + 1)
}