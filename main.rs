// std
use std::collections::hash_set::*;
use std::cmp; //for splitting

mod ppfov;

// size of the map
const MAP_WIDTH: i32 = 20;
const MAP_HEIGHT: i32 = 20;

type Map = Vec<Vec<Tile>>;

/// A tile of the map and its properties
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


// This is a generic entity: the player, a monster, an item, the stairs...
struct Entity {
    x: i32,
    y: i32,
    char: char,
    name: String,
    blocks: bool,
    alive: bool,
    fighter: Option<Fighter>,
    ai: Option<Ai>,
}

impl Entity {
    pub fn new(x: i32, y: i32, char: char, name: &str) -> Self {
        Entity { x, y, char, name: name.into(), blocks: true, alive: true, fighter: None,
            ai: None, }
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

    pub fn attack(&mut self, target: &mut Entity) {
        // should be randomized but we can't use rand crate :(
        let damage = 2;
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
fn move_by(id: usize, dx: i32, dy: i32, map: &Map, entities: &mut [Entity]) {
	let (x,y) = entities[id].pos();
	if !map[(x + dx) as usize][(y + dy) as usize].blocked {
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
	    player.attack(target);
        	}
	    }
	}
	else {
		println!("Attempted move into blocked tile!");
	}
}


fn move_towards(id: usize, target_x: i32, target_y: i32, map: &Map, entities: &mut [Entity]) {
    // vector from this object to the target, and distance
    let dx = target_x - entities[id].x;
    let dy = target_y - entities[id].y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    // normalize it to length 1 (preserving direction), then round it and
    // convert to integer so the movement is restricted to the map grid
    let dx = (dx as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;
    move_by(id, dx, dy, map, entities);
}

//components
// combat-related properties and methods (monster, player, NPC).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Fighter {
    max_hp: i32,
    hp: i32,
    defense: i32,
    attack: i32,
    on_death: DeathCallback,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ai;

fn ai_take_turn(monster_id: usize, map: &Map, entities: &mut [Entity], seen: &HashSet<(i32, i32)>) {
    // a basic monster takes its turn. If you can see it, it can see you
    let (monster_x, monster_y) = entities[monster_id].pos();
    if seen.contains(&(monster_x, monster_y)) {
        if entities[monster_id].distance_to(&entities[0]) >= 2.0 {
            // move towards player if far away
            let (player_x, player_y) = entities[0].pos();
            move_towards(monster_id, player_x, player_y, map, entities);
        } else if entities[0].fighter.map_or(false, |f| f.hp > 0) {
            // close enough, attack! (if the player is still alive.)
            let (monster, player) = mut_two(monster_id, 0, entities);
            monster.attack(player);
        }
    }
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

    map
}

fn print_all(entities: &[Entity], map: &Map, seen: &HashSet<(i32, i32)>) {
    let mut s=String::new();
    
    // go through all tiles, and print
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
	    if seen.contains(&(x,y)) {
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

fn prompt_and_handle_keys(map: &Map, entities: & mut [Entity]) -> PlayerAction {
       use std::io::{stdin,stdout,Write};
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
           move_by(0, -1, 0, map, entities);
	   return TookTurn;
       }
       if s.trim() == "e" {
           move_by(0, 1, 0, map, entities);
	   return TookTurn;
       }
       if s.trim() == "n" {
           move_by(0, 0,-1, map, entities);
	   return TookTurn;
       }
       if s.trim() == "s" {
           move_by(0, 0,1, map, entities);
	   return TookTurn;
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

fn main() {
    let mut game_quit: bool = false;

    let mut player = Entity::new(2,2, '@', "Player");
    player.fighter = Some(Fighter {
        max_hp: 30,
        hp: 30,
        defense: 2,
        attack: 5,
	on_death: DeathCallback::Player,
    });
    let mut npc = Entity::new(6,6, 'k', "kobold");
    npc.fighter = Some(Fighter {
                    max_hp: 10,
                    hp: 10,
                    defense: 0,
                    attack: 3,
		    on_death: DeathCallback::Monster,
                });
    npc.ai = Some(Ai);
    let mut npc2 = Entity::new(7,7, 'k', "kobold");
    npc2.fighter = Some(Fighter {
                    max_hp: 10,
                    hp: 10,
                    defense: 0,
                    attack: 3,
		    on_death: DeathCallback::Monster,
                });
    npc2.ai = Some(Ai);
    let mut entities = [player, npc, npc2];
    let map = make_map();
    let mut seen_set = HashSet::new();
    //init fov
    ppfov::ppfov(
	(2,2),
	5,
	|x, y| if x > 0 && x < 20 && y > 0 && y < 20 { map[x as usize][y as usize].block_sight } else { true },
      	|x, y| {
        	seen_set.insert((x, y));
      	   },
    	);

    while ! game_quit {
       //the order is important, we can't prompt first and draw second because that results in 
	//borrowing twice for some reason
       //render the map
       print_all(&mut entities, &map, &seen_set);
       //super unintuitive but avoids use of moved variable error
       //let player = &mut entities[0];
	
       let player_action = prompt_and_handle_keys(&map, &mut entities);
       //println!("player x {:?}", player.x);
       //println!("player y {:?}", player.y);
        //fov
	//clear set
	seen_set.clear();
	//call function from other file
	ppfov::ppfov(
      	(entities[0].x, entities[0].y),
      	5,
      	|x, y| if x > 0 && x < 20 && y > 0 && y < 20 { map[x as usize][y as usize].block_sight } else { true },
      	|x, y| {
        	seen_set.insert((x, y));
      	   },
    	);
	
	//println!("{:?}", seen_set);
	if player_action == PlayerAction::Exit {
            break;
        }

        // let monsters take their turn
        if entities[0].alive && player_action != PlayerAction::DidntTakeTurn {
            for id in 0..entities.len() {
                if entities[id].ai.is_some() {
                    ai_take_turn(id, &map, &mut entities, &seen_set);
                }
            }
        }
    }

    println!("You quit!");


}
