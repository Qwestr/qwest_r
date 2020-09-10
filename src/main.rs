use rand::Rng;
use std::cmp;
use tcod::colors::{
    Color,
    DARKER_GREEN,
    DESATURATED_GREEN,
    WHITE
};
use tcod::console::{
    BackgroundFlag,
    blit,
    Console,
    FontLayout,
    FontType,
    Offscreen,
    Root
};
use tcod::map::{
    FovAlgorithm,
    Map as FovMap
}; 

// Actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

// Size of the map
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

// Room parameters for dungeon generator
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;
const MAX_ROOM_MONSTERS: i32 = 3;

// Default FOV algorithm and other values
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true; 
const TORCH_RADIUS: i32 = 10;

// Wall/ ground colors
const COLOR_DARK_WALL: Color = Color {
    r: 0,
    g: 0,
    b: 100
};
const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};

// 20 frames-per-second maximum
const LIMIT_FPS: i32 = 20;

// Player will always be the first object
const PLAYER: usize = 0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}

// This is a generic object: the player, a monster, an item, the stairs...
// It's always represented by a character on screen.
#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,  
    blocks: bool,  
    alive: bool,
    name: String,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, name: &str, color: Color, blocks: bool) -> Self {
        Object {
            x,
            y,
            char,
            color,
            blocks,
            alive: false,
            name: name.into()
        }
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }
    
    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    // Set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
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

    pub fn center(&self) -> (i32, i32) {
        // Returns tuple containing (x, y) coords of Rect center
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }
    
    pub fn intersects_with(&self, other: &Rect) -> bool {
        // Returns true if this rectangle intersects with another one
        (self.x1 <= other.x2)
            && (self.x2 >= other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }
}

// A tile of the map and its properties
#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    explored: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            explored: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            explored: false,
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
    root: Root,
    con: Offscreen,
    fov: FovMap,
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

fn place_objects(room: Rect, map: &Map, objects: &mut Vec<Object>) {
    // Choose random number of monsters
    let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);

    for _ in 0..num_monsters {
        // Choose random spot for this monster
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        // Check if the tile is not blocked
        if !is_blocked(x, y, map, objects) {
            // Generate the monster
            let mut monster = if rand::random::<f32>() < 0.8 {
                // Create an orc (80% chance)
                Object::new(x, y, 'o', "orc", DESATURATED_GREEN, true)
            } else {
                // Create a troll (20% chance)
                Object::new(x, y, 'T', "troll", DARKER_GREEN, true)
            };

            // Give the monster life!
            monster.alive = true;

            // Add monster to objects list
            objects.push(monster);
        }
    }
}

fn make_map(objects: &mut Vec<Object>) -> Map {
    // Fill map with "blocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    // Create rooms vector
    let mut rooms = vec![];

    // Generate rooms
    for _ in 0..MAX_ROOMS {
        // Generate random width and height for new room
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        
        // Generate random position without going out of the boundaries of the map
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        // Create new room
        let new_room = Rect::new(x, y, w, h);

        // Run through the other rooms and see if they intersect with this one
        let failed = rooms.iter().any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            // This means there are no intersections, so this room is valid
            // "carve" it to the map's wall tiles
            create_room(new_room, &mut map);

            // Add some content to this room, such as monsters
            place_objects(new_room, &map, objects);

            // Center coordinates of the new room, will be useful later
            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                // This is the first room, where the player starts at
                objects[PLAYER].set_pos(new_x, new_y);
            }  else {
                // All rooms after the first:
                // connect it to the previous room with a tunnel
                // center coordinates of the previous room
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
            
                // Toss a coin (random bool value -- either true or false)
                if rand::random() {
                    // First move horizontally, then vertically
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    // First move vertically, then horizontally
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }

            // Finally, append the new room to the list
            rooms.push(new_room);
        }
    }

    // Return the map
    map
}

fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {
    // Recompute FOV if needed (eg. the player moved )
    if fov_recompute {
        let player = &objects[PLAYER];
        tcod.fov.compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }
    
    // Go through all tiles, and set their background color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            // Get visible state of tile
            let visible = tcod.fov.is_in_fov(x, y);
            let wall = game.map[x as usize][y as usize].block_sight;
            let color = match (visible, wall) {
                // Outside of field of view:
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                // Inside fov:
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };

            // Get explored state of tile
            let explored = &mut game.map[x as usize][y as usize].explored;
            // If it's visible, set explored to true
            if visible {
                *explored = true;
            }

            // Show explored tiles only (any visible tile is explored already)
            if *explored {
                tcod.con.set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }
    }

    // Draw all objects in the list (if it's in FOV)
    for object in objects {
        if tcod.fov.is_in_fov(object.x, object.y) {
            object.draw(&mut tcod.con);
        }
    }
    
    // Blit the contents of "con" to the root console
    blit(
        &tcod.con,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
}

fn is_blocked(x: i32, y: i32, map: &Map, objects: &[Object]) -> bool {
    // First test the map tile
    if map[x as usize][y as usize].blocked {
        return true;
    }
    // Now check for any blocking objects
    objects.iter().any(|object| object.blocks && object.pos() == (x, y))
}

//Move object by the given amount, if the destination is not blocked
fn move_by(id: usize, dx: i32, dy: i32, map: &Map, objects: &mut [Object]) {
    let (x, y) = objects[id].pos();
    if !is_blocked(x + dx, y + dy, map, objects) {
        objects[id].set_pos(x + dx, y + dy);
    }
}

// Define methods
fn handle_keys(tcod: &mut Tcod, game: &Game, objects: &mut Vec<Object>) -> PlayerAction {
    // Import modules
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    
    // Wait for keypress
    let key = tcod.root.wait_for_keypress(true);

    // Get status of player
    let player_alive = objects[PLAYER].alive;
    
    // Determine which key was pressed
    match (key, key.text(), player_alive) {
        // Movement keys
        (Key { code: Up, .. }, _, true) => {
            move_by(PLAYER, 0, -1, &game.map, objects);
            return PlayerAction::TookTurn;
        },
        (Key {code: Down, .. }, _, true) => {
            move_by(PLAYER, 0, 1, &game.map, objects);
            return PlayerAction::TookTurn;
        },
        (Key { code: Left, .. }, _, true) => {
            move_by(PLAYER, -1, 0, &game.map, objects);
            return PlayerAction::TookTurn;
        },
        (Key { code: Right, .. }, _, true) => {
            move_by(PLAYER, 1, 0, &game.map, objects);
            return PlayerAction::TookTurn;
        },
       (Key {
            code: Enter,
            alt: true,
            ..
        },
        _,
        _,) => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
            // Return action
            return PlayerAction::DidntTakeTurn;
        }
        (Key { code: Escape, .. }, _, _) => {
            // Exit game
            return PlayerAction::Exit;
        },
        _ => {
            return PlayerAction::DidntTakeTurn;
        }
    }
}

fn main() {
    // Define tcod implementation
    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Qwestr")
        .init();
    let mut tcod = Tcod {
        root,
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),  
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
    };
    
    // Define FPS
    tcod::system::set_fps(LIMIT_FPS);

    // Create the player
    let mut player = Object::new(0, 0, '@', "player", WHITE, true);

    // Give player life!
    player.alive = true;

    // Create a list of objects
    let mut objects = vec![player];

    // Define game
    let mut game = Game {
        map: make_map(&mut objects),
    };

    // Populate the FOV map, according to the generated map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                !game.map[x as usize][y as usize].block_sight,
                !game.map[x as usize][y as usize].blocked,
            );
        }
    }

    // Keep track of player position
    // Force FOV "recompute" first time through the game loop
    let mut previous_player_position = (-1, -1);

    // Setup game loop
    while !tcod.root.window_closed() {
        // Clear previous frame
        tcod.con.clear();

        // Determine if FOV should be recomputed
        let fov_recompute = previous_player_position != objects[PLAYER].pos();

        // Render the screen
        render_all(&mut tcod, &mut game, &objects, fov_recompute);
        
        // Draw everything on the window at once
        tcod.root.flush();
        
        // Get player object
        let player = &mut objects[0];

        // Save current position
        previous_player_position = (player.x, player.y);

        // Get player action
        let player_action = handle_keys(&mut tcod, &game, &mut objects);


        // Exit the game if Exit action was taken
        if player_action == PlayerAction::Exit {
            break;
        }

        // Let monsters take their turn
        if objects[PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
            for object in &objects {
                // only if object is not player
                if (object as *const _) != (&objects[PLAYER] as *const _) {
                    println!("The {} growls!", object.name);
                }
            }
        }
    }
}

// use qwest_r::play;

// fn main() {
//     // Start the game
//     play();
// }