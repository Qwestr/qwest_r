mod utils;

// Structs
struct Enemy {
    max_health: i32,
    current_health: i32,
}

struct Player {
    name: String,
    max_health: i32,
    current_health: i32,
}

impl Player {
    fn encounter_enemy(&mut self) {
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
            let selection = utils::get_user_selection();
    
            // Determine response action
            match selection {
                Some(1) => {
                    // Roll for player attack
                    let player_attack_roll = utils::roll(1, 6);
    
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
                    let enemy_attack_roll = utils::roll(1, 3);
    
                    // Apply attack damage to the player
                    self.current_health -= enemy_attack_roll;
    
                    // Result text
                    println!("The skeleton attacks you for {} damage!", enemy_attack_roll);
                    if self.current_health <= 0 {
                        println!("You were defeated!");
                        break;
                    } else {
                        println!("You have {} of {} health remaining", self.current_health, self.max_health);
                    }
                },
                Some(2) => println!("You can't leave!"),
                Some(_) => println!("I'm sorry, that's not a valid option.  Please try again."),
                _ => println!("(I'm sorry, I didn't quite understand that."),
            }
        }
    }
}

// Functions
fn create_player() -> Player {
    // Present context
    println!("I'm terribly sorry, but I don't seem to know who you are :(");
    println!("Let's start with a name.  What should I call you?");

    // Get name input
    let mut name = String::new();    
    utils::get_user_input(&mut name);

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

pub fn play() {
    // Present welcome prompt
    utils::prompts::welcome();

    // Create the player
    let mut player = create_player();

    // Present adventure start prompt
    utils::prompts::first_adventure_start();

    // Encounter enemy
    player.encounter_enemy();
}