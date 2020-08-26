mod utils;

pub fn play() {
    // Present welcome prompt
    utils::prompts::welcome();

    // Create the player
    let mut player = utils::models::player::Player::new();

    // Present adventure start prompt
    utils::prompts::first_adventure_start();

    // Create the first enemy
    let enemy = utils::models::enemy::Enemy {
        name: String::from("Animated Skeleton"),
        max_health: 13,
        current_health: 13,
        min_damage: 1,
        max_damage: 3,
    };

    // Encounter the enemy
    player.encounter_enemy(enemy);
}