use queues::{IsQueue, Queue};
use std::{fs, path::Path};
use yaml_rust::{self, ScanError, Yaml};

#[derive(Debug)]
pub enum LexerError {
    Io(std::io::Error),
    Utf(std::string::FromUtf8Error),
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub dest: String,
    pub position: Option<String>,
    pub show_how: String,
    pub options: Option<String>,
    pub render: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Callout {
    pub kind: String,
    pub title: Vec<Token>,
    pub contents: Vec<Token>,
    pub foldable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Text(String),
    Tag(String),
    Header(usize, Vec<Token>),
    Link(Link),
    Callout(Callout),
    Quote(Vec<Token>),
    Frontmatter(Yaml), // This can only appear as the first token
    Divider,
}

impl Token {
    pub fn is_whitespace(&self) -> bool {
        match self {
            Token::Text(t) => {
                for c in t.chars() {
                    if !c.is_whitespace() {
                        return false;
                    }
                }
                true
            }
            Token::Tag(_) => false,
            Token::Header(_, _) => false,
            Token::Link(_) => false,
            Token::Callout(_) => false,
            Token::Quote(_) => false,
            Token::Frontmatter(_) => false,
            Token::Divider => false,
        }
    }
}

pub struct Lexer {
    cursor: usize,
    text: Vec<char>,
    queue: Queue<Token>,
    first_token: bool,
}

impl Lexer {
    pub fn new(text: String) -> Self {
        let chars = text.chars().collect();
        Self {
            cursor: 0,
            text: chars,
            queue: Queue::new(),
            first_token: true,
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, LexerError> {
        let bytes = fs::read(path).map_err(LexerError::Io)?;
        let text = String::from_utf8(bytes).map_err(LexerError::Utf)?;
        Ok(Self::new(text))
    }

    fn peak(&self, offset: isize) -> Option<char> {
        let index = (self.cursor as isize + offset) as usize;
        self.text.get(index).copied()
    }

    // Consume a character
    fn consume(&mut self) -> Option<char> {
        self.cursor += 1;
        self.peak(-1)
    }

    fn current(&self) -> Option<char> {
        self.peak(0)
    }

    fn consume_whitespace(&mut self) -> String {
        let mut s = String::new();
        while match self.current() {
            Some(c) => c.is_whitespace(),
            None => false,
        } {
            let c = self.consume().unwrap();
            s.push(c);
        }
        s
    }

    fn consume_link(&mut self) -> Token {
        let mut fields = vec![String::new()];
        let mut shown = false;

        if self.current() == Some('!') {
            shown = true;
            self.consume();
        }

        assert_eq!(self.consume(), Some('['));
        assert_eq!(self.consume(), Some('['));

        while self.current() != Some(']') {
            match self.consume() {
                // Start new field
                Some('|') => fields.push(String::new()),

                // Add character to current field
                Some(c) => fields.last_mut().unwrap().push(c),

                None => break,
            };
        }

        assert_eq!(self.consume(), Some(']'));

        // TODO: Give good error message
        assert_eq!(self.consume(), Some(']'));

        let note_name = fields[0].clone();
        let (note_name, position) = match note_name.split_once('#') {
            Some((d, p)) => (d.to_string(), Some(p.to_string())),
            None => (note_name, None),
        };

        debug_assert!(!note_name.contains("#"));

        match fields.len() {
            0 => todo!("Emply link."),
            1 => Token::Link(Link {
                dest: note_name.clone(),
                position,
                show_how: note_name,
                options: None,
                render: shown,
            }),
            2 => Token::Link(Link {
                dest: note_name,
                position,
                show_how: fields[1].clone(),
                options: None,
                render: shown,
            }),
            3 => Token::Link(Link {
                dest: note_name,
                position,
                show_how: fields[2].clone(),
                options: Some(fields[1].clone()),
                render: shown,
            }),
            n => panic!("Invalid amount of fields in link: `{n}`."),
        }
    }

    fn consume_line(&mut self) -> Vec<Token> {
        // The vector of tokens for this line
        let mut tokens = Vec::new();

        // The string we are currently building
        let mut s = String::new();

        // Wether the line ends the entire file.
        let mut found_eof = false;

        while !matches!(self.current(), Some('\n') | None) {
            match (self.current(), self.peak(1), self.peak(2)) {
                (Some('['), Some('['), _) | (Some('!'), Some('['), Some('[')) => {
                    tokens.push(Token::Text(s.clone()));
                    s.clear();
                    tokens.push(self.consume_link());
                }
                (Some(c), _, _) => {
                    self.consume();
                    s.push(c);
                }
                (None, _, _) => {
                    found_eof = true;
                }
            }
        }
        if !found_eof {
            // Consume newline
            self.consume();
            s.push('\n');
        }
        tokens.push(Token::Text(s));
        tokens
    }

    fn consume_header(&mut self) -> Token {
        assert_eq!(self.current(), Some('#'));
        let mut level: usize = 0;
        while self.current() == Some('#') {
            self.consume();
            level += 1;
        }
        let _ = self.consume_whitespace();
        let header = self.consume_line();
        Token::Header(level, header)
    }

    fn consume_tag(&mut self) -> Token {
        assert_eq!(self.consume(), Some('#'));
        let mut text = String::new();
        while match self.current() {
            Some(c) => !c.is_whitespace(),
            None => false,
        } {
            let c = self.consume().unwrap();
            text.push(c);
        }
        Token::Tag(text)
    }

    // blocks of text beginning with '>'
    fn consume_block(&mut self) -> Token {
        // Find the starting character to determine if block is a callout
        assert_eq!(self.peak(0), Some('>'));
        let mut pointer: isize = 1;
        while match self.peak(pointer) {
            Some(c) => c.is_whitespace(),
            None => false,
        } {
            pointer += 1;
        }

        // Is the quote a callout?
        if (self.peak(pointer), self.peak(pointer + 1)) == (Some('['), Some('!')) {
            self.consume_callout()
        } else {
            Token::Quote(self.consume_quote())
        }
    }

    fn consume_callout(&mut self) -> Token {
        assert_eq!(self.consume(), Some('>'));
        self.consume_whitespace();
        assert_eq!(self.consume(), Some('['));
        assert_eq!(self.consume(), Some('!'));
        let mut kind = String::new();
        while self.current() != Some(']') {
            let c = self.consume().unwrap();
            kind.push(c);
        }
        kind = kind.to_lowercase();
        assert_eq!(self.consume(), Some(']'));
        let mut foldable = false;
        if self.current() == Some('-') {
            self.consume();
            foldable = true;
        }
        self.consume_whitespace();

        let title = if self.peak(-1) != Some('\n') {
            let mut title = self.consume_line();
            match title.last() {
                Some(Token::Text(t)) if t.ends_with('\n') => {
                    let mut t = t.clone();
                    t.pop();
                    title.pop();
                    title.push(Token::Text(t))
                }
                _ => {}
            }
            title
        } else {
            // If the callout does not have a title
            let mut name = kind.clone();
            name = name[0..1].to_uppercase() + &name[1..];
            vec![Token::Text(name)]
        };

        let contents = self.consume_quote();

        Token::Callout(Callout {
            kind,
            title,
            contents,
            foldable,
        })
    }

    fn consume_quote(&mut self) -> Vec<Token> {
        let mut contents = Vec::new();
        while self.current() == Some('>') {
            self.consume();

            // Ignore white space
            let _ = self.consume_whitespace();

            contents.extend(self.consume_line());
        }
        contents
    }

    fn consume_front_matter(&mut self) -> Result<Token, ScanError> {
        assert_eq!(self.consume(), Some('-'));
        assert_eq!(self.consume(), Some('-'));
        assert_eq!(self.consume(), Some('-'));

        let mut yaml_text = String::new();

        while (self.peak(0), self.peak(1), self.peak(2)) != (Some('-'), Some('-'), Some('-'))
            && self.peak(2).is_some()
        {
            yaml_text.push(
                self.consume()
                    .expect("The check for Some is already done here."),
            );
        }

        let frontmatter = yaml_rust::YamlLoader::load_from_str(yaml_text.as_str())?;
        if frontmatter.is_empty() {
            return Ok(Token::Text("".to_string()));
        }

        let frontmatter = frontmatter[0].clone();

        // TODO: make good error message if frontmatter is not ended correctly
        assert_eq!(self.consume(), Some('-'));
        assert_eq!(self.consume(), Some('-'));
        assert_eq!(self.consume(), Some('-'));

        Ok(Token::Frontmatter(frontmatter))
    }

    fn consume_divider(&mut self) -> Token {
        debug_assert_eq!(self.consume(), Some('-'));
        debug_assert_eq!(self.consume(), Some('-'));
        debug_assert_eq!(self.consume(), Some('-'));
        while self.current() == Some('-') {
            self.consume();
        }
        while self.current() == Some(' ') {
            self.consume();
        }
        debug_assert!(matches!(self.consume(), Some('\n') | None));
        Token::Divider
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.size() > 0 {
            return Some(
                self.queue
                    .remove()
                    .expect("We should not get to here if the queue is empty."),
            );
        }

        let token = match self.current()? {
            '#' => {
                let next = self.peak(1);
                if next == Some(' ') || next == Some('#') {
                    Some(self.consume_header())
                } else {
                    Some(self.consume_tag())
                }
            }
            '>' => Some(self.consume_block()),
            '-' if self.first_token && (self.peak(1), self.peak(2)) == (Some('-'), Some('-')) => {
                match self.consume_front_matter() {
                    Ok(t) => Some(t),
                    Err(e) => {
                        eprintln!("ERROR: Could not parse frontmatter: {}", e);
                        Some(Token::Text("".to_string()))
                    }
                }
            }
            '-' if (self.peak(1), self.peak(2)) == (Some('-'), Some('-')) => {
                Some(self.consume_divider())
            }
            c if c.is_whitespace() => Some(Token::Text(self.consume_whitespace())),
            _ => {
                for token in self.consume_line() {
                    self.queue.add(token).unwrap();
                }
                match self.queue.remove() {
                    Ok(t) => Some(t),
                    Err(_) => None,
                }
            }
        };
        if self.first_token {
            self.first_token = false
        }
        token
    }
}
