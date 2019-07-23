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

}

fn make_map() -> Map {
    // fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    // place two pillars to test the map
    map[10][12] = Tile::wall();
    map[5][12] = Tile::wall();

    map
}

fn print_all(entities: &[Entity], map: &Map) {
    let mut s=String::new();
    
    // go through all tiles, and print
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = map[x as usize][y as usize].block_sight;
            if wall {
		s.push('#');
                //con.set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
		s.push('.');
                //con.set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
            }
        }
	//our row ended, add a line break
	s.push('\n');
    }

    println!("{}", s);

    // draw all objects in the list
   //for object in entities {
   //    object.draw(&mut s);
   // }  
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

    let player = Entity::new(0,0, '@');
    let mut entities = [player];
    let map = make_map();


    while ! game_quit {
       //the order is important, we can't prompt first and draw second because that results in 
	//borrowing twice for some reason
       //render the map
       print_all(&mut entities, &map);
       //super unintuitive but avoids use of moved variable error
       let player = &mut entities[0];
       game_quit = prompt_and_handle_keys(player, &map);
       println!("player x {:?}", player.x);
       println!("player y {:?}", player.y);
	
    }

    println!("You quit!");


}