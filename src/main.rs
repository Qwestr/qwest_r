use tcod::colors;
use tcod::console;
use tcod::console::Console;

// Actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

// Size of the map
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOR_DARK_WALL: colors::Color = colors::Color {
    r: 0,
    g: 0,
    b: 100
};
const COLOR_DARK_GROUND: colors::Color = colors::Color {
    r: 50,
    g: 50,
    b: 150,
};

// 20 frames-per-second maximum
const LIMIT_FPS: i32 = 20;

// This is a generic object: the player, a monster, an item, the stairs...
// It's always represented by a character on screen.
#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: colors::Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: colors::Color) -> Self {
        Object { x, y, char, color }
    }

    // Move by the given amount, if the destination is not blocked
    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {  
        if !game.map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {  
            self.x += dx;  
            self.y += dy;
        }
}

    // Set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, console::BackgroundFlag::None);
    }
}

// A rectangle on the map, used to characterise a room.
#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }
}

// A tile of the map and its properties
#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
        }
    }
}

// Map type (2D array of Tiles)
type Map = Vec<Vec<Tile>>;

// Game struct
struct Game {
    map: Map,
}

// Tcod struct
struct Tcod {
    root: console::Root,
    con: console::Offscreen,
}

// Define methods
fn handle_keys(tcod: &mut Tcod, game: &Game, player: &mut Object) -> bool {
    // Import modules
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    
    // Wait for keypress
    let key = tcod.root.wait_for_keypress(true);
    
    // Determine which key was pressed
    match key {
        // Movement keys
        Key { code: Up, .. } => player.move_by(0, -1, game),
        Key { code: Down, .. } => player.move_by(0, 1, game),
        Key { code: Left, .. } => player.move_by(-1, 0, game),
        Key { code: Right, .. } => player.move_by(1, 0, game),
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

fn make_map() -> Map {
    // Fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    
    // place two pillars to test the map
    map[30][22] = Tile::wall();
    map[50][22] = Tile::wall();

    // Return the map
    map
}

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]) {
    // Draw all objects in the list
    for object in objects {
        object.draw(&mut tcod.con);
    }
    
    // Go through all tiles, and set their background color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].block_sight;
            if wall {
                tcod.con.set_char_background(x, y, COLOR_DARK_WALL, console::BackgroundFlag::Set);
            } else {
                tcod.con.set_char_background(x, y, COLOR_DARK_GROUND, console::BackgroundFlag::Set);
            }
        }
    }
    
    // Blit the contents of "con" to the root console
    console::blit(
        &tcod.con,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
}

fn main() {
    // Define tcod implementation
    let root = console::Root::initializer()
        .font("arial10x10.png", console::FontLayout::Tcod)
        .font_type(console::FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Qwestr")
        .init();
    let con = console::Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
    let mut tcod = Tcod { root, con };
    
    // Define FPS
    tcod::system::set_fps(LIMIT_FPS);

    // Define game
    let game = Game {
        map: make_map(),
    };

    // Create object representing the player
    let player = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', colors::WHITE);

    // Create an NPC
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', colors::YELLOW);

    // Create a list of objects
    let mut objects = [player, npc];

    // Setup game loop
    while !tcod.root.window_closed() {
        // Clear previous frame
        tcod.con.clear();

        // Render the screen
        render_all(&mut tcod, &game, &objects);
        
        // Draw everything on the window at once
        tcod.root.flush();
        
        // Handle keys/ player movement and exit game if needed
        let player = &mut objects[0];
        let exit = handle_keys(&mut tcod, &game, player);
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