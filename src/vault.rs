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

#[derive(Debug, Clone)]
pub struct Note {
    name: String,
    tokens: Vec<Token>,
    tags: Vec<String>,
    backlinks: Vec<String>,
    links: Vec<String>,
}

#[derive(Debug)]
pub struct Vault {
    // Maps normalized note names to notes
    notes: HashMap<String, Note>,
    attachments: HashSet<String>,
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
            links: Vec::new(),
            tags: Vec::new(),
            backlinks: Vec::new(),
            
        };
        let normalized_name = normalize_name(name.to_string());
        self.notes.insert(normalized_name, note);
    }

    fn add_attachment(&mut self, path: PathBuf) {
        println!("Adding attachment: {:?}", path.display());
        let name = normalize_name(path.file_name().unwrap().to_str().unwrap().to_string());
        self.attachments.insert(name);
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

    pub fn index(&mut self) {
        for (note_name, mut note) in self.notes.clone() {
            for token in note.tokens.clone() {
                match token {
                    Token::Text(_) => {},
                    Token::Header(_, _) => {},
                    Token::Callout(_) => {},
                    Token::Quote(_) => {},
                    Token::Frontmatter(_) => {},
                    Token::Tag(tag) => note.tags.push(tag.to_string()),
                    Token::Link(link) => {

                        // if `dest` field is emply, the link points to itself and we don't have to
                        // do anything in that case.
                        if link.dest.is_empty() {
                            continue;
                        }

                        let to_note_name = normalize_name(link.dest.clone());
                        let to_note = match self.notes.get_mut(&to_note_name) {
                            Some(n) => n,
                            None => {

                                // Is it an attachment?
                                if self.attachments.get(&to_note_name).is_some() {
                                    continue;
                                }

                                eprintln!("WARNING [{}]: Could not find linked note: '{}'", note_name, to_note_name);
                                continue;
                            },
                        };
                        note.links.push(to_note_name.clone());
                        to_note.backlinks.push(note_name.clone());
                        self.notes.insert(note_name.clone(), note.clone());
                    },
                }
            }
        }
    }

}
