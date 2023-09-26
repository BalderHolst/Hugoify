use std::{fs, io::Write};

mod lexer;

use lexer::Lexer;

use crate::lexer::Token;

fn main() {
    let note = "/home/balder/Documents/uni/noter/notes/Filters.md";

    let lexer = Lexer::from_file(note).unwrap();
    
    let hugo_text = Token::tokens_to_string(lexer);

    fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("output.md")
        .unwrap()
        .write_all(hugo_text.as_bytes())
        .unwrap();

}
