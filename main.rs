fn main() {
    use std::io::{stdin,stdout,Write};

    let mut game_quit: bool = false;

    let mut player_x = 1;
    let mut player_y = 1;

    while ! game_quit {
       let mut s=String::new();
       print!("Please enter some text: ");
       let _=stdout().flush();
       stdin().read_line(&mut s).expect("Did not enter a correct string");
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
           game_quit = true;
       }
       if s.trim() == "w" {
           player_x -= 1;
       }
       if s.trim() == "e" {
           player_x += 1;
       }
       if s.trim() == "n" {
           player_y -= 1;
       }
       if s.trim() == "s" {
           player_y += 1;
       }
       println!("player x {:?}", player_x);
       println!("player y {:?}", player_y);
    }

    println!("You quit!");


}
