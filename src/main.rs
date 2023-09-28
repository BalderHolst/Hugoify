use std::path::PathBuf;

mod lexer;
mod vault;

use vault::Vault;

fn main() {
    let notes = PathBuf::from("/home/balder/Documents/uni/noter");

    let mut vault = Vault::from_directory(notes).unwrap();
    vault.index();
    vault.output_to(PathBuf::from("output"))
}
