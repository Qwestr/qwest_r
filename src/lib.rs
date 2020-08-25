mod utils;

pub fn play() {
    // Present welcome prompt
    utils::prompts::welcome();

    // Create the player
    let mut player = utils::models::Player::new();

    // Present adventure start prompt
    utils::prompts::first_adventure_start();

    // Encounter enemy
    player.encounter_enemy();
}