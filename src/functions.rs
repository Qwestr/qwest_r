use rand::Rng;
use std::io;

mod models;

fn roll(min: i32, max: i32) -> i32 {
    // Generate random number between min and max + 1 (exclusive)
    rand::thread_rng().gen_range(min, max + 1)
}

fn get_user_input(input: &mut String) {
    // Read line of input
    io::stdin()
        .read_line(input)
        .expect("I'm sorry, I didn't quite understand that.");
}

fn get_user_selection() -> Option<i32> {
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

fn present_welcome_text() {
    // Welcome text
    println!("Welcome to Qwestr!");
}

fn present_adventure_start_prompt() {
    // Question text
    println!("So, are you ready to start your first adventure?");
    loop {
        // Present options
        println!("(1) Yes (2) No");
      
        // Get user selection
        let selection = get_user_selection();

        // Determine response action
        match selection {
            Some(1) => {
                println!("Awesome!");
                break;
            },
            Some(2) => println!("I won't take no for an answer!  Are you ready?"),
            Some(_) => println!("I'm sorry, that's not a valid option.  Please try again."),
            None => println!("(I'm sorry, I didn't quite understand that."),
        }
    }
}

fn create_player() -> models::Player {
    // Present context
    println!("I'm terribly sorry, but I don't seem to know who you are :(");
    println!("Let's start with a name.  What should I call you?");

    // Get name input
    let mut name = String::new();    
    get_user_input(&mut name);

    // Create player
    let player = models::Player {
        name: String::from(name.trim()),
        max_health: 30,
        current_health: 30,
    };

    // Response text
    println!("Hello {}!  Pleasure to meet you :)", player.name);

    // Return the player
    player
}

pub fn play() {
    // Present welcome text
    present_welcome_text();

    // Create the player
    let mut player = create_player();

    // Present adventure start prompt
    present_adventure_start_prompt();

    // Encounter enemy
    player.encounter_enemy();
}