use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

mod lexer;
mod vault;

use lexer::Lexer;
use vault::Vault;

use crate::lexer::Token;

fn main() {
    let notes = PathBuf::from("/home/balder/Documents/uni/noter");

    // let lexer = Lexer::from_file(std::path::Path::new("./test.md")).unwrap();
    // for token in lexer {
    //     println!("{:?}", token);
    // }

    let mut vault = Vault::from_directory(notes).unwrap();
    vault.index();
    vault.output_to(PathBuf::from("output"))

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
