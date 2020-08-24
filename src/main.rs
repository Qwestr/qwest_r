use rand::Rng;
use std::io;

fn get_user_input(input: &mut String) {
    // Read line of input
    io::stdin()
        .read_line(input)
        .expect("I'm sorry, I didn't quite understand that.");
}

fn roll(min: u32, max: u32) -> u32 {
    // Generate random number between min and max + 1 (exclusive)
    rand::thread_rng().gen_range(min, max + 1)
}

fn main() {
    // Welcome text
    println!("Welcome to Qwestr!");
    println!("I'm terribly sorry, but I don't seem to know who you are :(");
    println!("Let's start with a name.  What should I call you?");

    // Enter your name
    let mut name = String::new();    
    get_user_input(&mut name);

    // Response text
    println!("Hello {}!  Pleasure to meet you :)", name.trim());

    // Question text
    println!("So, are you ready to start your first adventure?");
    loop {
        // Present options
        println!("(1) Yes (2) No");
        
        // Get answer
        let mut answer = String::new();
        get_user_input(&mut answer);

        // Cast answer to a number
        let answer: u32 = match answer.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        // Determine response action
        match answer {
            1 => {
                println!("Awesome!");
                break;
            },
            2 => println!("I won't take no for an answer!  Are you ready?"),
            _ => println!("(I'm sorry, I didn't quite understand that."),
        }
    }

    // First encounter
    println!("Uh oh, you encounter an animated skeleton!  He wants to attack you!");
    println!("What do you want to do?");
    loop {
        // Present options
        println!("(1) Attack (2) Run");
        
        // Get answer
        let mut answer = String::new();
        get_user_input(&mut answer);

        // Cast answer to a number
        let answer: u32 = match answer.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        // Determine response action
        match answer {
            1 => {
                // Roll for attack
                let attack_roll = roll(2, 6);
                println!("You attack with your cool sword for {} damage!", attack_roll);

                // Roll for counter attack
                let counter_attack_roll = roll(1, 3);
                println!("The skeleton attacks you for {} damage!", counter_attack_roll);

            },
            2 => {
                println!("Bye!");
                break;
            },
            _ => println!("(I'm sorry, I didn't quite understand that."),
        }
    }
}
