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
}

impl Entity {
    pub fn new(x: i32, y: i32, char: char) -> Self {
        Entity { x, y, char }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, map: &Map) {
	if !map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {
		// move by the given amount
		self.x += dx;
		self.y += dy;
	}
	else {
		println!("Attempted move into blocked tile!");
	}
    }

    //dark magic in order to draw entity on top of map
    pub fn draw(&self, map: &mut String) -> String {
	// +1 added to self.y and self.x due to a weird print offset
        let mut lines = map.lines();
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
        let mut result = String::with_capacity(map.len());
        let mut count = 0;
	count_l = 0;
	let mut lines = map.lines();

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
       s = object.draw(&mut s);
    }  
    println!("{}", s);
}

fn prompt_and_handle_keys(player: &mut Entity, map: &Map) -> bool {
       use std::io::{stdin,stdout,Write};

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
           return true; //exit
       }
       if s.trim() == "w" {
           player.move_by(-1, 0, map);
       }
       if s.trim() == "e" {
           player.move_by(1,0, map);
       }
       if s.trim() == "n" {
           player.move_by(0,-1, map);
       }
       if s.trim() == "s" {
           player.move_by(0,1, map);
       }
       //default return
       false
}


fn main() {
    let mut game_quit: bool = false;

    let player = Entity::new(2,2, '@');
    let mut entities = [player];
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
       let player = &mut entities[0];
	
       game_quit = prompt_and_handle_keys(player, &map);
       //println!("player x {:?}", player.x);
       //println!("player y {:?}", player.y);
        //fov
	//clear set
	seen_set.clear();
	//call function from other file
	ppfov::ppfov(
      	(player.x, player.y),
      	5,
      	|x, y| if x > 0 && x < 20 && y > 0 && y < 20 { map[x as usize][y as usize].block_sight } else { true },
      	|x, y| {
        	seen_set.insert((x, y));
      	   },
    	);
	
	//println!("{:?}", seen_set);
	
    }

    println!("You quit!");


}
