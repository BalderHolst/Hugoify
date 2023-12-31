use std::{path::PathBuf, process::exit};

mod cli;
mod lexer;
mod vault;

use clap::Parser;
use cli::Args;
use vault::Vault;

fn main() {
    let args = Args::parse();
    let input_path = PathBuf::from(args.vault);
    let output_path = PathBuf::from(args.dest);
    let hugo_root_path = args.hugo_root.map(|p| PathBuf::from(p));

    let mut vault = if input_path.is_dir() {
        let mut vault = Vault::from_directory(&input_path, output_path, hugo_root_path).unwrap();
        vault.add_dir(&input_path).unwrap();
        vault
    } else if input_path.is_file() {
        let dir = match input_path.parent() {
            Some(d) => d,
            None => {
                eprintln!(
                    "Could not find parrent directory of file: '{}'",
                    input_path.display()
                );
                exit(1);
            }
        };
        let mut vault =
            Vault::from_directory(&dir.to_path_buf(), output_path, hugo_root_path).unwrap();
        vault.add_note(&input_path);
        vault
    } else {
        eprintln!("'{}' is not a valid file.", input_path.display());
        exit(1);
    };
    vault.index();
    vault.output();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    struct TmpDir(PathBuf);
    impl Drop for TmpDir {
        fn drop(&mut self) {
            if self.0.is_dir() {
                fs::remove_dir_all(&self.0).unwrap();
            }
        }
    }

    // #[test]
    // #[allow(unused_variables)]
    // fn reproducable() {
    //     let n: usize = 10;

    //     let out_dir = PathBuf::from("test_output");
    //     let dir_handle = TmpDir(out_dir.clone());
    //     let input_dir = &PathBuf::from("tests/test_vault");
    //     let mut vault =
    //         Vault::from_directory(input_dir, out_dir, Some(PathBuf::from("."))).unwrap();
    //     vault.add_dir(input_dir).unwrap();

    //     let mut vaults = vec![];

    //     let mut indexed_vault = vault.clone();
    //     indexed_vault.index();

    //     for _ in 0..n {
    //         let mut new_vault = vault.clone();
    //         new_vault.index();
    //         assert_eq!(new_vault, indexed_vault);
    //         vaults.push(new_vault);
    //     }

    //     for (index, note) in indexed_vault.notes() {
    //         let reference_note_string = note.to_string();
    //         for (i, other_vault) in vaults.iter().enumerate() {
    //             let other_note = other_vault.notes().get(index).unwrap();
    //             let other_note_string = other_note.to_string();
    //             println!("{other_note_string}");
    //             assert_eq!(
    //                 reference_note_string, other_note_string,
    //                 "Failed on vault nr: {}",
    //                 i
    //             );
    //         }
    //     }
    // }
}
