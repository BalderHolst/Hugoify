use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs, io,
    path::PathBuf,
};

use crate::lexer::{Lexer, Token};

fn normalize_name(name: String) -> String {
    name.to_lowercase()
}

#[derive(Debug)]
pub struct Note {
    name: String,
    tokens: Vec<Token>,
}

#[derive(Debug)]
pub struct Vault {
    // Maps normalized note names to notes
    notes: HashMap<String, Note>,
    attachments: HashSet<PathBuf>,
}

impl Vault {
    pub fn new() -> Self {
        Self {
            notes: HashMap::new(),
            attachments: HashSet::new(),
        }
    }

    pub fn add_note(&mut self, path: PathBuf) {
        println!("Adding note: {}", path.display());
        let name = path.file_stem().unwrap().to_str().unwrap();
        let tokens = match Lexer::from_file(path.as_path()) {
            Ok(lexer) => lexer.collect(),
            Err(e) => {
                eprintln!("ERROR: {:?}\nSkipping file '{}'.", e, path.display());
                return;
            }
        };
        let note = Note {
            name: name.to_string(),
            tokens,
        };
        let normalized_name = normalize_name(name.to_string());
        self.notes.insert(normalized_name, note);
    }

    fn add_attachment(&mut self, path: PathBuf) {
        println!("Adding attachment: {:?}", path.display());
        self.attachments.insert(path);
    }

    fn add_dir(&mut self, path: PathBuf) -> io::Result<()> {
        match path.file_name().map(|n| n.to_str()) {
            Some(Some(".git")) | Some(Some(".obsidian")) | Some(Some(".trash")) | Some(Some("Excalidraw")) => return Ok(()),
            _ => {}
        }

        println!("Adding directory: {}", path.display());

        for file in fs::read_dir(path)? {
            let file = file?;
            let file_path = file.path();
            if file_path.is_file() {
                match file_path.extension() {
                    Some(ex) => match ex.to_str() {
                        Some("md") => self.add_note(file_path),
                        _ => self.add_attachment(file_path),
                    },
                    None => self.add_attachment(file_path),
                }
            } else if file_path.is_dir() {
                self.add_dir(file_path)?;
            }
        }
        Ok(())
    }

    pub fn from_directory(path: PathBuf) -> io::Result<Self> {
        let mut vault = Self::new();
        vault.add_dir(path)?;
        Ok(vault)
    }
}
