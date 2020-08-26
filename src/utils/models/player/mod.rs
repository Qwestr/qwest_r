pub struct Player {
    name: String,
    max_health: i32,
    current_health: i32,
    weapon: crate::utils::models::weapon::Weapon,
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
            weapon: crate::utils::models::weapon::Weapon {
                name: String::from("Training Sword"),
                min_damage: 1,
                max_damage: 4,
            }
        };

        // Response text
        println!("Hello {}!  Pleasure to meet you :)\n", player.name);

        // Return the player
        player
    }
}

impl Player {
    fn attack(&self) -> i32 {
        // Roll for attack
        let attack_roll = crate::utils::roll(
            self.weapon.min_damage,
            self.weapon.max_damage
        );

        // Result text
        println!("You attack with your {} for {} damage!\n", self.weapon.name, attack_roll);

        // Return attack roll
        attack_roll
    }

    fn take_damage(&mut self, damage: i32) {
        // Apply damage
        self.current_health -= damage;
    }

    pub fn encounter_enemy(&mut self, mut enemy: crate::utils::models::enemy::Enemy) {
        // Encounter text
        println!("Uh oh, you've encountered a(n) {}!  It wants to attack you!\n", enemy.name);
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
                    let player_attack_roll = self.attack();

                    // Apply attack to the enemy
                    enemy.take_damage(player_attack_roll);

                    if enemy.current_health <= 0 {
                        println!("You've defeated {}!\n", enemy.name);
                        break;
                    } else {
                        println!("{} has {} of {} health remaining\n", enemy.name, enemy.current_health, enemy.max_health);
                    }

                    // Roll for enemy attack
                    let enemy_attack_roll = enemy.attack();

                    // Apply attack damage to the player
                    self.take_damage(enemy_attack_roll);

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