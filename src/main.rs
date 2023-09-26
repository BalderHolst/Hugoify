use std::fs;

mod lexer;

use lexer::Lexer;

use crate::lexer::Token;

fn main() {
    let note = "/home/balder/Documents/uni/noter/notes/Filters.md";

    let lexer = Lexer::from_file(note).unwrap();
    for token in lexer {
        println!("{token:?}");
    }
}
