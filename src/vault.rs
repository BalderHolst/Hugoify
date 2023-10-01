use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::PathBuf,
};

use yaml_rust::Yaml;

use crate::lexer::{Lexer, Token};

fn normalize_path_to_string(path: &PathBuf) -> String {
    path.to_str().unwrap().split('/').map(|s| normalize_string(s.to_string())).collect::<Vec<String>>().join("/").to_string()
}

fn normalize_path(path: &PathBuf) -> PathBuf {
    PathBuf::from(normalize_path_to_string(path))
}

fn normalize_string(mut name: String) -> String {
    if name.contains('/') {
        eprint!("WARNING: Normalizing name with '/': `{name}`. Only using filename.");
        name = name.split_once('/').unwrap().1.to_string();
    }

    name.chars()
        .map(|c| match c {
            ' ' => '-',
            _ => c.to_lowercase().next().unwrap(),
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    /// Relative path within vault
    path: PathBuf,
    name: String,
    tokens: Vec<Token>,
    frontmatter: Yaml,
    tags: Vec<String>,
    backlinks: Vec<String>,
    links: Vec<String>,
}

impl ToString for Note {
    fn to_string(&self) -> String {
        // This could probably be done better...
        let frontmatter = match self.frontmatter.clone() {
            Yaml::Hash(mut hash) => {
                // Title
                hash.insert(
                    Yaml::String("title".to_string()),
                    Yaml::String(self.name.clone()),
                );

                // Tags
                hash.insert(
                    Yaml::String("tags".to_string()),
                    Yaml::Array(self.tags.iter().map(|t| Yaml::String(t.clone())).collect()),
                );

                // Backlinks
                hash.insert(
                    Yaml::String("backlinks".to_string()),
                    Yaml::Array(
                        self.backlinks
                            .iter()
                            .map(|t| Yaml::String(t.clone()))
                            .collect(),
                    ),
                );

                // Links
                hash.insert(
                    Yaml::String("links".to_string()),
                    Yaml::Array(self.links.iter().map(|t| Yaml::String(t.clone())).collect()),
                );

                Yaml::Hash(hash)
            }
            _ => panic!("Frontmatter should always be hash."),
        };

        let frontmatter_text = {
            let mut out_str = String::new();
            yaml_rust::YamlEmitter::new(&mut out_str)
                .dump(&frontmatter)
                .unwrap();

            // I do not know why, but a "---" is already added in the beginning of `out_str`
            format!("{}\n---\n\n", out_str)
        };

        let body: String = self.tokens.iter().map(|t| t.to_string()).collect();

        frontmatter_text + body.as_str()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vault {
    // Maps normalized note names to notes
    notes: HashMap<String, Note>,
    attachments: HashMap<String, PathBuf>,
    vault_path: PathBuf,
    output_path: PathBuf,
    hugo_site_path: PathBuf,
}

impl Vault {
    pub fn add_note(&mut self, path: &PathBuf) {
        println!("Adding note: {}", path.display());
        let name = path.file_stem().unwrap().to_str().unwrap();
        let tokens: Vec<Token> = match Lexer::from_file(path.as_path()) {
            Ok(lexer) => lexer.collect(),
            Err(e) => {
                eprintln!("ERROR: {:?}\nSkipping file '{}'.", e, path.display());
                return;
            }
        };

        let (mut frontmatter, tokens) = match tokens.first() {
            Some(Token::Frontmatter(f)) => (f.clone(), tokens.split_first().unwrap().1.to_vec()),
            _ => (Yaml::Hash(yaml_rust::yaml::Hash::new()), tokens),
        };

        match &mut frontmatter {
            Yaml::Hash(f) => {
                f.insert(
                    Yaml::String("type".to_string()),
                    Yaml::String("note".to_string()),
                );
            }
            _ => panic!("Frontmatter yaml root should be `Hash`."),
        }

        let note = Note {
            path: path.strip_prefix(&self.vault_path).unwrap().to_path_buf(),
            name: name.to_string(),
            tokens,
            frontmatter,
            links: Vec::new(),
            tags: Vec::new(),
            backlinks: Vec::new(),
        };
        let normalized_name = normalize_string(name.to_string());
        self.notes.insert(normalized_name, note);
    }

    fn add_attachment(&mut self, path: PathBuf) {
        println!("Adding attachment: {:?}", path.display());
        let name = normalize_string(path.file_name().unwrap().to_str().unwrap().to_string());
        self.attachments.insert(name, path);
    }

    pub fn notes(&self) -> &HashMap<String, Note> {
        &self.notes
    }

    pub fn add_dir(&mut self, path: &PathBuf) -> io::Result<()> {
        match path.file_name().map(|n| n.to_str()) {
            Some(Some(".git"))
            | Some(Some(".obsidian"))
            | Some(Some(".trash"))
            | Some(Some("Excalidraw")) => return Ok(()),
            _ => {}
        }

        println!("Adding directory: {}", path.display());

        for file in fs::read_dir(path)? {
            let file = file?;
            let file_path = file.path();
            if file_path.is_file() {
                match file_path.extension() {
                    Some(ex) => match ex.to_str() {
                        Some("md") => self.add_note(&file_path),
                        _ => self.add_attachment(file_path),
                    },
                    None => self.add_attachment(file_path),
                }
            } else if file_path.is_dir() {
                self.add_dir(&file_path)?;
            }
        }
        Ok(())
    }

    pub fn from_directory(
        path: &PathBuf,
        output_path: PathBuf,
        hugo_site_path: Option<PathBuf>,
    ) -> io::Result<Self> {
        let hugo_site_path = match hugo_site_path {
            Some(p) => p,
            None => {
                let mut site_path = output_path.clone();

                'outer: loop {
                    if site_path.is_dir() {
                        for file in fs::read_dir(&site_path).unwrap() {
                            let file = file.unwrap();
                            if file.file_name() == std::ffi::OsString::from("config.toml") {
                                break 'outer;
                            }
                        }
                    }

                    match site_path.parent() {
                        Some(p) => site_path = p.to_path_buf(),
                        None => panic!("Could not find hugo site path."),
                    }
                }
                site_path
            }
        };

        println!("Found Hugo site path :'{}'", hugo_site_path.display());

        let vault = Self {
            notes: HashMap::new(),
            attachments: HashMap::new(),
            vault_path: path.clone(),
            output_path,
            hugo_site_path,
        };
        Ok(vault)
    }

    pub fn index(&mut self) {
        for note_name in self.notes.clone().keys() {
            let mut note = self.notes.get(&note_name.clone()).unwrap().clone();
            for token in note.tokens.clone() {
                match token {
                    Token::Text(_) => {}
                    Token::Header(_, _) => {}
                    Token::Callout(_) => {}
                    Token::Quote(_) => {}
                    Token::Frontmatter(_) => {}
                    Token::Tag(tag) => note.tags.push(tag.to_string()),
                    Token::Link(link) => {
                        // if `dest` field is emply, the link points to itself and we don't have to
                        // do anything in that case.
                        if link.dest.is_empty() {
                            continue;
                        }

                        let to_note_name = normalize_string(link.dest.clone());
                        let mut to_note = match self.notes.get(&to_note_name) {
                            Some(n) => n.clone(),
                            None => {
                                // Is it an attachment?
                                if self.attachments.get(&to_note_name).is_some() {
                                    continue;
                                }

                                eprintln!(
                                    "WARNING [{}]: Could not find linked note: '{}'",
                                    note_name, to_note_name
                                );
                                continue;
                            }
                        };

                        // TODO: Add path instead of link
                        note.links.push(to_note_name.clone());
                        to_note.backlinks.push(
                            self.hugo_site_path.to_str().unwrap().to_string()
                                + "/"
                                + &normalize_path_to_string(&note.path),
                        );
                        self.notes.insert(to_note_name, to_note);
                    }
                }
                self.notes.insert(note_name.clone(), note.clone());
            }
        }
    }

    pub fn output(&self) {
        // Convert Notes
        for note in self.notes.values() {
            let note_text = note.to_string();
            let out_path = self.output_path.join(&note.path);

            let dir = out_path.parent().unwrap();
            fs::create_dir_all(dir).unwrap();

            fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(out_path)
                .unwrap()
                .write_all(note_text.as_bytes())
                .unwrap();
        }

        // Copy attachments
        for attachment in self.attachments.values() {
            let vault_path = self.vault_path.join(attachment);
            let out_path = self.output_path.join(attachment);
            let dir = out_path.parent().unwrap();
            fs::create_dir_all(dir).unwrap();
            fs::copy(vault_path, out_path).unwrap();
        }
    }
}
