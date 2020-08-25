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

fn create_player() -> Player {
    // Present context
    println!("I'm terribly sorry, but I don't seem to know who you are :(");
    println!("Let's start with a name.  What should I call you?");

    // Get name input
    let mut name = String::new();    
    get_user_input(&mut name);

    // Create player
    let player = Player {
        name: String::from(name.trim()),
        max_health: 30,
        current_health: 30,
    };

    // Response text
    println!("Hello {}!  Pleasure to meet you :)", player.name);

    // Return the player
    player
}

fn roll(min: i32, max: i32) -> i32 {
    // Generate random number between min and max + 1 (exclusive)
    rand::thread_rng().gen_range(min, max + 1)
}

pub fn play() {
    // Present welcome text
    present_welcome_text();

    // Create the player
    let mut player = create_player();

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

    // Create enemy
    let mut enemy = Enemy {
        max_health: 20,
        current_health: 20,
    };

    // Encounter text
    println!("Uh oh, you encounter an animated skeleton!  He wants to attack you!");
    println!("What do you want to do?");
    loop {
        // Present options
        println!("(1) Attack (2) Run");
      
        // Get user selection
        let selection = get_user_selection();

        // Determine response action
        match selection {
            Some(1) => {
                // Roll for player attack
                let player_attack_roll = roll(1, 6);

                // Apply attack damage to the enemy
                enemy.current_health -= player_attack_roll;

                // Result text
                println!("You attack with your cool sword for {} damage!", player_attack_roll);
                if enemy.current_health <= 0 {
                    println!("You defeated the skeleton!");
                    break;
                } else {
                    println!("The skeleton has {} of {} health remaining", enemy.current_health, enemy.max_health);
                }

                // Roll for enemy attack
                let enemy_attack_roll = roll(1, 3);

                // Apply attack damage to the player
                player.current_health -= enemy_attack_roll;

                // Result text
                println!("The skeleton attacks you for {} damage!", enemy_attack_roll);
                if player.current_health <= 0 {
                    println!("You were defeated!");
                    break;
                } else {
                    println!("You have {} of {} health remaining", player.current_health, player.max_health);
                }
            },
            Some(2) => println!("You can't leave!"),
            Some(_) => println!("I'm sorry, that's not a valid option.  Please try again."),
            _ => println!("(I'm sorry, I didn't quite understand that."),
        }
    }
}