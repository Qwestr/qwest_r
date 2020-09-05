pub struct Shop {
    pub name: String,
}

impl Shop {
    pub fn list_items(&self) {
        // List shop items
        crate::utils::wait_about_one_second();
        println!("(1) New Sword: 10 gold");
        println!("(2) Health Potion (3 available): 5 gold");
        println!("(3) Cancel");

        // Get user selection
        let selection = crate::utils::get_user_selection();

        // Determine response action
        match selection {
            Some(1) => {
                // Present text
                println!("You bought the New Sword!\n");
            },
            Some(2) => {
                // Present text
                println!("You bought a Health Potion!\n");
            },
            Some(3) => println!("OK!\n"),
            Some(_) => println!("I'm sorry, that's not a valid option.  Please try again.\n"),
            _ => println!("(I'm sorry, I didn't quite understand that.\n"),
        }
    }
}