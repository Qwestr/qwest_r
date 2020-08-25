pub fn welcome() {
  // Welcome text
  println!("Welcome to Qwestr!");
}

pub fn first_adventure_start() {
  // Question text
  println!("So, are you ready to start your first adventure?");
  loop {
      // Present options
      println!("(1) Yes (2) No");
    
      // Get user selection
      let selection = crate::utils::get_user_selection();

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