pub fn welcome() {
    // Welcome text
    println!("\nWelcome to Qwestr!\n");
    crate::utils::wait_one_second();
}

pub fn first_adventure_start() {
    // Question text
    println!("So, are you ready to start your first adventure?\n");
    loop {
        // Present options
        crate::utils::wait_one_second();
        println!("(1) Yes");
        println!("(2) No");

        // Get user selection
        let selection = crate::utils::get_user_selection();

        // Determine response action
        match selection {
            Some(1) => {
                println!("Awesome!\n");
                crate::utils::wait_one_second();
                break;
            },
            Some(2) => println!("I won't take no for an answer!  Are you ready?\n"),
            Some(_) => println!("I'm sorry, that's not a valid option.  Please try again.\n"),
            None => println!("(I'm sorry, I didn't quite understand that.\n"),
        }
    }
}