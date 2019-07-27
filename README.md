# roguelikedev-rust-2019
Can a roguelike skeleton be made just with stdin and stdout?

In other words, I attempt to learn Rust by going through the usual roguelike tutorial.

The big caveat is that the computer I am writing it on does not have a proper IDE nor any Rust support (more properly, no cargo support so I can't use any crates outside of standard library). Therefore for editing, I'm limited to a notepad (Vim won't run :( ) and for running the code, I use https://repl.it/languages/rust (Rust Playground does not support stdin for some arcane reason). Code is committed using the GitHub online editor, so beware of typos.

Hopefully in the future (with my own computer) I will be able to switch to libtcod-rs or some other actual graphics library, or even a terminal emulator.

## Week 1
Due to having to rely on stdin and stdout solely, I implemented player control as somewhat MUD-style commands, "w", "e", "s", "n" - type any of those when prompted to see the player position update.

## Week 2
Figured out how to print a map to stdout thanks to my Python implementation. Drawing a player on top of that involved arcane string manipulation that I will be very happy to be rid of once I am no longer so horribly limited (terminal emulator > stdout any day of the week - even when discounting color support, it at least lets you draw glyphs at arbitrary positions)

## Week 3
Cribbed a FOV implementation from Lokathor's tutorial. For obvious reasons (no color support) I don't believe I will be making the "explored" part of the usual unexplored-visible-explored triad.

## Week 4
Since we're printing all of the stuff to stdout, we don't need a dedicated message log structure. And luckily Rust's default font has a Unicode block character built in, so I could approximate the health bar using it.

## Week 5
Inventory basically reuses existing code for input/output, nothing unusual there. I skipped the scrolls/targeting for now, as I have no clue how to represent targeting given the limitations of stdout.

## Week 6
Save/load is a hard skip for now, due to no cargo support we can't use serde for now (that will be rectified once I am at my own dev machine).
