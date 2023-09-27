use std::{fs, io::Write, path::PathBuf};

mod lexer;
mod vault;

use lexer::Lexer;
use vault::Vault;

use crate::lexer::Token;

fn main() {
    let notes = PathBuf::from("/home/balder/Documents/uni/noter");

    let mut vault = Vault::from_directory(notes).unwrap();
    vault.index();

    // let lexer = Lexer::from_file(note).unwrap();

    // let hugo_text = Token::tokens_to_string(lexer);

    // fs::OpenOptions::new()
    //     .write(true)
    //     .create(true)
    //     .open("output.md")
    //     .unwrap()
    //     .write_all(hugo_text.as_bytes())
    //     .unwrap();
}
