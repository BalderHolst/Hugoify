use std::path::PathBuf;

mod lexer;
mod vault;
mod cli;

use clap::Parser;
use vault::Vault;
use cli::Args;

fn main() {
    let args = Args::parse();
    let vault_path = PathBuf::from(args.vault);
    let mut vault = Vault::from_directory(vault_path).unwrap();
    vault.index();
    vault.output_to(PathBuf::from(args.dest))
}
