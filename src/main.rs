use std::{path::PathBuf, process::exit};

mod lexer;
mod vault;
mod cli;

use clap::Parser;
use vault::Vault;
use cli::Args;

fn main() {
    let args = Args::parse();
    let input_path = PathBuf::from(args.vault);

    let mut vault = if input_path.is_dir() {
        let mut vault = Vault::from_directory(&input_path).unwrap();
        vault.add_dir(&input_path).unwrap();
        vault
    }
    else if input_path.is_file() {
        let dir = match input_path.parent() {
            Some(d) => d,
            None => {
                eprintln!("Could not find parrent directory of file: '{}'", input_path.display());
                exit(1);
            },
        };
        let mut vault = Vault::from_directory(&dir.to_path_buf()).unwrap();
        vault.add_note(&input_path);
        vault
    }
    else {
        eprintln!("'{}' is not a valid file.", input_path.display());
        exit(1);
    };
    vault.index();
    vault.output_to(PathBuf::from(args.dest))
}
