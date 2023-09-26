use std::fs;

mod lexer;

use lexer::Lexer;

fn main() {
    let note = "/home/balder/Documents/uni/noter/Filters.md";

    let lexer = Lexer::from_file(note).unwrap();

    println!("Hello, world!");
}
