use tcod::colors;
use tcod::console::Console;
use tcod::console;

// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

// 20 frames-per-second maximum
const LIMIT_FPS: i32 = 20;

// Define Tcod struct
struct Tcod {
    root: console::Root,
}

// let root = console::Root::initializer();

// let root = console::Root::initializer()
//     .font("arial10x10.png", FontLayout::Tcod)
//     .font_type(FontType::Greyscale)
//     .size(SCREEN_WIDTH, SCREEN_HEIGHT)
//     .title("Rust/libtcod tutorial")
//     .init();

// let mut tcod = Tcod { root };

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
    
    // Setup game loop
    while !tcod.root.window_closed() {
        tcod.root.set_default_foreground(colors::WHITE);
        tcod.root.clear();
        tcod.root.put_char(1, 1, '@', console::BackgroundFlag::None);
        tcod.root.flush();
        tcod.root.wait_for_keypress(true);
    }
}

// use qwest_r::play;

// fn main() {
//     // Start the game
//     play();
// }