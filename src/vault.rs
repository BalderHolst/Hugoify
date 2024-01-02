use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    io::{self, Write},
    path::PathBuf,
};


use yaml_rust::Yaml;

use crate::{lexer::{Lexer, Token}, latex};

fn remove_extension(path: &PathBuf) -> PathBuf {
    path.parent().unwrap().join(path.file_stem().unwrap())
}

fn normalize_path_to_string_keep_ext(path: &PathBuf) -> String {
    let extension = path.extension();
    let mut path = path.clone();
    path.set_extension("");
    let path = normalize_path_to_string(&path);
    match extension {
        Some(ext) => path + "." + ext.to_str().unwrap(),
        None => path,
    }
}

fn normalize_path_to_string(path: &PathBuf) -> String {
    let components = path.components();
    components
        .map(|s| normalize_string(s.as_os_str().to_str().unwrap().to_string()))
        .collect::<Vec<String>>()
        .join("/")
}

fn normalize_path(path: &PathBuf) -> PathBuf {
    PathBuf::from(normalize_path_to_string_keep_ext(path))
}

fn normalize_name(mut name: String) -> String {
    if name.contains('/') {
        eprintln!("WARNING: Normalizing name with '/': `{name}`. Only using filename.");
        name = name.split_once('/').unwrap().1.to_string();
    }
    return normalize_string(name);
}

// TODO: Use references
fn normalize_string(name: String) -> String {
    name.chars()
        .map(|c| match c {
            ' ' => '-',
            _ => c.to_lowercase().next().unwrap(),
        })
        // Remove repeated '-'
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect::<Vec<String>>()
        .join("-")
        // Filter other chars away in name
        .chars()
        .filter(|c| match c {
            '\'' => false,
            '.' => false,
            _ => true,
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

impl Note {
    pub fn path_debth(&self) -> usize {
        self.path.components().count()
    }

    fn add_backlink(&mut self, backlink: String) {
        if !self.backlinks.contains(&backlink) {
            self.backlinks.push(backlink);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vault {
    /// Maps normalized note names to notes
    notes: HashMap<String, Note>,
    /// Maps normalized attachment names to their relative paths within vault
    attachments: HashMap<String, PathBuf>,
    /// Path to Obidian vault
    obsidian_vault_path: PathBuf,
    /// Path to the generated vault relative to `hugo_site_path`
    hugo_vault_path: PathBuf,
    /// Path to the generated vault for attachments relative to `hugo_site_path`
    hugo_vault_attachment_path: PathBuf,
    /// The path to the hugo site on local machine
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
            path: path
                .strip_prefix(&self.obsidian_vault_path)
                .unwrap()
                .to_path_buf(),
            name: name.to_string(),
            tokens,
            frontmatter,
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
        self.attachments.insert(
            name,
            path.strip_prefix(&self.obsidian_vault_path)
                .unwrap()
                .to_path_buf(),
        );
    }

    #[cfg(test)]
    pub fn notes(&self) -> &HashMap<String, Note> {
        &self.notes
    }

    pub fn add_dir(&mut self, path: &PathBuf) -> io::Result<()> {
        match path.file_name().map(|n| n.to_str()) {
            Some(Some(".git"))
            | Some(Some(".obsidian"))
            | Some(Some("Templates"))
            | Some(Some("External"))
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

        let hugo_vault_path = output_path
            .strip_prefix(&hugo_site_path)
            .unwrap()
            .to_path_buf();

        let hugo_vault_attachment_path = PathBuf::from("static").join(
            hugo_vault_path
                .strip_prefix("content")
                .unwrap()
                .to_path_buf(),
        );

        let vault = Self {
            notes: HashMap::new(),
            attachments: HashMap::new(),
            obsidian_vault_path: path.clone(),
            hugo_vault_path,
            hugo_vault_attachment_path,
            hugo_site_path,
        };
        Ok(vault)
    }

    pub fn index(&mut self) {
        for note_name in self.notes.clone().keys() {
            let mut note = self.notes.get(&note_name.clone()).unwrap().clone();
            for token in note.tokens.clone() {
                match token {
                    Token::Text(_) => {},
                    Token::Header(_, _) => {},
                    Token::Callout(_) => {},
                    Token::Quote(_) => {},
                    Token::Frontmatter(_) => {},
                    Token::Divider => {},
                    Token::InlineMath(_) => {},
                    Token::DisplayMath(_) => {},
                    Token::Tag(tag) => note.tags.push(tag.to_string()),
                    Token::Link(link) => {
                        // if `dest` field is emply, the link points to itself and we don't have to
                        // do anything in that case.
                        if link.dest.is_empty() {
                            continue;
                        }

                        let to_note_name = normalize_name(link.dest.clone());
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

                        let hugo_site_path = self
                            .hugo_vault_path
                            .strip_prefix("content")
                            .unwrap()
                            .join(&note.path.parent().unwrap())
                            .join(&note.path.file_stem().unwrap());
                        to_note.add_backlink(
                            "/".to_string() + &normalize_path_to_string(&hugo_site_path),
                        );
                        self.notes.insert(to_note_name, to_note);
                    },
                }
                self.notes.insert(note_name.clone(), note.clone());
            }
        }
    }

    pub fn tokens_to_string<I>(&self, note: &Note, tokens: I) -> String
    where
        I: IntoIterator<Item = Token>,
    {
        tokens
            .into_iter()
            .map(|token| self.token_to_string(note, &token))
            .collect::<String>()
    }

    fn get_note(&self, normalized_name: &String) -> Option<&Note> {
        self.notes.get(normalized_name)
    }

    fn get_attachment(&self, normalized_name: &String) -> Option<&PathBuf> {
        self.attachments.get(normalized_name)
    }

    fn token_to_string(&self, note: &Note, token: &Token) -> String {
        let mut is_attachment = false;

        match token {
            Token::Text(s) => s.clone(),
            Token::Tag(s) => format!("#{s}"),
            Token::Header(level, title) => format!(
                "{} {}",
                "#".repeat(*level),
                self.tokens_to_string(note, title.clone()),
            ),
            Token::Link(link) => {
                let normalized_name = normalize_name(link.dest.clone());
                let normalized_path = match self.get_note(&normalized_name){
                    Some(note) if note.path.extension() == Some(OsStr::new("md")) => normalize_path_to_string(&remove_extension(&note.path)),
                    Some(note) => normalize_path_to_string(&note.path),

                    // If link does not point to a note
                    None => {
                        match self.get_attachment(&normalized_name) {
                            // Found attachment!
                            Some(p) => {
                                is_attachment = true;
                                normalize_path_to_string_keep_ext(p)
                            },

                            // Remove link if it does not point to anything
                            None => {
                                let s = link.label();
                                // TODO: specify from where in error
                                eprintln!("WARNING: Removing link to '{s}'.");
                                return s.to_owned();
                            }
                        }

                    },
                };
                let url = "../".repeat(note.path_debth()) + normalized_path.as_str();
                let vault_url = "notes/vault/".to_string() + normalized_path.as_str();
                match (is_attachment, link.render, &link.position) {
                    (_, true, None) => format!("![{}]({})", link.label(), url),
                    (_, false, None) => format!("[{}]({})", link.label(), url),
                    // TODO: actually embed things idk
                    (_, true, Some(position))  => format!("![{}#{}]({})", link.label(), position, url),
                    (false, false, Some(_position)) => format!(
                        // TODO: Jump to heading (position) when link is clicked
                        "[{show_how}]({{{{< ref \"{url}\" >}}}})",
                        show_how=link.label(),
                        url=vault_url
                        //pos=position,
                        ),
                    (true, false, Some(position)) => format!(
                        "[{show_how}]({url}#{pos})",
                        show_how=link.label(),
                        pos=position,
                        url=url
                        ),
                }
            },
            Token::Callout(callout) => format!(
                "\n{{{{< callout type=\"{}\" title=\"{}\" foldable=\"{}\" >}}}}\n{}{{{{< /callout >}}}}\n",
                callout.kind,
                self.tokens_to_string(note, callout.title.clone()),
                if callout.foldable { "true" } else { "false" },
                self.tokens_to_string(note, callout.contents.clone()),
            ),
            Token::Quote(quote) => quote.iter().map(|token| "> ".to_string() + self.token_to_string(note, token).as_str()).collect(),
            Token::Frontmatter(_) => panic!("Frontmatter should never be part of a note body."),
            Token::Divider => "\n--------------------\n".to_string(),
            Token::InlineMath(math) => format!("${}$", latex::process_latex(math.to_string())),
            Token::DisplayMath(math) => format!("$$\n{}\n$$", latex::process_latex(math.to_string())),
        }
    }

    fn note_to_string(&self, note: &Note) -> String {
        // This could probably be done better...
        let frontmatter = match note.frontmatter.clone() {
            Yaml::Hash(mut hash) => {
                // Title
                hash.insert(
                    Yaml::String("title".to_string()),
                    Yaml::String(note.name.clone()),
                );

                // Tags
                hash.insert(
                    Yaml::String("note_tags".to_string()),
                    Yaml::Array(note.tags.iter().map(|t| Yaml::String(t.clone())).collect()),
                );

                // Backlinks
                hash.insert(
                    Yaml::String("backlinks".to_string()),
                    Yaml::Array(
                        note.backlinks
                            .iter()
                            .map(|t| Yaml::String(t.clone()))
                            .collect(),
                    ),
                );

                // Links
                hash.insert(
                    Yaml::String("links".to_string()),
                    Yaml::Array(note.links.iter().map(|t| Yaml::String(t.clone())).collect()),
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

        let mut tokens = note.tokens.clone();

        for (tags_begin_offset, rtoken) in tokens.clone().iter().rev().enumerate() {
            match rtoken {
                Token::Divider => {
                    let (slice, _) = tokens
                        .as_slice()
                        .split_at(tokens.len() - tags_begin_offset - 1);
                    tokens = Vec::from(slice);
                }
                Token::Tag(_) => {}
                t => {
                    if !t.is_whitespace() {
                        break;
                    }
                }
            }
        }

        let body: String = self.tokens_to_string(note, tokens);

        frontmatter_text + body.as_str()
    }

    pub fn output(&self) {
        // Convert Notes
        for note in self.notes.values() {
            let note_text = self.note_to_string(note);
            let out_path = self
                .hugo_site_path
                .join(&self.hugo_vault_path)
                .join(&note.path);

            let out_dir = out_path.parent().unwrap();

            // TODO: this does not work with paths with `.`
            if let Err(e) = fs::create_dir_all(normalize_path(&out_dir.to_path_buf())) {
                eprint!(
                    "Could not create output directory `{}`: {}",
                    out_dir.display(),
                    e
                );
                std::process::exit(1)
            }

            let out_path = normalize_path_to_string_keep_ext(&out_path);

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
            let vault_path = self.obsidian_vault_path.join(attachment);
            let out_path = normalize_path(
                &self
                    .hugo_site_path
                    .join(&self.hugo_vault_attachment_path)
                    .join(attachment),
            );
            let dir = out_path.parent().unwrap();
            fs::create_dir_all(dir).unwrap();
            fs::copy(vault_path, out_path).unwrap();
        }
    }
}
