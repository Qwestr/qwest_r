use std::cmp;
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

fn create_room(room: Rect, map: &mut Map) {
    // Go through the tiles in the rectangle and make them passable
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    // Horizontal tunnel
    // `min()` and `max()` are used in case `x1 > x2`
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    // Vertical tunnel
    // `min()` and `max()` are used in case `x1 > x2`
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn make_map() -> Map {
    // Fill map with "blocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    // Create two rooms
    let room1 = Rect::new(20, 15, 10, 15);
    let room2 = Rect::new(50, 15, 10, 15);
    create_room(room1, &mut map);
    create_room(room2, &mut map);

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

    // Create the player
    let player = Object::new(25, 23, '@', colors::WHITE);

    // Create an NPC
    let npc = Object::new(55, 23, '@', colors::YELLOW);

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