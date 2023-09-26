use queues::{IsQueue, Queue};
use std::fs;

#[derive(Debug)]
pub enum LexerError {
    Io(std::io::Error),
    Utf(std::string::FromUtf8Error),
}

// pub struct TextSpan {
//     file: String,
//     range: (usize, usize),
// }

#[derive(Debug, Clone)]
pub struct Link {
    dest: String,
    show_how: Option<String>,
    options: Option<String>,
    render: bool,
}

#[derive(Debug, Clone)]
pub struct Callout {
    kind: String,
    title: Vec<Token>,
    contents: Vec<Token>,
    foldable: bool,
}

#[derive(Debug, Clone)]
pub enum Token {
    Text(String),
    Tag(String),
    Header(usize, Vec<Token>),
    Link(Link),
    Callout(Callout),
    Quote(Vec<Token>),
}

pub struct Lexer {
    cursor: usize,
    text: Vec<char>,
    queue: Queue<Token>,
}

impl Lexer {
    pub fn new(text: String) -> Self {
        let chars = text.chars().collect();
        Self {
            cursor: 0,
            text: chars,
            queue: Queue::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, LexerError> {
        let bytes = fs::read(path).map_err(|e| LexerError::Io(e))?;
        let text = String::from_utf8(bytes).map_err(|e| LexerError::Utf(e))?;
        Ok(Self::new(text))
    }

    fn peak(&self, offset: isize) -> Option<char> {
        let index = (self.cursor as isize + offset) as usize;
        match self.text.get(index) {
            Some(c) => Some(c.clone()),
            None => None,
        }
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

        match fields.len() {
            0 => panic!("Emply link."),
            1 => Token::Link(Link {
                dest: fields[0].clone(),
                show_how: None,
                options: None,
                render: shown,
            }),
            2 => Token::Link(Link {
                dest: fields[0].clone(),
                show_how: Some(fields[1].clone()),
                options: None,
                render: shown,
            }),
            3 => Token::Link(Link {
                dest: fields[0].clone(),
                show_how: Some(fields[2].clone()),
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

        while self.current() != Some('\n') {
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
        let mut is_callout = false;

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
        }
        else {
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
        assert_eq!(self.consume(), Some(']'));
        let mut foldable = false;
        if self.current() == Some('-') {
            self.consume();
            foldable = true;
        }
        self.consume_whitespace();
        let title = self.consume_line();

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

        match self.current()?.clone() {
            '#' => {
                let next = self.peak(1);
                if next == Some(' ') || next == Some('#') {
                    Some(self.consume_header())
                } else {
                    Some(self.consume_tag())
                }
            }
            '>' => Some(self.consume_block()),
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
        }
    }
}
