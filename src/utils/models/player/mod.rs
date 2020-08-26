pub struct Player {
    name: String,
    max_health: i32,
    current_health: i32,
  }
  
impl Player {
    pub fn new() -> Player {
        // Present context
        println!("I'm terribly sorry, but I don't seem to know who you are :(\n");
        println!("Let's start with a name.  What should I call you?\n");

        // Get name input
        let mut name = String::new();    
        crate::utils::get_user_input(&mut name);

        // Create player
        let player = Player {
            name: String::from(name.trim()),
            max_health: 30,
            current_health: 30,
        };

        // Response text
        println!("Hello {}!  Pleasure to meet you :)\n", player.name);

        // Return the player
        player
    }

    pub fn encounter_enemy(&mut self) {
        // Create enemy
        let mut enemy = crate::utils::models::enemy::Enemy {
            max_health: 20,
            current_health: 20,
        };

        // Encounter text
        println!("Uh oh, you encounter an animated skeleton!  He wants to attack you!\n");
        println!("What do you want to do?\n");
        loop {
            // Present options
            println!("(1) Attack (2) Run");
            
            // Get user selection
            let selection = crate::utils::get_user_selection();

            // Determine response action
            match selection {
                Some(1) => {
                    // Roll for player attack
                    let player_attack_roll = crate::utils::roll(1, 6);

                    // Apply attack damage to the enemy
                    enemy.current_health -= player_attack_roll;

                    // Result text
                    println!("You attack with your cool sword for {} damage!\n", player_attack_roll);
                    if enemy.current_health <= 0 {
                        println!("You defeated the skeleton!\n");
                        break;
                    } else {
                        println!("The skeleton has {} of {} health remaining\n", enemy.current_health, enemy.max_health);
                    }

                    // Roll for enemy attack
                    let enemy_attack_roll = crate::utils::roll(1, 3);

                    // Apply attack damage to the player
                    self.current_health -= enemy_attack_roll;

                    // Result text
                    println!("The skeleton attacks you for {} damage!\n", enemy_attack_roll);
                    if self.current_health <= 0 {
                        println!("You were defeated!\n");
                        break;
                    } else {
                        println!("You have {} of {} health remaining\n", self.current_health, self.max_health);
                    }
                },
                Some(2) => println!("You can't leave!\n"),
                Some(_) => println!("I'm sorry, that's not a valid option.  Please try again.\n"),
                _ => println!("(I'm sorry, I didn't quite understand that.\n"),
            }
        }
    }
}