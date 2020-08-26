pub struct Enemy {
    pub name: String,
    pub max_health: i32,
    pub current_health: i32,
    pub min_damage: i32,
    pub max_damage: i32,
}

impl Enemy {
    pub fn attack(&self) -> i32 {
        // Roll for attack
        let attack_roll = crate::utils::roll(
            self.min_damage,
            self.max_damage
        );

        // Result text
        println!("{} attacks you for {} damage!\n", self.name, attack_roll);
        crate::utils::wait_one_second();

        // Return attack roll
        attack_roll
    }

    pub fn take_damage(&mut self, damage: i32) {
        // Apply damage
        self.current_health -= damage;
    }
}