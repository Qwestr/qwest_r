use rand::distributions::{IndependentSample, Weighted, WeightedChoice};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use tcod::colors::{
    Color,
    BLACK,
    DARK_RED,
    DARKER_GREEN,
    DARKER_RED,
    DESATURATED_GREEN,
    GOLD,
    GREEN,
    LIGHT_BLUE,
    LIGHT_CYAN,
    LIGHT_GREEN,
    LIGHT_GREY,
    LIGHT_RED,
    LIGHT_VIOLET,
    LIGHT_YELLOW,
    ORANGE,
    RED,
    VIOLET,
    WHITE,
    YELLOW,
};
use tcod::console::{
    self,
    BackgroundFlag,
    Console,
    FontLayout,
    FontType,
    Offscreen,
    Root,
    TextAlignment,
};
use tcod::input::{
    self,
    Event,
    Key,
    KeyCode,
    Mouse
};
use tcod::map::{
    FovAlgorithm,
    Map as FovMap,
}; 

// Actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

// Sizes and coordinates relevant for the GUI
const BAR_WIDTH: i32 = 20;
const PANEL_HEIGHT: i32 = 7;
const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;
const INVENTORY_WIDTH: i32 = 50;
const LEVEL_SCREEN_WIDTH: i32 = 40;
const CHARACTER_SCREEN_WIDTH: i32 = 30;

// Message log GUI constants
const MSG_X: i32 = BAR_WIDTH + 2;
const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;

// Size of the map
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 43;

// Room parameters for dungeon generator
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

// Default FOV algorithm and other values
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true; 
const TORCH_RADIUS: i32 = 10;

// Game item constants 
const HEAL_AMOUNT: i32 = 40;
const LIGHTNING_DAMAGE: i32 = 40;
const LIGHTNING_RANGE: i32 = 5;
const CONFUSE_RANGE: i32 = 8;
const CONFUSE_NUM_TURNS: i32 = 10;
const FIREBALL_RADIUS: i32 = 3;
const FIREBALL_DAMAGE: i32 = 25;

// Player will always be the first object
const PLAYER: usize = 0;

// Experience and level-ups
const LEVEL_UP_BASE: i32 = 200;
const LEVEL_UP_FACTOR: i32 = 150;

// Wall/ ground colors
const COLOR_DARK_WALL: Color = Color {
    r: 0,
    g: 0,
    b: 100,
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
#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum AI {
    Basic,
    Confused {
        previous_ai: Box<AI>,
        num_turns: i32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum Item {
    Heal,
    Lightning,
    Confuse,
    Fireball,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum DeathCallback {
    Player,
    Monster,
}

impl DeathCallback {
    fn callback(self, object: &mut Object, game: &mut Game) {
        let callback = match self {
            DeathCallback::Player => player_death,
            DeathCallback::Monster => monster_death,
        };
        callback(object, game);
    }
}

enum UseResult {
    UsedUp,
    Cancelled,
}

// This is a generic object: the player, a monster, an item, the stairs...
// It's always represented by a character on screen.
#[derive(Debug, Serialize, Deserialize)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,  
    blocks: bool,  
    alive: bool,
    name: String,
    always_visible: bool,
    fighter: Option<Fighter>,  
    ai: Option<AI>,
    item: Option<Item>,
    level: i32,
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
            always_visible: false,
            name: name.into(),
            fighter: None,  
            ai: None,
            item: None,
            level: 1,
        }
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }
    
    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    /// Return the distance to another object
    pub fn distance_to(&self, other: &Object) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
    }

    pub fn take_damage(&mut self, damage: i32, game: &mut Game) -> Option<i32> {
        // Apply damage if possible
        if let Some(fighter) = self.fighter.as_mut() {
            if damage > 0 {
                fighter.hp -= damage;
            }
        }

        // Check for death
        if let Some(fighter) = self.fighter {
            if fighter.hp <= 0 {
                self.alive = false;

                // Call the death function
                fighter.on_death.callback(self, game);

                // Return xp for killed fighter
                return Some(fighter.xp);
            }
        }

        // Return None if fighter was not killed
        None
    }

    pub fn attack(&mut self, target: &mut Object, game: &mut Game) {
        // A simple formula for attack damage
        let damage = self.fighter.map_or(0, |f| f.power) - target.fighter.map_or(0, |f| f.defense);
        if damage > 0 {
            // Make the target take some damage
            game.messages.add(
                format!(
                    "{} attacks {} for {} hit points.",
                    self.name, target.name, damage
                ),
                WHITE,
            );
            // Assign damage to target and check if xp is returned for killing target
            if let Some(xp) = target.take_damage(damage, game) {
                // Yield experience to the player
                self.fighter.as_mut().unwrap().xp += xp;
            }
        } else {
            game.messages.add(
                format!(
                    "{} attacks {} but it has no effect!",
                    self.name, target.name
                ),
                WHITE,
            );
        }
    }

    // Heal by the given amount, without going over the maximum
    pub fn heal(&mut self, amount: i32) {
        if let Some(ref mut fighter) = self.fighter {
            fighter.hp += amount;
            if fighter.hp > fighter.max_hp {
                fighter.hp = fighter.max_hp;
            }
        }
    }

    // Return the distance to some coordinates
    pub fn distance(&self, x: i32, y: i32) -> f32 {
        (((x - self.x).pow(2) + (y - self.y).pow(2)) as f32).sqrt()
    }

    // Set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}

// Combat-related properties and methods (monster, player, NPC).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct Fighter {
    max_hp: i32,
    hp: i32,
    defense: i32,
    power: i32,
    xp: i32,
    on_death: DeathCallback,
}

// Console messages
#[derive(Serialize, Deserialize)]
struct Messages {
    messages: Vec<(String, Color)>,
}

impl Messages {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }

    // Add the new message as a tuple, with the text and the color
    pub fn add<T: Into<String>>(&mut self, message: T, color: Color) {
        self.messages.push((message.into(), color));
    }

    // Create a `DoubleEndedIterator` over the messages
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &(String, Color)> {
        self.messages.iter()
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
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
struct Game {
    map: Map,
    messages: Messages,
    inventory: Vec<Object>,
    dungeon_level: u32,
}

struct Transition {
    level: u32,
    value: u32,
}

// Tcod struct
struct Tcod {
    root: Root,
    con: Offscreen,
    panel: Offscreen,
    fov: FovMap,
    key: Key,  
    mouse: Mouse,
}

// Mutably borrow two *separate* elements from the given slice.
fn mut_two<T>(first_index: usize, second_index: usize, items: &mut [T]) -> (&mut T, &mut T) {
    // Panic when the indexes are equal or out of bounds.
    assert!(first_index != second_index);
    let split_at_index = cmp::max(first_index, second_index);
    let (first_slice, second_slice) = items.split_at_mut(split_at_index);
    if first_index < second_index {
        (&mut first_slice[first_index], &mut second_slice[0])
    } else {
        (&mut second_slice[0], &mut first_slice[second_index])
    }
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

fn place_objects(room: Rect, map: &Map, objects: &mut Vec<Object>, level: u32) {
    // Define maximum number of monsters per room based on level
    let max_monsters = from_dungeon_level(
        &[
            Transition { level: 1, value: 2 },
            Transition { level: 4, value: 3 },
            Transition { level: 6, value: 5 },
        ],
        level,
    );
    // Choose random number of monsters
    let num_monsters = rand::thread_rng().gen_range(0, max_monsters + 1);

    for _ in 0..num_monsters {
        // Choose random spot for this monster
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        // Check if the tile is not blocked
        if !is_blocked(x, y, map, objects) {
            // Define chance of creating a troll based on level
            let troll_chance = from_dungeon_level(
                &[
                    Transition { level: 3, value: 15 },
                    Transition { level: 5, value: 30 },
                    Transition { level: 7, value: 60 },
                ],
                level,
            );
            // Create monster generator table
            let monster_chances = &mut [
                Weighted {
                    weight: 80,
                    item: "orc",
                },
                Weighted {
                    weight: troll_chance,
                    item: "troll",
                },
            ];

            // Create monster choice generator
            let monster_choice = WeightedChoice::new(monster_chances);

            // Generate the monster
            let mut monster = match monster_choice.ind_sample(&mut rand::thread_rng()) {
                "orc" => {
                    // Create an orc
                    let mut object = Object::new(x, y, 'o', "orc", DESATURATED_GREEN, true);
                    
                    // Set orc components
                    object.fighter = Some(Fighter {
                        max_hp: 10,
                        hp: 10,
                        defense: 0,
                        power: 3,
                        xp: 35,
                        on_death: DeathCallback::Monster,
                    });
                    object.ai = Some(AI::Basic);
                    
                    // Return the orc
                    object
                }
                "troll" => {
                    // Create a troll
                    let mut object = Object::new(x, y, 'T', "troll", DARKER_GREEN, true);

                    // Set troll components
                    object.fighter = Some(Fighter {
                        max_hp: 16,
                        hp: 16,
                        defense: 1,
                        power: 4,
                        xp: 100,
                        on_death: DeathCallback::Monster,
                    });
                    object.ai = Some(AI::Basic);

                    // Return the troll
                    object
                }
                _ => unreachable!(),
            };

            // Give the monster life!
            monster.alive = true;

            // Add monster to objects list
            objects.push(monster);
        }
    }

    // Define maximum number of items per room based on level
    let max_items = from_dungeon_level(
        &[
            Transition { level: 1, value: 1 },
            Transition { level: 4, value: 2 },
        ],
        level,
    );

    // Choose random number of items
    let num_items = rand::thread_rng().gen_range(0, max_items + 1);

    for _ in 0..num_items {
        // Choose random spot for this item
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        // Only place it if the tile is not blocked
        if !is_blocked(x, y, map, objects) {
            // Create item generator table
            let item_chances = &mut [
                Weighted {
                    weight: 35,
                    item: Item::Heal,
                },
                Weighted {
                    weight: from_dungeon_level(
                        &[Transition { level: 4, value: 25 }],
                        level,
                    ),
                    item: Item::Lightning,
                },
                Weighted {
                    weight: from_dungeon_level(
                        &[Transition { level: 6, value: 25 }],
                        level,
                    ),
                    item: Item::Fireball,
                },
                Weighted {
                    weight: from_dungeon_level(
                        &[Transition { level: 2, value: 10 }],
                        level,
                    ),
                    item: Item::Confuse,
                },
            ];

            // Create item choice generator
            let item_choice = WeightedChoice::new(item_chances);

            // Generate the item
            let mut item = match item_choice.ind_sample(&mut rand::thread_rng()) {
                Item::Heal => {
                    // Create a healing potion
                    let mut object = Object::new(x, y, '!', "healing potion", VIOLET, false);
                    object.item = Some(Item::Heal);

                    // Return the item
                    object
                }
                Item::Lightning => {
                    // Create a lightning bolt scroll
                    let mut object = Object::new(x, y, '#', "scroll of lightning bolt", LIGHT_YELLOW, false);
                    object.item = Some(Item::Lightning);

                    // Return the item
                    object
                }
                Item::Confuse => {
                    // Create a confuse scroll
                    let mut object = Object::new(x, y, '#', "scroll of confusion", LIGHT_YELLOW, false);
                    object.item = Some(Item::Confuse);
                    
                    // Return the object
                    object
                } 
                Item::Fireball => {
                    // Create a fireball scroll
                    let mut object = Object::new(x, y, '#', "scroll of fireball", LIGHT_YELLOW, false);
                    object.item = Some(Item::Fireball);
                    
                    // Return the object
                    object
                }
            };
            
            // Set item to be always visible once found
            item.always_visible = true;

            // Add item to objects list
            objects.push(item);
        }
    }
}

fn make_map(objects: &mut Vec<Object>, level: u32) -> Map {
    // Fill map with "blocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    // Player is the first element, remove everything else.
    // NOTE: works only when the player is the first object!
    assert_eq!(&objects[PLAYER] as *const _, &objects[0] as *const _);
    objects.truncate(1);

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
            place_objects(new_room, &map, objects, level);

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

    // Create stairs at the center of the last room
    let (last_room_x, last_room_y) = rooms[rooms.len() - 1].center();
    let mut stairs = Object::new(last_room_x, last_room_y, '<', "stairs", WHITE, false);
    stairs.always_visible = true;
    objects.push(stairs);

    // Return the map
    map
}

fn is_blocked(x: i32, y: i32, map: &Map, objects: &[Object]) -> bool {
    // First test the map tile
    if map[x as usize][y as usize].blocked {
        return true;
    }
    // Now check for any blocking objects
    objects.iter().any(|object| object.blocks && object.pos() == (x, y))
}

// Move object by the given amount, if the destination is not blocked
fn move_by(id: usize, dx: i32, dy: i32, map: &Map, objects: &mut [Object]) {
    let (x, y) = objects[id].pos();
    if !is_blocked(x + dx, y + dy, map, objects) {
        objects[id].set_pos(x + dx, y + dy);
    }
}

fn move_towards(id: usize, target_x: i32, target_y: i32, map: &Map, objects: &mut [Object]) {
    // Vector from this object to the target, and distance
    let dx = target_x - objects[id].x;
    let dy = target_y - objects[id].y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    // Normalize it to length 1 (preserving direction), then round it and
    // convert to integer so the movement is restricted to the map grid
    let dx = (dx as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;

    // Move object
    move_by(id, dx, dy, map, objects);
}

fn player_move_or_attack(dx: i32, dy: i32, game: &mut Game, objects: &mut [Object]) {
    // The coordinates the player is moving to/attacking
    let x = objects[PLAYER].x + dx;
    let y = objects[PLAYER].y + dy;

    // Try to find an attackable object there
    let target_id = objects.iter().position(|object| object.fighter.is_some() && object.pos() == (x, y));

    // Attack if target found, move otherwise
    match target_id {
        Some(target_id) => {
            // Attack the target
            let (player, target) = mut_two(PLAYER, target_id, objects);
            player.attack(target, game);
        }
        None => {
            // Move the player
            move_by(PLAYER, dx, dy, &game.map, objects);
        }
    }
}

// Find closest enemy, up to a maximum range, and in the player's FOV
fn closest_monster(tcod: &Tcod, objects: &[Object], max_range: i32) -> Option<usize> {
    let mut closest_monster = None;

    // Start with (slightly more than) maximum range
    let mut closest_dist = (max_range + 1) as f32; 

    // Iterate through objects
    for (id, object) in objects.iter().enumerate() {
        // Check if this is a valid monster object
        if (id != PLAYER)
            && object.fighter.is_some()
            && object.ai.is_some()
            && tcod.fov.is_in_fov(object.x, object.y)
        {
            // Calculate distance between this object and the player
            let dist = objects[PLAYER].distance_to(object);
            if dist < closest_dist {
                // It's closer, so remember it
                closest_monster = Some(id);
                closest_dist = dist;
            }
        }
    }

    // Return closest monster
    closest_monster
}

// Add to the player's inventory and remove from the map
fn pick_item_up(object_id: usize, game: &mut Game, objects: &mut Vec<Object>) {
    if game.inventory.len() >= 26 {
        game.messages.add(
            format!(
                "Your inventory is full, cannot pick up {}.",
                objects[object_id].name
            ),
            RED,
        );
    } else {
        let item = objects.swap_remove(object_id);
        game.messages.add(
            format!("You picked up a {}!", item.name),
            GREEN
        );
        game.inventory.push(item);
    }
}

// Drop an item
fn drop_item(inventory_id: usize, game: &mut Game, objects: &mut Vec<Object>) {
    let mut item = game.inventory.remove(inventory_id);
    item.set_pos(objects[PLAYER].x, objects[PLAYER].y);
    game.messages
        .add(format!("You dropped a {}.", item.name), YELLOW);
    objects.push(item);
}

fn ai_take_turn(monster_id: usize, tcod: &Tcod, game: &mut Game, objects: &mut [Object]) {
    // Take AI component of monster
    if let Some(ai) = objects[monster_id].ai.take() {
        // Perform action based on AI variant (return new AI)
        let new_ai = match ai {
            AI::Basic => ai_basic(monster_id, tcod, game, objects),
            AI::Confused {
                previous_ai,
                num_turns,
            } => ai_confused(monster_id, tcod, game, objects, previous_ai, num_turns),
        };

        // Set new AI of monster
        objects[monster_id].ai = Some(new_ai);
    }
}

fn ai_basic(monster_id: usize, tcod: &Tcod, game: &mut Game, objects: &mut [Object]) -> AI {
    // A basic monster takes its turn
    let (monster_x, monster_y) = objects[monster_id].pos();

    // If you can see it, it can see you
    if tcod.fov.is_in_fov(monster_x, monster_y) {
        if objects[monster_id].distance_to(&objects[PLAYER]) >= 2.0 {
            // Move towards player if far away
            let (player_x, player_y) = objects[PLAYER].pos();
            move_towards(monster_id, player_x, player_y, &game.map, objects);
        } else if objects[PLAYER].fighter.map_or(false, |f| f.hp > 0) {
            // Close enough, attack! (if the player is still alive.)
            let (monster, player) = mut_two(monster_id, PLAYER, objects);
            monster.attack(player, game);
        }
    }

    // Return Basic AI variant
    AI::Basic
}

fn ai_confused(
    monster_id: usize,
    _tcod: &Tcod,
    game: &mut Game,
    objects: &mut [Object],
    previous_ai: Box<AI>,
    num_turns: i32,
) -> AI {
    if num_turns >= 0 {
        // Monster still in confused state!
        // Move in a random direction
        move_by(
            monster_id,
            rand::thread_rng().gen_range(-1, 2),
            rand::thread_rng().gen_range(-1, 2),
            &game.map,
            objects,
        );

        // Return a Confused AI with a decreased number of turns
        AI::Confused {
            previous_ai: previous_ai,
            num_turns: num_turns - 1,
        }
    } else {
        // Indicate that the monster is no longer confused
        game.messages.add(
            format!("The {} is no longer confused!", objects[monster_id].name),
            RED,
        );

        // Restore the previous AI (this one will be deleted)
        *previous_ai
    }
}

fn player_death(player: &mut Object, game: &mut Game) {
    // The game ended!
    game.messages.add("You died!", RED);

    // For added effect, transform the player into a corpse!
    player.char = '%';
    player.color = DARK_RED;
}

fn monster_death(monster: &mut Object, game: &mut Game) {
    // Transform it into a nasty corpse!
    // It doesn't block, can't be attacked and doesn't move
    game.messages.add(
        format!(
            "{} is dead! You gain {} experience points.",
            monster.name,
            monster.fighter.unwrap().xp
        ),
        ORANGE,
    );
    monster.char = '%';
    monster.color = DARK_RED;
    monster.blocks = false;
    monster.fighter = None;
    monster.ai = None;
    monster.name = format!("remains of {}", monster.name);
}

fn render_bar(
    panel: &mut Offscreen,
    x: i32,
    y: i32,
    total_width: i32,
    name: &str,
    value: i32,
    maximum: i32,
    bar_color: Color,
    back_color: Color,
) {
    // Render a bar (HP, experience, etc).
    // First calculate the width of the bar
    let bar_width = (value as f32 / maximum as f32 * total_width as f32) as i32;

    // Render the background first
    panel.set_default_background(back_color);
    panel.rect(x, y, total_width, 1, false, BackgroundFlag::Screen);

    // Now render the bar on top
    panel.set_default_background(bar_color);
    if bar_width > 0 {
        panel.rect(x, y, bar_width, 1, false, BackgroundFlag::Screen);
    }

    // Finally, some centered text with the values
    panel.set_default_foreground(WHITE);
    panel.print_ex(
        x + total_width / 2,
        y,
        BackgroundFlag::None,
        TextAlignment::Center,
        &format!("{}: {}/{}", name, value, maximum),
    );
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

    // Get all objects in FOV (and objects that are always visible once explored)
    let mut to_draw: Vec<_> = objects
        .iter()
        .filter(|o| {
            tcod.fov.is_in_fov(o.x, o.y)
            || (o.always_visible && game.map[o.x as usize][o.y as usize].explored)
        })
        .collect();
    
    // Sort so that non-blocking objects come first
    to_draw.sort_by(|o1, o2| { o1.blocks.cmp(&o2.blocks) });
    
    // Draw the objects in the list
    for object in &to_draw {
        object.draw(&mut tcod.con);
    }
    
    // Add the contents of con to the root console
    console::blit(
        &tcod.con,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );

    // Prepare to render the GUI panel
    tcod.panel.set_default_background(BLACK);
    tcod.panel.clear();

    // Render the player's stats (HP, etc.)
    let hp = objects[PLAYER].fighter.map_or(0, |f| f.hp);
    let max_hp = objects[PLAYER].fighter.map_or(0, |f| f.max_hp);
    render_bar(
        &mut tcod.panel,
        1,
        1,
        BAR_WIDTH,
        "HP",
        hp,
        max_hp,
        LIGHT_RED,
        DARKER_RED
    );

    // Render dungeon information (lvl, etc.)
    tcod.panel.print_ex(
        1,
        3,
        BackgroundFlag::None,
        TextAlignment::Left,
        format!("Dungeon level: {}", game.dungeon_level),
    );

    // Render the names of objects under the mouse
    tcod.panel.set_default_foreground(LIGHT_GREY);
    tcod.panel.print_ex(
        1,
        0,
        BackgroundFlag::None,
        TextAlignment::Left,
        get_names_under_mouse(tcod.mouse, objects, &tcod.fov),
    );

    // Render the game messages, one line at a time,
    // from top to bottom, until the end of the screen is hit.
    
    // As new messages are pushed onto the end of the vector,
    // and the messages are being interated in reserve
    let mut y = MSG_HEIGHT as i32;
    for &(ref msg, color) in game.messages.iter().rev() {
        // Get the required message height
        let msg_height = tcod.panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        // Remove from the current message
        y -= msg_height;
        // If the message goes past the bottom of the panel, break the loop
        if y < 0 {
            break;
        }
        // Print the message
        tcod.panel.set_default_foreground(color);
        tcod.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
    }

    // Add the contents of panel to the root console
    console::blit(
        &tcod.panel,
        (0, 0),
        (SCREEN_WIDTH, PANEL_HEIGHT),
        &mut tcod.root,
        (0, PANEL_Y),
        1.0,
        1.0,
    );
}

// Return a string with the names of all objects under the mouse
fn get_names_under_mouse(mouse: Mouse, objects: &[Object], fov_map: &FovMap) -> String {
    let (x, y) = (mouse.cx as i32, mouse.cy as i32);

    // Create a list with the names of all objects at the mouse's coordinates and in FOV
    let names = objects
        .iter()
        .filter(|obj| obj.pos() == (x, y) && fov_map.is_in_fov(obj.x, obj.y))
        .map(|obj| obj.name.clone())
        .collect::<Vec<_>>();

    // Join the names, separated by commas
    names.join(", ") 
}

// Handle key input
fn handle_keys(tcod: &mut Tcod, game: &mut Game, objects: &mut Vec<Object>) -> PlayerAction {    
    // Get status of player
    let player_alive = objects[PLAYER].alive;
    
    // Determine which key was pressed
    match (tcod.key, tcod.key.text(), player_alive) {
        // Movement keys
        (Key { code: KeyCode::Up, .. }, _, true) | (Key { code: KeyCode::NumPad8, .. }, _, true) => {
            player_move_or_attack(0, -1, game, objects);
            return PlayerAction::TookTurn
        }
        (Key { code: KeyCode::Down, .. }, _, true) | (Key { code: KeyCode::NumPad2, .. }, _, true) => {
            player_move_or_attack(0, 1, game, objects);
            return PlayerAction::TookTurn
        }
        (Key { code: KeyCode::Left, .. }, _, true) | (Key { code: KeyCode::NumPad4, .. }, _, true) => {
            player_move_or_attack(-1, 0, game, objects);
            return PlayerAction::TookTurn;
        }
        (Key { code: KeyCode::Right, .. }, _, true) | (Key { code: KeyCode::NumPad6, .. }, _, true) => {
            player_move_or_attack(1, 0, game, objects);
            return PlayerAction::TookTurn;
        }
        (Key { code: KeyCode::Home, .. }, _, true) | (Key { code: KeyCode::NumPad7, .. }, _, true) => {
            player_move_or_attack(-1, -1, game, objects);
            return PlayerAction::TookTurn;
        }
        (Key { code: KeyCode::PageUp, .. }, _, true) | (Key { code: KeyCode::NumPad9, .. }, _, true) => {
            player_move_or_attack(1, -1, game, objects);
            return PlayerAction::TookTurn;
        }
        (Key { code: KeyCode::End, .. }, _, true) | (Key { code: KeyCode::NumPad1, .. }, _, true) => {
            player_move_or_attack(-1, 1, game, objects);
            return PlayerAction::TookTurn;
        }
        (Key { code: KeyCode::PageDown, .. }, _, true) | (Key { code: KeyCode::NumPad3, .. }, _, true) => {
            player_move_or_attack(1, 1, game, objects);
            return PlayerAction::TookTurn;
        }
        (Key { code: KeyCode::NumPad5, .. }, _, true) => {
            // Sleep, i.e. don't moave, wait for the monster(s) to come to you
            return PlayerAction::TookTurn; 
        }
        (Key { code: KeyCode::Text, .. }, "g", true) => {
            // Pick up an item
            let item_id = objects
                .iter()
                .position(|object| object.pos() == objects[PLAYER].pos() && object.item.is_some());
            if let Some(item_id) = item_id {
                pick_item_up(item_id, game, objects);
            }
            return PlayerAction::DidntTakeTurn;
        }
        (Key { code: KeyCode::Text, .. }, "i", true) => {
            // Show the inventory
            let inventory_index = inventory_menu(
                &game.inventory,
                "Press the key next to an item to use it, or any other to cancel.\n",
                &mut tcod.root,
            );
            // If an item is selected, use it
            if let Some(inventory_index) = inventory_index {
                use_item(inventory_index, tcod, game, objects);
            }
            return PlayerAction::TookTurn;
        }
        (Key { code: KeyCode::Text, .. }, "d", true) => {
            // Show the inventory; if an item is selected, drop it
            let inventory_index = inventory_menu(
                &game.inventory,
                "Press the key next to an item to drop it, or any other to cancel.\n'",
                &mut tcod.root,
            );
            if let Some(inventory_index) = inventory_index {
                drop_item(inventory_index, game, objects);
            }
            return PlayerAction::DidntTakeTurn;
        }
        (Key { code: KeyCode::Text, .. }, "<", true) => {
            // Go down stairs, if the player is on them
            let player_on_stairs = objects
                .iter()
                .any(|object| object.pos() == objects[PLAYER].pos() && object.name == "stairs");
            if player_on_stairs {
                next_level(tcod, game, objects);
            }
            return PlayerAction::DidntTakeTurn;
        }
        (Key { code: KeyCode::Text, .. }, "c", true) => {
            // Show character information
            let player = &objects[PLAYER];
            let level = player.level;
            let level_up_xp = LEVEL_UP_BASE + player.level * LEVEL_UP_FACTOR;
            if let Some(fighter) = player.fighter.as_ref() {
                let msg = format!(
                    "Character Information
        
Level: {}
Experience: {}
Experience to level up: {}

Maximum HP: {}
Attack: {}
Defense: {}", level, fighter.xp, level_up_xp, fighter.max_hp, fighter.power, fighter.defense);

                // Show message box
                message_box(&msg, CHARACTER_SCREEN_WIDTH, &mut tcod.root);
            }  
            return PlayerAction::DidntTakeTurn;
        }
        (Key { code: KeyCode::Enter, alt: true, .. }, _, _,) => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
            // Return action
            return PlayerAction::DidntTakeTurn;
        }
        (Key { code: KeyCode::Escape, .. }, _, _) => {
            // Exit game
            return PlayerAction::Exit;
        },
        _ => {
            return PlayerAction::DidntTakeTurn;
        }
    }
}

fn menu<T: AsRef<str>>(header: &str, options: &[T], width: i32, root: &mut Root) -> Option<usize> {
    // Assert that the menu doesn't exceed 26 options
    assert!(
        options.len() <= 26,
        "Cannot have a menu with more than 26 options."
    );

    // Calculate total height for the header (after auto-wrap) and one line per option
    let header_height = if header.is_empty() {
        0
    } else {
        root.get_height_rect(0, 0, width, SCREEN_HEIGHT, header)
    };
    let height = options.len() as i32 + header_height;

    // Create an off-screen console that represents the menu's window
    let mut window = Offscreen::new(width, height);

    // Print the header, with auto-wrap
    window.set_default_foreground(WHITE);
    window.print_rect_ex(
        0,
        0,
        width,
        height,
        BackgroundFlag::None,
        TextAlignment::Left,
        header,
    );

    // Print all the options
    for (index, option_text) in options.iter().enumerate() {
        let menu_letter = (b'a' + index as u8) as char;
        let text = format!("({}) {}", menu_letter, option_text.as_ref());
        window.print_ex(
            0,
            header_height + index as i32,
            BackgroundFlag::None,
            TextAlignment::Left,
            text,
        );
    }

    // blit the contents of "window" to the root console
    let x = SCREEN_WIDTH / 2 - width / 2;
    let y = SCREEN_HEIGHT / 2 - height / 2;
    console::blit(&window, (0, 0), (width, height), root, (x, y), 1.0, 0.7);

    // Present the root console to the player and wait for a key-press
    root.flush();
    let key = root.wait_for_keypress(true);

    // Convert the ASCII code to an index; if it corresponds to an option, return it
    if key.printable.is_alphabetic() {
        let index = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
        if index < options.len() {
            Some(index)
        } else {
            None
        }
    } else {
        None
    }
}

fn message_box(text: &str, width: i32, root: &mut Root) {
    let options: &[&str] = &[];
    menu(text, options, width, root);
}

fn inventory_menu(inventory: &[Object], header: &str, root: &mut Root) -> Option<usize> {
    // Show a menu with each item of the inventory as an option
    let options = if inventory.len() == 0 {
        vec!["Inventory is empty.".into()]
    } else {
        inventory.iter().map(|item| item.name.clone()).collect()
    };

    let inventory_index = menu(header, &options, INVENTORY_WIDTH, root);

    // If an item was chosen, return it
    if inventory.len() > 0 {
        inventory_index
    } else {
        None
    }
}

fn use_item(inventory_id: usize, tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) {
    // Just call the "use_function" if it is defined
    if let Some(item) = game.inventory[inventory_id].item {
        let on_use = match item {
            Item::Heal => cast_heal,
            Item::Lightning => cast_lightning,
            Item::Confuse => cast_confuse,
            Item::Fireball => cast_fireball,
        };
        match on_use(inventory_id, tcod, game, objects) {
            UseResult::UsedUp => {
                // destroy after use, unless it was cancelled for some reason
                game.inventory.remove(inventory_id);
            }
            UseResult::Cancelled => {
                game.messages.add("Cancelled", WHITE);
            }
        }
    } else {
        game.messages.add(
            format!("The {} cannot be used.", game.inventory[inventory_id].name),
            WHITE,
        );
    }
}

// Return the position of a tile left-clicked in player's FOV
// (optionally in a range), or (None,None) if right-clicked.
fn target_tile(
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &[Object],
    max_range: Option<f32>,
) -> Option<(i32, i32)> {
    loop {
        // Check for input event
        let event = input::check_for_event(input::KEY_PRESS | input::MOUSE).map(|e| e.1);
        match event {
            Some(Event::Mouse(m)) => tcod.mouse = m,
            Some(Event::Key(k)) => tcod.key = k,
            None => tcod.key = Default::default(),
        }

        // Render the screen
        render_all(tcod, game, objects, false);

        // Draw everything on the window at once
        // This erases the inventory and shows the names of objects under the mouse.
        tcod.root.flush();

        // Get (x, y) coordinates of the mouse
        let (x, y) = (tcod.mouse.cx as i32, tcod.mouse.cy as i32);

        // Accept the target if the player clicked in FOV,
        let in_fov = (x < MAP_WIDTH) && (y < MAP_HEIGHT) && tcod.fov.is_in_fov(x, y);
        // and in case a range is specified, if it's in that range
        let in_range = max_range.map_or(true, |range| objects[PLAYER].distance(x, y) <= range);
        if tcod.mouse.lbutton_pressed && in_fov && in_range {
            return Some((x, y));
        }

        // Cancel if the player right-clicked or pressed Escape
        if tcod.mouse.rbutton_pressed || tcod.key.code == KeyCode::Escape {
            return None;
        }
    }
}

// Returns a clicked monster inside FOV up to a range, or None if right-clicked
fn target_monster(
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &[Object],
    max_range: Option<f32>,
) -> Option<usize> {
    loop {
        match target_tile(tcod, game, objects, max_range) {
            Some((x, y)) => {
                // Return the first clicked monster, otherwise continue looping
                for (id, obj) in objects.iter().enumerate() {
                    if obj.pos() == (x, y) && obj.fighter.is_some() && id != PLAYER {
                        return Some(id);
                    }
                }
            }
            None => return None
        }
    }
}

fn cast_heal(
    _inventory_id: usize,
    _tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    // Heal the player
    if let Some(fighter) = objects[PLAYER].fighter {
        if fighter.hp == fighter.max_hp {
            game.messages.add("You are already at full health.", RED);
            return UseResult::Cancelled;
        }
        game.messages.add("Your wounds start to feel better!", LIGHT_VIOLET);
        objects[PLAYER].heal(HEAL_AMOUNT);
        return UseResult::UsedUp;
    }
    UseResult::Cancelled
}

fn cast_lightning(
    _inventory_id: usize,
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    // Find closest enemy (inside a maximum range)
    let monster_id = closest_monster(tcod, objects, LIGHTNING_RANGE);
    if let Some(monster_id) = monster_id {
        // Zap it!
        game.messages.add(
            format!(
                "A lightning bolt strikes the {} with a loud thunder! \
                 The damage is {} hit points.",
                objects[monster_id].name, LIGHTNING_DAMAGE
            ),
            LIGHT_BLUE,
        );
        
        // Assign damage to target and check if xp is returned for killing target
        if let Some(xp) = objects[monster_id].take_damage(LIGHTNING_DAMAGE, game) {
            // Yield experience to the player
            objects[PLAYER].fighter.as_mut().unwrap().xp += xp;
        }

        // Return UsedUp result
        UseResult::UsedUp
    } else {
        // No enemy found within maximum range
        game.messages.add("No enemy is close enough to strike.", RED);

        // Return Cancelled result
        UseResult::Cancelled
    }
}

fn cast_confuse(
    _inventory_id: usize,
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    // Ask the player for a target to confuse
    game.messages.add("Left-click an enemy to confuse it, or right-click to cancel.", LIGHT_CYAN);
    let monster_id = target_monster(tcod, game, objects, Some(CONFUSE_RANGE as f32));
    if let Some(monster_id) = monster_id {
        let old_ai = objects[monster_id].ai.take().unwrap_or(AI::Basic);
        // Replace the monster's AI with a "confused" one
        // that will restore the old AI after some turns
        objects[monster_id].ai = Some(AI::Confused {
            previous_ai: Box::new(old_ai),
            num_turns: CONFUSE_NUM_TURNS,
        });
        game.messages.add(
            format!(
                "The eyes of {} look vacant, as he starts to stumble around!",
                objects[monster_id].name
            ),
            LIGHT_GREEN,
        );
        UseResult::UsedUp
    } else {
        // Cancel the action
        game.messages.add("Saving it for later, eh?  Good choice!", WHITE);
        UseResult::Cancelled
    }
}

fn cast_fireball(
    _inventory_id: usize,
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    // Ask the player for a target tile to throw a fireball at
    game.messages.add(
        "Left-click a target tile for the fireball, or right-click to cancel.",
        LIGHT_CYAN,
    );
    let (x, y) = match target_tile(tcod, game, objects, None) {
        Some(tile_pos) => tile_pos,
        None => return UseResult::Cancelled,
    };
    game.messages.add(
        format!(
            "The fireball explodes, burning everything within {} tiles!",
            FIREBALL_RADIUS
        ),
        ORANGE,
    );

    // Create a counter to keep track of xp gained (if any)
    let mut xp_to_gain = 0;
    for (id, obj) in objects.iter_mut().enumerate() {
        if obj.distance(x, y) <= FIREBALL_RADIUS as f32 && obj.fighter.is_some() {
            // Create attack success message
            game.messages.add(
                format!(
                    "The {} gets burned for {} hit points.",
                    obj.name, FIREBALL_DAMAGE
                ),
                ORANGE,
            );

            // Assign damage to target and check if xp is returned for killing target
            if let Some(xp) = obj.take_damage(FIREBALL_DAMAGE, game) {
                // Don't reward the player for burning (and killing) themself!
                if id != PLAYER {                    
                    xp_to_gain += xp;
                }
            }
        }
    }

    // Yield experience to the player
    objects[PLAYER].fighter.as_mut().unwrap().xp += xp_to_gain;

    // Return UsedUp result
    UseResult::UsedUp
}

/// Start a new game
fn new_game() -> (Game, Vec<Object>) {
    // Create the player
    let mut player = Object::new(0, 0, '@', "player", WHITE, true);

    // Set player's fighter component
    player.fighter = Some(Fighter {
        max_hp: 30,
        hp: 30,
        defense: 2,
        power: 5,
        xp: 0,
        on_death: DeathCallback::Player,
    });

    // Give player life!
    player.alive = true;

    // Create a list of objects
    let mut objects = vec![player];

    // Define game
    let mut game = Game {
        map: make_map(&mut objects, 1),
        messages: Messages::new(),
        inventory: vec![],
        dungeon_level: 1,
    };

    // Add a warm welcoming message!
    game.messages.add(
        "Welcome to Qwestr! Prepare to perish in the Tombs of the Fallen Heroes...",
        GOLD,
    );

    // Return game, objects
    (game, objects)
}

/// Initialize FOV
fn initialise_fov(tcod: &mut Tcod, map: &Map) {
    // Create the FOV map, according to the generated map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                !map[x as usize][y as usize].block_sight,
                !map[x as usize][y as usize].blocked,
            );
        }
    }
}

/// Play the game
fn play_game(tcod: &mut Tcod, game: &mut Game, objects: &mut Vec<Object>) {
    // Initialize FOV
    initialise_fov(tcod, &game.map);

    // Keep track of player position
    // Force FOV "recompute" first time through the game loop
    let mut previous_player_position = (-1, -1);

    // Setup game loop
    while !tcod.root.window_closed() {
        // Clear previous frame
        tcod.con.clear();

        // Determine if FOV should be recomputed
        let fov_recompute = previous_player_position != objects[PLAYER].pos();

        // Check for input event
        match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
            Some((_, Event::Mouse(m))) => tcod.mouse = m,
            Some((_, Event::Key(k))) => tcod.key = k,
            _ => tcod.key = Default::default(),
        }

        // Render the screen
        render_all(tcod, game, &objects, fov_recompute);
        
        // Draw everything on the window at once
        tcod.root.flush();

        // Level up if needed
        level_up(tcod, game, objects);
        
        // Get player object
        let player = &mut objects[0];

        // Save current position
        previous_player_position = (player.x, player.y);

        // Get player action
        let player_action = handle_keys(tcod, game, objects);

        // Save & Exit the game if Exit action was taken
        if player_action == PlayerAction::Exit {
            save_game(game, objects).unwrap();
            break;
        }

        // Let monsters take their turn
        if objects[PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
            for id in 0..objects.len() {
                if objects[id].ai.is_some() {
                    ai_take_turn(id, &tcod, game, objects);
                }
            }
        }
    }
}

/// Advance to the next level
fn next_level(tcod: &mut Tcod, game: &mut Game, objects: &mut Vec<Object>) {
    // Show end level message
    game.messages.add(
        "You take a moment to rest, and recover your strength.",
        VIOLET,
    );

    // Heal up to half of the player's max hp
    let heal_hp = objects[PLAYER].fighter.map_or(0, |f| f.max_hp / 2);
    objects[PLAYER].heal(heal_hp);

    // Show next level message
    game.messages.add(
        "After a rare moment of peace, you descend deeper into the heart of the dungeon...",
        RED,
    );

    // Increase dungeon level
    game.dungeon_level += 1;

    // Make new map for level
    game.map = make_map(objects, game.dungeon_level);

    // Initialize FOV
    initialise_fov(tcod, &game.map);
}

/// Promote a character to the next level
fn level_up(tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) {
    // Get player object
    let player = &mut objects[PLAYER];

    // Determine how much xp is required for the next level
    let level_up_xp = LEVEL_UP_BASE + player.level * LEVEL_UP_FACTOR;
    
    // See if the player's xp is enough to level-up
    if player.fighter.as_ref().map_or(0, |f| f.xp) >= level_up_xp {
        // It is! Create Level-Up message
        player.level += 1;
        game.messages.add(
            format!(
                "Your battle skills grow stronger! You reached level {}!",
                player.level
            ),
            YELLOW,
        );
        // Increase players's stats
        let fighter = player.fighter.as_mut().unwrap();
        let mut choice = None;
        while choice.is_none() {
            // Keep asking until a choice is made
            choice = menu(
                "Level up! Choose a stat to raise:\n",
                &[
                    format!("Constitution (+20 HP, from {})", fighter.max_hp),
                    format!("Strength (+1 attack, from {})", fighter.power),
                    format!("Agility (+1 defense, from {})", fighter.defense),
                ],
                LEVEL_SCREEN_WIDTH,
                &mut tcod.root,
            );
        }

        // Remove xp required to level up from the player
        // (resetting to 0 would make the player lose xp over the required amount)
        fighter.xp -= level_up_xp;
        
        // Upgrade the character based on their choice
        match choice.unwrap() {
            0 => {
                // Constitution
                fighter.max_hp += 20;
                fighter.hp += 20;
            }
            1 => {
                // Strength
                fighter.power += 1;
            }
            2 => {
                // Agility
                fighter.defense += 1;
            }
            _ => unreachable!(),
        }
    }
}

/// Returns a value that depends on level. the table specifies what
/// value occurs after each level, default is 0.
fn from_dungeon_level(table: &[Transition], level: u32) -> u32 {
    table
        .iter()
        .rev()
        .find(|transition| level >= transition.level)
        .map_or(0, |transition| transition.value)
}

/// Initialize the main menu of the game
fn main_menu(tcod: &mut Tcod) {
    // Load menu background image
    let img = tcod::image::Image::from_file("menu_background.png") 
        .ok()
        .expect("Background image not found");  

    while !tcod.root.window_closed() {  
        // Show the background image, at twice the regular console resolution
        tcod::image::blit_2x(&img, (0, 0), (-1, -1), &mut tcod.root, (0, 0));

        // Show title/ tag line
        tcod.root.set_default_foreground(LIGHT_YELLOW);
        tcod.root.print_ex(
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT / 2 - 4,
            BackgroundFlag::None,
            TextAlignment::Center,
            "QWESTR",
        );
        tcod.root.print_ex(
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT - 2,
            BackgroundFlag::None,
            TextAlignment::Center,
            "A game about games, life, and everything in-between",
        );

        // Show options and wait for the player's choice
        let choices = &["Play New Game", "Continue Last Game", "Quit"];
        let choice = menu("", choices, 24, &mut tcod.root);

        match choice {  
            Some(0) => {
                // Create a new game
                let (mut game, mut objects) = new_game();

                // Play the game!
                play_game(tcod, &mut game, &mut objects);
            }
            Some(1) => {
                // Load game
                match load_game() {
                    Ok((mut game, mut objects)) => {
                        // Play the game!
                        play_game(tcod, &mut game, &mut objects);
                    }
                    Err(_e) => {
                        // Print error message to message box
                        message_box("\nNo saved game to load.\n", 24, &mut tcod.root);
                        continue;
                    }
                }
            }
            Some(2) => {
                // Quit
                break;
            }
            _ => {}  
        }
    }
}

/// Save the game
fn save_game(game: &Game, objects: &[Object]) -> Result<(), Box<dyn Error>> {  
    // Serialize game/ object data to json
    let save_data = serde_json::to_string(&(game, objects))?;

    // Create a save file  
    let mut file = File::create("savegame")?; 
    
    // Write to the save file
    file.write_all(save_data.as_bytes())?;
    
    // Return successful result
    Ok(())  
}

/// Load the last saved game
fn load_game() -> Result<(Game, Vec<Object>), Box<dyn Error>> {
    // Prepare save state string
    let mut json_save_state = String::new();

    // Open save file
    let mut file = File::open("savegame")?;

    // Read file to save state string
    file.read_to_string(&mut json_save_state)?;

    // Deserialize string to game/ object data
    let result = serde_json::from_str::<(Game, Vec<Object>)>(&json_save_state)?;

    // Return successful result
    Ok(result)
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
        panel: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT),  
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
        key: Default::default(),
        mouse: Default::default(),
    };
    
    // Define FPS
    tcod::system::set_fps(LIMIT_FPS);

    // Show the main menu
    main_menu(&mut tcod);
}

// use qwest_r::play;

// fn main() {
//     // Start the game
//     play();
// }