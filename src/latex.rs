/// Convert latex to be hugo-friendly
pub fn process_latex(input: String) -> String {
    let mut output = String::new();

    for c in input.chars() {
        match c {
            '\n' => output.push(' '),
            _ => output.push(c),
        }
    }

    output
}

#[derive(Clone, Debug, PartialEq, Default)]
struct TexTokenDecoration {
    superscript: Vec<TexToken>,
    subscript: Vec<TexToken>,
}

#[derive(Clone, Debug, PartialEq)]
struct TexCommand {
    name: String,
    arguments: Vec<Vec<TexToken>>,
}

type TexSymbol = String;

#[derive(Clone, Debug, PartialEq)]
enum TexTokenKind {
    /// `\command` or `\command{arg1 arg2 ...}`
    Command(TexCommand),

    /// The `\\` sequence
    Newline,

    /// Everything else
    Symbol(TexSymbol),
}

#[derive(Clone, Debug, PartialEq)]
struct TexToken {
    kind: TexTokenKind,
    decoration: TexTokenDecoration,
}

impl TexToken {
    fn new_plain(kind: TexTokenKind) -> Self {
        Self {
            kind,
            decoration: TexTokenDecoration::default(),
        }
    }

    fn is_empty(&self) -> bool {
        if self.kind == TexTokenKind::Symbol("".to_string())
            && self.decoration == TexTokenDecoration::default()
        {
            true
        } else {
            false
        }
    }
}

#[derive(Debug)]
struct TexParser {
    cursor: usize,
    chars: Vec<char>,
    unfinished_token: String,
}

impl TexParser {
    fn new(chars: Vec<char>) -> Self {
        Self {
            cursor: 0,
            chars,
            unfinished_token: String::new(),
        }
    }

    fn current(&self) -> Option<char> {
        self.chars.get(self.cursor).cloned()
    }

    fn consume(&mut self) -> Option<char> {
        let c = self.current();
        self.cursor += 1;
        c
    }

    fn finish_text_token(&mut self) -> TexToken {
        let token = TexToken {
            kind: TexTokenKind::Symbol(self.unfinished_token.clone()),
            decoration: TexTokenDecoration::default(),
        };
        self.unfinished_token.clear();
        token
    }

    /// `\command{arg1}{arg2}{arg3}`
    fn parse_command(&mut self) -> TexToken {
        let mut name = String::new();
        let mut args = Vec::new();
        while let Some(c) = self.consume() {
            match c {
                c if c.is_whitespace() => break,
                '{' => {
                    self.cursor -= 1;
                    while self.current() == Some('{') {
                        self.consume();
                        args.push(self.parse());
                    }
                    break;
                }
                c if c.is_ascii_punctuation() => {
                    self.cursor -= 1;
                    break;
                }
                c => name.push(c),
            }
        }

        TexToken {
            kind: TexTokenKind::Command(TexCommand {
                name,
                arguments: args,
            }),
            decoration: TexTokenDecoration::default(),
        }
    }

    fn parse(&mut self) -> Vec<TexToken> {
        let mut paren_level: usize = 0;
        let mut tokens = Vec::new();
        let mut current = self.consume();

        while let Some(current_char) = current {
            match current_char {
                '\\' => {
                    tokens.push(self.finish_text_token());
                    tokens.push(self.parse_command());
                }
                '{' => {
                    paren_level += 1;
                    self.unfinished_token.push(current_char);
                }
                '}' => {
                    if paren_level == 0 {
                        break;
                    }
                    paren_level -= 1;
                }
                '_' => {
                    tokens.push(self.finish_text_token());
                    if let Some(c) = self.consume() {
                        if c == '{' {
                            // } make treesitter happy
                            if let Some(last_token) = tokens.last_mut() {
                                last_token.decoration.subscript = self.parse();
                            }
                        } else {
                            if let Some(last_token) = tokens.last_mut() {
                                last_token.decoration.subscript = vec![TexToken {
                                    kind: TexTokenKind::Symbol(c.to_string()),
                                    decoration: TexTokenDecoration::default(),
                                }];
                            }
                        }
                    }
                }
                '^' => {
                    tokens.push(self.finish_text_token());
                    if let Some(c) = self.consume() {
                        if c == '{' {
                            // } make treesitter happy
                            if let Some(last_token) = tokens.last_mut() {
                                last_token.decoration.superscript = self.parse();
                            }
                        } else {
                            if let Some(last_token) = tokens.last_mut() {
                                last_token.decoration.superscript = vec![TexToken {
                                    kind: TexTokenKind::Symbol(c.to_string()),
                                    decoration: TexTokenDecoration::default(),
                                }];
                            }
                        }
                    }
                }
                // Whitespace splits tokens
                c if c.is_whitespace() => {
                    if !self.unfinished_token.is_empty() {
                        tokens.push(self.finish_text_token());
                    }
                }
                c => self.unfinished_token.push(c),
            }

            current = self.consume()
        }

        tokens.push(self.finish_text_token());

        tokens.iter().filter(|t| !t.is_empty()).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Usage:
    /// let tex = "";
    /// let expected: Vec<TexToken> = vec![];
    /// assert_eq!(expected, parse_tex(tex));
    fn parse_tex(tex: &str) -> Vec<TexToken> {
        TexParser::new(tex.chars().collect()).parse()
    }

    #[test]
    fn test_commands_without_arguments() {
        let tex = r#"\sin(\lambda) \cdot x"#;
        let expected: Vec<TexToken> = vec![
            TexToken::new_plain(TexTokenKind::Command(TexCommand {
                name: "sin".to_string(),
                arguments: vec![],
            })),
            TexToken::new_plain(TexTokenKind::Symbol("(".to_string())),
            TexToken::new_plain(TexTokenKind::Command(TexCommand {
                name: "lambda".to_string(),
                arguments: vec![],
            })),
            TexToken::new_plain(TexTokenKind::Symbol(")".to_string())),
            TexToken::new_plain(TexTokenKind::Command(TexCommand {
                name: "cdot".to_string(),
                arguments: vec![],
            })),
            TexToken::new_plain(TexTokenKind::Symbol("x".to_string())),
        ];
        assert_eq!(expected, parse_tex(tex));
    }

    #[test]
    fn test_commands_with_arguments() {
        let tex = r#"\frac{numerator is here}{denominator} \function{arg1}"#;
        let expected: Vec<TexToken> = vec![
            TexToken::new_plain(TexTokenKind::Command(TexCommand {
                name: "frac".to_string(),
                arguments: vec![
                    vec![
                        TexToken::new_plain(TexTokenKind::Symbol("numerator".to_string())),
                        TexToken::new_plain(TexTokenKind::Symbol("is".to_string())),
                        TexToken::new_plain(TexTokenKind::Symbol("here".to_string())),
                    ],
                    vec![TexToken::new_plain(TexTokenKind::Symbol(
                        "denominator".to_string(),
                    ))],
                ],
            })),
            TexToken::new_plain(TexTokenKind::Command(TexCommand {
                name: "function".to_string(),
                arguments: vec![vec![TexToken::new_plain(TexTokenKind::Symbol(
                    "arg1".to_string(),
                ))]],
            })),
        ];
        assert_eq!(expected, parse_tex(tex));
    }

    #[test]
    fn test_commands_with_arguments2() {
        let tex = r#"\command{arg1}{arg2}{arg3}"#;
        let expected: Vec<TexToken> =
            vec![TexToken::new_plain(TexTokenKind::Command(TexCommand {
                name: "command".to_string(),
                arguments: vec![
                    vec![TexToken::new_plain(TexTokenKind::Symbol(
                        "arg1".to_string(),
                    ))],
                    vec![TexToken::new_plain(TexTokenKind::Symbol(
                        "arg2".to_string(),
                    ))],
                    vec![TexToken::new_plain(TexTokenKind::Symbol(
                        "arg3".to_string(),
                    ))],
                ],
            }))];
        assert_eq!(expected, parse_tex(tex));
    }

    #[test]
    fn test_superscript() {
        let tex = "hello this^is a test^{of superscript}. It should go^{super^{super^{super}}}!";
        let expected: Vec<TexToken> = vec![
            TexToken::new_plain(TexTokenKind::Symbol("hello".to_string())),
            TexToken {
                kind: TexTokenKind::Symbol("this".to_string()),
                decoration: TexTokenDecoration {
                    subscript: vec![],
                    superscript: vec![TexToken::new_plain(TexTokenKind::Symbol("i".to_string()))],
                },
            },
            TexToken::new_plain(TexTokenKind::Symbol("s".to_string())),
            TexToken::new_plain(TexTokenKind::Symbol("a".to_string())),
            TexToken {
                kind: TexTokenKind::Symbol("test".to_string()),
                decoration: TexTokenDecoration {
                    subscript: vec![],
                    superscript: vec![
                        TexToken::new_plain(TexTokenKind::Symbol("of".to_string())),
                        TexToken::new_plain(TexTokenKind::Symbol("superscript".to_string())),
                    ],
                },
            },
            TexToken::new_plain(TexTokenKind::Symbol(".".to_string())),
            TexToken::new_plain(TexTokenKind::Symbol("It".to_string())),
            TexToken::new_plain(TexTokenKind::Symbol("should".to_string())),
            TexToken {
                kind: TexTokenKind::Symbol("go".to_string()),
                decoration: TexTokenDecoration {
                    subscript: vec![],
                    superscript: vec![TexToken {
                        kind: TexTokenKind::Symbol("super".to_string()),
                        decoration: TexTokenDecoration {
                            subscript: vec![],
                            superscript: vec![TexToken {
                                kind: TexTokenKind::Symbol("super".to_string()),
                                decoration: TexTokenDecoration {
                                    subscript: vec![],
                                    superscript: vec![TexToken::new_plain(TexTokenKind::Symbol(
                                        "super".to_string(),
                                    ))],
                                },
                            }],
                        },
                    }],
                },
            },
            TexToken::new_plain(TexTokenKind::Symbol("!".to_string())),
        ];
        assert_eq!(expected, parse_tex(tex));
    }

    #[test]
    fn test_subscript() {
        let tex = "hello this_is a test_{of subscript}. It should go_{sub_{sub_{sub}}}!";
        let expected: Vec<TexToken> = vec![
            TexToken::new_plain(TexTokenKind::Symbol("hello".to_string())),
            TexToken {
                kind: TexTokenKind::Symbol("this".to_string()),
                decoration: TexTokenDecoration {
                    superscript: vec![],
                    subscript: vec![TexToken::new_plain(TexTokenKind::Symbol("i".to_string()))],
                },
            },
            TexToken::new_plain(TexTokenKind::Symbol("s".to_string())),
            TexToken::new_plain(TexTokenKind::Symbol("a".to_string())),
            TexToken {
                kind: TexTokenKind::Symbol("test".to_string()),
                decoration: TexTokenDecoration {
                    superscript: vec![],
                    subscript: vec![
                        TexToken::new_plain(TexTokenKind::Symbol("of".to_string())),
                        TexToken::new_plain(TexTokenKind::Symbol("subscript".to_string())),
                    ],
                },
            },
            TexToken::new_plain(TexTokenKind::Symbol(".".to_string())),
            TexToken::new_plain(TexTokenKind::Symbol("It".to_string())),
            TexToken::new_plain(TexTokenKind::Symbol("should".to_string())),
            TexToken {
                kind: TexTokenKind::Symbol("go".to_string()),
                decoration: TexTokenDecoration {
                    superscript: vec![],
                    subscript: vec![TexToken {
                        kind: TexTokenKind::Symbol("sub".to_string()),
                        decoration: TexTokenDecoration {
                            superscript: vec![],
                            subscript: vec![TexToken {
                                kind: TexTokenKind::Symbol("sub".to_string()),
                                decoration: TexTokenDecoration {
                                    superscript: vec![],
                                    subscript: vec![TexToken::new_plain(TexTokenKind::Symbol(
                                        "sub".to_string(),
                                    ))],
                                },
                            }],
                        },
                    }],
                },
            },
            TexToken::new_plain(TexTokenKind::Symbol("!".to_string())),
        ];
        assert_eq!(expected, parse_tex(tex));
    }
}
