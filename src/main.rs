use tcod::colors;
use tcod::console;
use tcod::console::Console;

// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

// 20 frames-per-second maximum
const LIMIT_FPS: i32 = 20;

// Define Tcod struct
struct Tcod {
    root: console::Root,
}

// Define methods
fn handle_keys(tcod: &mut Tcod, player_x: &mut i32, player_y: &mut i32) -> bool {
    // Import modules
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    // Wait for keypress
    let key = tcod.root.wait_for_keypress(true);
    // Determine which key was pressed
    match key {
        // Movement keys
        Key { code: Up, .. } => *player_y -= 1,
        Key { code: Down, .. } => *player_y += 1,
        Key { code: Left, .. } => *player_x -= 1,
        Key { code: Right, .. } => *player_x += 1,
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        }
        Key { code: Escape, .. } => {
            // Exit game
            return true
        },
        _ => {}
    }
    false
}

fn main() {
    // Define tcod implementation
    let root = console::Root::initializer()
        .font("arial10x10.png", console::FontLayout::Tcod)
        .font_type(console::FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Qwestr")
        .init();
    let mut tcod = Tcod { root };

    // Define FPS
    tcod::system::set_fps(LIMIT_FPS);

    // Define player x / y positions
    let mut player_x = SCREEN_WIDTH / 2;
    let mut player_y = SCREEN_HEIGHT / 2;

    // Setup game loop
    while !tcod.root.window_closed() {
        // Set draw colour
        tcod.root.set_default_foreground(colors::WHITE);
        // Clear previous frame
        tcod.root.clear();
        // Draw the @ character
        tcod.root.put_char(player_x, player_y, '@', console::BackgroundFlag::None);
        // Draw everything on the window at once
        tcod.root.flush();
        // Handle keys and exit game if needed
        let exit = handle_keys(&mut tcod, &mut player_x, &mut player_y);
        if exit {
            break;
        }
    }
}

// use qwest_r::play;

// fn main() {
//     // Start the game
//     play();
// }