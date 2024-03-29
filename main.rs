// external crate
use rand::Rng;

#[macro_use]
extern crate serde_derive;

// std
use std::collections::hash_set::*;
use std::cmp; //for splitting

//for save/load
use std::io::{Read, Write};
use std::fs::File;
use std::error::Error;

mod ppfov;

// size of the map
const MAP_WIDTH: i32 = 20;
const MAP_HEIGHT: i32 = 20;

type Map = Vec<Vec<Tile>>;

/// A tile of the map and its properties
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Tile {
    blocked: bool,
    block_sight: bool,
    stairs: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
            stairs: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
            stairs: false,
        }
    }

    pub fn stairs() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
            stairs: true,
        }
    }
}


// This is a generic entity: the player, a monster, an item, the stairs...
#[derive(Serialize, Deserialize, Debug)]
struct Entity {
    x: i32,
    y: i32,
    char: char,
    name: String,
    blocks: bool,
    alive: bool,
    fighter: Option<Fighter>,
    ai: Option<Ai>,
    item: Option<Item>,
    equipment: Option<Equipment>,
}

impl Entity {
    pub fn new(x: i32, y: i32, char: char, name: &str) -> Self {
        Entity { x, y, char, name: name.into(), blocks: true, alive: true, fighter: None,
            ai: None, item: None, equipment: None }
    }

    //shorthand for ease of use
    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }
    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    /// return the distance to another entity
    pub fn distance_to(&self, other: &Entity) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
    }

    pub fn take_damage(&mut self, damage: i32) {
        // apply damage if possible
        if let Some(fighter) = self.fighter.as_mut() {
            if damage > 0 {
                fighter.hp -= damage;
            }
        }
        // check for death, call the death function
        if let Some(fighter) = self.fighter {
            if fighter.hp <= 0 {
                self.alive = false;
		fighter.on_death.callback(self);
            }
        }
    }

    pub fn attack(&mut self, target: &mut Entity, game: &mut Game) {
        let mut damage = self.get_damage(game);
        //random factor
        damage += rand::thread_rng().gen_range(-2,4);
        if damage > 0 {
            // make the target take some damage
            println!(
                "{} attacks {} for {} hit points.",
                self.name, target.name, damage
            );
            target.take_damage(damage);
        } else {
            println!(
                "{} attacks {} but it has no effect!",
                self.name, target.name
            );
        }
    }


    //dark magic in order to draw entity on top of map
    pub fn draw(&self, map: &mut String, seen: &HashSet<(i32, i32)>) -> String {
 	let mut result = String::with_capacity(map.len());
	//do nothing if not in fov
	if !seen.contains(&(self.x,self.y)) {
		let lines = map.lines();
	
		for l in lines {
		     result.push_str(l);
		     //linebreak
		     result.push('\n');
		}
        	return result
		//doesn't work because we made map mutable :(	 
		//return map 
	}

	// +1 added to self.y and self.x due to a weird print offset
        let lines = map.lines();
	let mut count_l = 0;
        let mut map_line=String::new();
	//this gives string slices
        for l in lines {
	     count_l += 1;
	     if count_l == (self.y+1) as usize {
		//println!("Count_l = y {}", self.y);
		map_line = l.to_string();
		break;
	     }
	}
	
	// based on https://stackoverflow.com/questions/26544542/modifying-chars-in-a-string-by-index?noredirect=1&lq=1
        let mut count = 0;
	count_l = 0;
	let lines = map.lines();

	for l in lines {
	    count_l += 1;
	    if count_l != (self.y+1) as usize {
		result.push_str(l);
		//linebreak
		result.push('\n');
	    }
	    else {
		if !map_line.is_empty() {
            	    let mut chars = map_line.chars();

            	    for c in chars {
            		count += 1;
	    		if count == (self.x+1) as usize {
				result.push(self.char);
	    		}
	    		else{
            			result.push(c);
	    		}
           	   }
    		}
		//linebreak
		result.push('\n');
	    }
	}


    //println!("{}", result);

    result
    }

    /// heal by the given amount, without going over the maximum
    pub fn heal(&mut self, amount: i32) {
        if let Some(ref mut fighter) = self.fighter {
            fighter.hp += amount;
            if fighter.hp > fighter.max_hp {
                fighter.hp = fighter.max_hp;
            }
        }
    }

    //alas, no equivalent of Python properties here, we have to do it by hand
    pub fn get_damage(&self, game: &Game) -> i32 {
        let base_damage = self.fighter.map_or(0, |f| f.base_damage);
        let bonus: i32 = self
            .get_all_equipped(game)
            .iter()
            .map(|e| e.damage_bonus)
            .sum();
        base_damage + bonus
    }

    //equipment system
    pub fn equip(&mut self) {
        //paranoia
        if self.item.is_none() {
            print!("Can't equip something which is not an item");
            return;
        };
        if let Some(ref mut equipment) = self.equipment {
            if !equipment.equipped {
                equipment.equipped = true;
                println!("Equipped {} in slot {}.", self.name, equipment.slot);
            }
        } else {
            println!("Can't equip {:?} because it's not an Equipment.", self);
        }

    }

    pub fn take_off(&mut self){
        //paranoia
        if self.item.is_none() {
            print!("Can't take off something which is not an item");
            return;
        };
        if let Some(ref mut equipment) = self.equipment {
            if equipment.equipped {
                equipment.equipped = false;
                println!("Took off {} in slot {}.", self.name, equipment.slot);
            }
        } else {
            println!("Can't take off {:?} because it's not an Equipment.", self);
        }
    }

    /// returns a list of equipped items
    pub fn get_all_equipped(&self, game: &Game) -> Vec<Equipment> {
        if self.name == "player" {
            game.inventory
                .iter()
                .filter(|item| item.equipment.map_or(false, |e| e.equipped))
                .map(|item| item.equipment.unwrap())
                .collect()
        } else {
            vec![] // other entities have no equipment
        }
    }

}

/// Mutably borrow two *separate* elements from the given slice.
/// Panics when the indexes are equal or out of bounds.
fn mut_two<T>(first_index: usize, second_index: usize, items: &mut [T]) -> (&mut T, &mut T) {
    assert!(first_index != second_index);
    let split_at_index = cmp::max(first_index, second_index);
    let (first_slice, second_slice) = items.split_at_mut(split_at_index);
    if first_index < second_index {
        (&mut first_slice[first_index], &mut second_slice[0])
    } else {
        (&mut second_slice[0], &mut first_slice[second_index])
    }
}

//these are global functions, not Entity's because uh, Rust borrow weirdness prevents us from using entity as first parameter and entities as the last...
fn move_by(id: usize, dx: i32, dy: i32, entities: &mut [Entity], game: &mut Game) {
	let (x,y) = entities[id].pos();
	if !game.map[(x + dx) as usize][(y + dy) as usize].blocked {
	    // try to find an attackable entity there
            let target_id = entities
            .iter()
            .position(|e| e.fighter.is_some() && e.pos() == (x + dx, y + dy));

	    match target_id {
		None => {
		    // move by the given amount
		   entities[id].set_pos(x + dx, y + dy)
        	}
        Some(target_id) => {
            //combat!
            //println!("Trying to move into npc!");
	    let (player, target) = mut_two(0, target_id, entities);
	    player.attack(target, game);
        	}
	    }
	}
	else {
		println!("Attempted move into blocked tile!");
	}
}


fn move_towards(id: usize, target_x: i32, target_y: i32, entities: &mut [Entity], game: &mut Game) {
    // vector from this object to the target, and distance
    let dx = target_x - entities[id].x;
    let dy = target_y - entities[id].y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    // normalize it to length 1 (preserving direction), then round it and
    // convert to integer so the movement is restricted to the map grid
    let dx = (dx as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;
    move_by(id, dx, dy, entities, game);
}

//components
// combat-related properties and methods (monster, player, NPC).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct Fighter {
    max_hp: i32,
    hp: i32,
    defense: i32,
    base_damage: i32,
    on_death: DeathCallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum DeathCallback {
    Player,
    Monster,
}

impl DeathCallback {
    fn callback(self, object: &mut Entity) {
        use DeathCallback::*;
        let callback: fn(&mut Entity) = match self {
            Player => player_death,
            Monster => monster_death,
        };
        callback(object);
    }
}

fn player_death(player: &mut Entity) {
    // the game ended!
    println!("You died!");

    // for added effect, transform the player into a corpse!
    player.char = '%';
}

fn monster_death(monster: &mut Entity) {
    // transform it into a nasty corpse! it doesn't block, can't be
    // attacked and doesn't move
    println!("{} is dead!", monster.name);
    monster.char = '%';
    monster.blocks = false;
    monster.fighter = None;
    monster.ai = None;
    monster.name = format!("remains of {}", monster.name);
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum Ai{
    Normal
}

fn ai_take_turn(monster_id: usize, entities: &mut [Entity], seen: &HashSet<(i32, i32)>, game: &mut Game) {
    // a basic monster takes its turn. If you can see it, it can see you
    let (monster_x, monster_y) = entities[monster_id].pos();
    if seen.contains(&(monster_x, monster_y)) {
        if entities[monster_id].distance_to(&entities[0]) >= 2.0 {
            // move towards player if far away
            let (player_x, player_y) = entities[0].pos();
            move_towards(monster_id, player_x, player_y, entities, game);
        } else if entities[0].fighter.map_or(false, |f| f.hp > 0) {
            // close enough, attack! (if the player is still alive.)
            let (monster, player) = mut_two(monster_id, 0, entities);
            monster.attack(player, game);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum Item {
    Heal, //item type for now
    Equipment, //generic that enables wearing/taking off
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
/// An object that can be equipped, yielding bonuses.
struct Equipment {
    slot: Slot,
    equipped: bool,
    damage_bonus: i32, //allows negative values
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum Slot {
    LeftHand,
    RightHand,
    Head,
}

impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Slot::LeftHand => write!(f, "left hand"),
            Slot::RightHand => write!(f, "right hand"),
            Slot::Head => write!(f, "head"),
        }
    }
}

enum UseResult {
    UsedUp,
    UsedAndKept,
    Cancelled,
}

fn use_item(
    inventory_id: usize,
    inventory: &mut Vec<Entity>,
    entities: &mut [Entity],
) {
    use Item::*;
    // just call the "use_function" if it is defined
    if let Some(item) = inventory[inventory_id].item {
        let on_use = match item {
            Heal => cast_heal,
            Equipment => toggle_equipment,
        };
        match on_use(inventory_id, entities, inventory) {
            UseResult::UsedUp => {
                // destroy after use, unless it was cancelled for some reason
                inventory.remove(inventory_id);
            },
            UseResult::UsedAndKept => {}, // do nothing
            UseResult::Cancelled => {
                println!("Cancelled");
            }
        }
    } else {
        println!("The {} cannot be used.", inventory[inventory_id].name);
    }
}

fn cast_heal(_inventory_id: usize, entities: &mut [Entity], inventory: &mut [Entity]) -> UseResult {
    // heal the player
    if let Some(fighter) = entities[0].fighter {
        if fighter.hp == fighter.max_hp {
            println!("You are already at full health.");
            return UseResult::Cancelled;
        }
        println!("Your wounds start to feel better!");
        entities[0].heal(4);
        return UseResult::UsedUp;
    }
    UseResult::Cancelled
}

fn toggle_equipment(
    inventory_id: usize,
    entities: &mut [Entity],
    inventory: &mut [Entity]) -> UseResult {
        let equipment = match inventory[inventory_id].equipment {
        Some(equipment) => equipment,
        None => return UseResult::Cancelled,
    };
    if equipment.equipped {
        //note: if we were using a game.log (as is the case in a usual roguelike),
        // game is already borrowed here (in game.inventory)
        inventory[inventory_id].take_off();
    } else {
        // if the slot is already being used, take off whatever is there first
        if let Some(current) = get_equipped_in_slot(equipment.slot, &inventory) {
            inventory[current].take_off();
        }
        inventory[inventory_id].equip();
    }
    UseResult::UsedAndKept
    }

/// add to the player's inventory and remove from the map
fn pick_item_up(
    object_id: usize,
    entities: &mut Vec<Entity>,
    inventory: &mut Vec<Entity>,
) {
    if inventory.len() >= 26 {
	//println! is effectively equal to format!
        println!("Your inventory is full, cannot pick up {}.",
                entities[object_id].name);
    } else {
        let item = entities.swap_remove(object_id);
        println!("You picked up a {}!", item.name);
        inventory.push(item);
    }
}

fn get_equipped_in_slot(slot: Slot, inventory: &[Entity]) -> Option<usize> {
    for (inventory_id, item) in inventory.iter().enumerate() {
        if item
            .equipment
            .as_ref()
            .map_or(false, |e| e.equipped && e.slot == slot)
        {
            return Some(inventory_id);
        }
    }
    None
}

fn make_map() -> Map {
    // fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    //place walls around
    //Rust is weird, ranges are inclusive at the beginning but exclusive at the end
    for x in 0 ..MAP_WIDTH{
    	map[x as usize][0] = Tile::wall();
	map[x as usize][(MAP_WIDTH-1) as usize] = Tile::wall();
    }
    for y in 0 ..MAP_HEIGHT{
    	map[0][y as usize] = Tile::wall();
	map[(MAP_WIDTH-1) as usize][y as usize] = Tile::wall();
    }

    // place two pillars to test the map
    map[10][12] = Tile::wall();
    map[5][12] = Tile::wall();

    //stairs in random place
    let x = rand::thread_rng().gen_range(1,18);
    let y = rand::thread_rng().gen_range(1,18);

    map[x][y] = Tile::stairs();

    map
}


//GUI
fn menu<T: AsRef<str>>(header: &str, options: &[T]) -> Option<usize> {
    assert!(
        options.len() <= 26,
        "Cannot have a menu with more than 26 options."
    );

    // calculate total height for the header (after auto-wrap) and one line per option
    let header_height = 1;
    let _height = options.len() as i32 + header_height;
    //print header
    println!("{}", header);

    // print all the options
    for (index, option_text) in options.iter().enumerate() {
        let menu_letter = (b'a' + index as u8) as char;
        let text = println!("({}) {}", menu_letter, option_text.as_ref());
    }

    // convert the ASCII code to an index; if it corresponds to an option, return it Option<usize>
       use std::io::{stdin,stdout};
       use PlayerAction::*;

       let mut s=String::new();
       print!("Please enter letter: ");
       let _=stdout().flush();
       stdin().read_line(&mut s).expect("Did not enter a correct letter");
       if let Some('\n')=s.chars().next_back() {
	   s.pop();
       }
       if let Some('\r')=s.chars().next_back() {
	   s.pop();
       }
       println!("You typed: {}",s);
       //lots of dark magic here, thanks Rust for making it difficult to get chars out of a string
       let c = s[0..].chars().next().unwrap();
       let index = c as usize - 'a' as usize;
       //println!("Index is: {}", index);
       if index < options.len() {
            Some(index)
       } else {
            None
       }
}

fn inventory_menu(inventory: &[Entity], header: &str) -> Option<usize> {
    // show a menu with each item of the inventory as an option
    let options = if inventory.len() == 0 {
        vec!["Inventory is empty.".into()]
    } else {
        inventory
        .iter()
        .map(|item| {
            // show additional information, in case it's equipped
            match item.equipment {
                Some(equipment) if equipment.equipped => {
                    format!("{} (on {})", item.name, equipment.slot)
                }
                _ => item.name.clone(),
            }
        })
        .collect()
    };

    //menu(header, &options);

    let inventory_index = menu(header, &options);

    // if an item was chosen, return it
    if inventory.len() > 0 {
	//pretty print
	println!("Inv index: {:?}", inventory_index);
        inventory_index
    } else {
        None
    }
}

fn draw_bar(name: &str, total_width: i32, value: i32, max: i32) -> String {
    let mut s=String::from(name);
    let bar_width = (value as f32 / max as f32 * total_width as f32) as i32;

    for _i in 0..bar_width+1{
	s.push_str("\u{2588}")
    }

    if total_width > bar_width{
	let diff = total_width - bar_width;
	for _i in 0..diff+1 {
	    s.push(' ');
	}
    }

    s.push(' ');
    //deref
    s.push_str(&value.to_string());
    s.push_str("\\");
    s.push_str(&max.to_string());
    return s;
}

fn print_all(entities: &[Entity], map: &Map, seen: &HashSet<(i32, i32)>) {
    let mut s=String::new();
    
    // go through all tiles, and print
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
	    if seen.contains(&(x,y)) {
                let stairs = map[x as usize][y as usize].stairs;
                if stairs {
                    s.push('>');
                }
            	let wall = map[x as usize][y as usize].block_sight;
            	if wall {
		            s.push('#');
                } else {
		            s.push('.');
            	}
	    }
	    else {
		s.push(' ');
	    }
        }
	//our row ended, add a line break
	s.push('\n');
    }

    //println!("{}", s);

    // draw all objects in the list
    for object in entities {
       s = object.draw(&mut s, seen);
    }  
    println!("{}", s);
}

fn prompt_and_handle_keys(game: &mut Game, entities: & mut Vec<Entity>) -> PlayerAction {
       use std::io::{stdin,stdout};
       use PlayerAction::*;

       let mut s=String::new();
       print!("Please enter command: ");
       let _=stdout().flush();
       stdin().read_line(&mut s).expect("Did not enter a correct command");
       if let Some('\n')=s.chars().next_back() {
	   s.pop();
       }
       if let Some('\r')=s.chars().next_back() {
	   s.pop();
       }
       println!("You typed: {}",s);

       //key handling
       if s.trim() == "Q" {
           println!("Quit!");
           return Exit; //exit
       }
       if s.trim() == "w" {
           move_by(0, -1, 0, entities, game);
	        return TookTurn;
       }
       if s.trim() == "e" {
           move_by(0, 1, 0, entities, game);
	   return TookTurn;
       }
       if s.trim() == "n" {
           move_by(0, 0, -1, entities, game);
	   return TookTurn;
       }
       if s.trim() == "s" {
           move_by(0, 0, 1,entities, game);
	   return TookTurn;
       }
       if s.trim() == ">" || s.trim() == "<" {
           //tuple unpacking
           let (x, y) = entities[0].pos();
           if game.map[x as usize][y as usize].stairs {
               //new level
               next_level(entities, game);
               return DidntTakeTurn;
           }
           else
           {
               return DidntTakeTurn;
           }
       }
       if s.trim() == "g" {
	   // pick up an item
    	   let item_id = entities
           .iter()
           .position(|e| e.pos() == entities[0].pos() && e.item.is_some());
           if let Some(item_id) = item_id {
        	pick_item_up(item_id, entities, &mut game.inventory);
		return TookTurn;
           }
           //return DidntTakeTurn;
       }
       if s.trim() == "i" {
    	   // show the inventory
           let inventory_index = inventory_menu(
           &game.inventory,
           "Press the key next to an item to use it, or any other to cancel.\n");
           if let Some(inventory_index) = inventory_index {
           	use_item(inventory_index, &mut game.inventory, entities);
           }
           return DidntTakeTurn;
       }
       //default return
       DidntTakeTurn
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}

#[derive(Serialize, Deserialize)]
struct Game {
    map: Map,
    inventory: Vec<Entity>,
    dungeon_level: u32,
}

fn next_level(entities: &mut Vec<Entity>, game: &mut Game){
    print!("You descend deeper in the dungeons...");
    game.dungeon_level += 1;
    //make the new level
    game.map = make_map();
    
    // Player is the first element, remove everything else
    entities.truncate(1);

    //create NPCs
    let x = rand::thread_rng().gen_range(1,18);
    let y = rand::thread_rng().gen_range(1,18);
    let mut npc = Entity::new(x,y, 'k', "kobold");
    npc.fighter = Some(Fighter {
                    max_hp: 10,
                    hp: 10,
                    defense: 0,
                    base_damage: 4,
		    on_death: DeathCallback::Monster,
                });
    npc.ai = Some(Ai::Normal);
    let x = rand::thread_rng().gen_range(1,18);
    let y = rand::thread_rng().gen_range(1,18);
    let mut npc2 = Entity::new(x,y, 'k', "kobold");
    npc2.fighter = Some(Fighter {
                    max_hp: 10,
                    hp: 10,
                    defense: 0,
                    base_damage: 4,
		    on_death: DeathCallback::Monster,
                });
    npc2.ai = Some(Ai::Normal);


}


fn main_menu() -> Option<(Vec<Entity>, Game)>{
    use std::io::{stdin,stdout};

    let mut s=String::new();

    print!("1) New game");
    print!("2) Load game");

    print!("Please enter command: ");
    let _=stdout().flush();
    stdin().read_line(&mut s).expect("Did not enter a correct command");
    if let Some('\n')=s.chars().next_back() {
	   s.pop();
    }
    if let Some('\r')=s.chars().next_back() {
	   s.pop();
    }
    println!("You typed: {}",s);

    //key handling
    if s.trim() == "1" {
        print!("New game!");
        let (mut entities, mut game) = new_game();
        return Some((entities, game));
    }
    if s.trim() == "2" {
        print!("Load game!");
        //load game
        let (mut entities, mut game) = load_game().unwrap();
        return Some((entities, game));
    }
    //default
    None
}


fn new_game() -> (Vec<Entity>, Game) {
    //create player
    let mut player = Entity::new(1,1, '@', "Player");
    player.fighter = Some(Fighter {
        max_hp: 30,
        hp: 30,
        defense: 2,
        base_damage: 5,
	on_death: DeathCallback::Player,
    });

    //create NPCs
    let x = rand::thread_rng().gen_range(1,18);
    let y = rand::thread_rng().gen_range(1,18);
    let mut npc = Entity::new(x,y, 'k', "kobold");
    npc.fighter = Some(Fighter {
                    max_hp: 10,
                    hp: 10,
                    defense: 0,
                    base_damage: 3,
		    on_death: DeathCallback::Monster,
                });
    npc.ai = Some(Ai::Normal);
    let x = rand::thread_rng().gen_range(1,18);
    let y = rand::thread_rng().gen_range(1,18);
    let mut npc2 = Entity::new(x,y, 'k', "kobold");
    npc2.fighter = Some(Fighter {
                    max_hp: 10,
                    hp: 10,
                    defense: 0,
                    base_damage: 3,
		    on_death: DeathCallback::Monster,
                });
    npc2.ai = Some(Ai::Normal);
    let mut object = Entity::new(2, 5, '!', "healing potion");
    object.item = Some(Item::Heal);

    // create a sword
    let mut sword = Entity::new(2, 2, '/', "sword");
    sword.item = Some(Item::Equipment);
    sword.equipment = Some(Equipment{equipped: false, slot: Slot::RightHand, damage_bonus:1});

    let mut entities = vec![player, npc, npc2, object, sword];

    let mut game = Game { 
        map: make_map(),
        inventory: vec![],
        dungeon_level: 1,
        };

    (entities, game)
}

fn play_game(entities: &mut Vec<Entity>, game: &mut Game, seen_set: &mut HashSet<(i32, i32)>, game_quit: bool) {
    while ! game_quit {
       //the order is important, we can't prompt first and draw second because that results in 
	//borrowing twice for some reason
       //render the map
       print_all(&entities, &game.map, &seen_set);
	    // draw basic infos
        let hp = entities[0].fighter.map_or(0, |f| f.hp);
        let max_hp = entities[0].fighter.map_or(0, |f| f.max_hp);
	    println!("{}", draw_bar("HP: ", 4, hp, max_hp));
        println!("Dungeon level: {}", game.dungeon_level);
       //super unintuitive but avoids use of moved variable error
       //let player = &mut entities[0];
	
       let player_action = prompt_and_handle_keys(game, entities);
       //println!("player x {:?}", player.x);
       //println!("player y {:?}", player.y);
       //println!("\u{2588}");

        //fov
	    //clear set
	    seen_set.clear();
	    //call function from other file
	    ppfov::ppfov(
      	(entities[0].x, entities[0].y),
      	5,
      	|x, y| if x > 0 && x < 20 && y > 0 && y < 20 { game.map[x as usize][y as usize].block_sight } else { true },
      	|x, y| {
        	seen_set.insert((x, y));
      	   },
    	);
	
	    //println!("{:?}", seen_set);
	    if player_action == PlayerAction::Exit {
            //save game when quitting
            save_game(entities, game);
            break;
        }

        // let monsters take their turn
        if entities[0].alive && player_action != PlayerAction::DidntTakeTurn {
            for id in 0..entities.len() {
                if entities[id].ai.is_some() {
                    //println!("Taking turn...");
                    ai_take_turn(id, entities, &seen_set, game);
                }
            }
        }
    }

    println!("You quit!");
}

//save/load
fn save_game(entities: &[Entity], game: &Game) -> Result<(), Box<Error>> {
    let save_data = serde_json::to_string(&(entities, game))?;
    let mut file = File::create("savegame")?;
    file.write_all(save_data.as_bytes())?;
    Ok(())
}

fn load_game() -> Result<(Vec<Entity>, Game), Box<Error>> {
    let mut json_save_state = String::new();
    let mut file = File::open("savegame")?;
    file.read_to_string(&mut json_save_state)?;
    let result = serde_json::from_str::<(Vec<Entity>, Game)>(&json_save_state)?;
    Ok(result)
}

fn main() {
    let mut game_quit: bool = false;

    let data = main_menu();
    match data {
        None => {
            //quit because we went wrong
            game_quit = true;
        }
        Some(data) => {
            //unpack tuple
            let (mut entities, mut game) = data;
            let mut seen_set = HashSet::new();
            //init fov
            ppfov::ppfov(
	        (2,2),
	        5,
	        |x, y| if x > 0 && x < 20 && y > 0 && y < 20 {game.map[x as usize][y as usize].block_sight } else { true },
      	    |x, y| {
        	    seen_set.insert((x, y));
      	    },
    	    );

            play_game(&mut entities, &mut game, &mut seen_set, game_quit);
        }
    }

    //unpack a tuple
    //let (mut entities, mut game) = new_game();



}
