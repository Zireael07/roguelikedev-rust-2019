// std
use std::collections::hash_set::*;

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
}

impl Entity {
    pub fn new(x: i32, y: i32, char: char, name: &str) -> Self {
        Entity { x, y, char, name: name.into(), blocks: true, alive: true }
    }

    //shorthand for ease of use
    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }
    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
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

fn move_by(id: usize, dx: i32, dy: i32, map: &Map, entities: &mut [Entity]) {
	let (x,y) = entities[id].pos();
	if !map[(x + dx) as usize][(y + dy) as usize].blocked {
        if !get_entities_at_tile(x + dx, y + dy, entities){
		    // move by the given amount
		   entities[id].set_pos(x + dx, y + dy)
        }
        else {
            //combat!
            println!("Trying to move into npc!");
        }
	}
	else {
		println!("Attempted move into blocked tile!");
	}
}

fn get_entities_at_tile(x: i32, y: i32, entities: &[Entity]) -> bool {
    entities.iter().any(|e| {
        e.blocks && e.pos() == (x, y)
    })
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

    let player = Entity::new(2,2, '@', "Player");
    let npc = Entity::new(6,6, 'k', "kobold");
    let npc2 = Entity::new(7,7, 'k', "kobold");
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
            for e in &entities {
                // only if e is not player
		// pointer comparison, not the usual != value equality
                if (e as *const _) != (&entities[0] as *const _) {
                    println!("The {} growls!", e.name);
                }
            }
        }
    }

    println!("You quit!");


}
