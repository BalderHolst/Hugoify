use std::{fs, str::Chars};

#[derive(Debug)]
pub enum LexerError {
    Io(std::io::Error),
    Utf(std::string::FromUtf8Error),
}

pub enum TokenKind {
    Text
}

pub struct TextSpan {
    file: String,
    range: (usize, usize),
}

pub struct Token {
    kind: TokenKind,
    span: TextSpan,
}

pub struct Lexer {
    cursor: usize,
    text: Vec<char>,
}


impl Lexer {
    pub fn new(text: String) -> Self { 
        let chars = text.chars().collect();
        Self { cursor: 0, text: chars }
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

    fn current(self) -> char {
        self.peak(0).expect("current char should never be invalid")
    }

}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
