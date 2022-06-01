use std::collections::{HashMap, VecDeque};

use crate::types::{Hashable, MalType};

#[derive(Clone, Debug)]
enum Token {
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Number(i64),
    Symbol(String),
    Keyword(String),
    String(String),
    Quote,
    Backtick,
    Tilde,
    TildeAt,
    At,
    Caret,
}

struct Reader {
    tokens: VecDeque<Token>,
}

impl Reader {
    pub fn new(tokens: VecDeque<Token>) -> Self {
        Reader { tokens }
    }

    pub fn read_form(&mut self) -> Result<MalType, String> {
        let token = match self.tokens.front() {
            Some(token) => token,
            None => return Err("Unexpected EOF.".to_string()),
        };

        match token {
            Token::LeftParen => {
                self.tokens.pop_front();
                self.read_list()
            }
            _ => self.read_atom(),
        }
    }

    fn read_list(&mut self) -> Result<MalType, String> {
        let mut list = Vec::new();

        loop {
            let token = match self.tokens.front() {
                Some(token) => token,
                None => break,
            };

            if let Token::RightParen = token {
                self.tokens.pop_front();
                return Ok(MalType::List(list));
            }

            match self.read_form() {
                Ok(result) => list.push(result),
                Err(message) => return Err(message),
            }
        }

        Err("Unexpected EOF.".to_string())
    }

    fn read_atom(&mut self) -> Result<MalType, String> {
        let token = self.tokens.pop_front().unwrap();
        match token {
            Token::Number(value) => Ok(MalType::Number(value)),
            Token::Symbol(name) => Ok(self.read_symbol(name)),
            Token::String(value) => Ok(MalType::String(unescape_string(&value))),
            Token::Keyword(name) => Ok(MalType::Keyword(name)),
            Token::LeftBrace => self.read_hashmap(),
            Token::LeftBracket => self.read_vector(),
            Token::Quote => self
                .read_form()
                .and_then(|value| Ok(MalType::List(vec![MalType::symbol("quote"), value]))),
            Token::Backtick => self
                .read_form()
                .and_then(|value| Ok(MalType::List(vec![MalType::symbol("quasiquote"), value]))),
            Token::Tilde => self
                .read_form()
                .and_then(|value| Ok(MalType::List(vec![MalType::symbol("unquote"), value]))),
            Token::TildeAt => self.read_form().and_then(|value| {
                Ok(MalType::List(vec![
                    MalType::symbol("splice-unquote"),
                    value,
                ]))
            }),
            Token::At => match self.tokens.pop_front() {
                Some(Token::Symbol(name)) => Ok(MalType::Deref(Box::new(self.read_symbol(name)))),
                next => Err(format!("Unexpected next token {:?}.", next)),
            },
            Token::Caret => self.read_form().and_then(|first| {
                self.read_form()
                    .and_then(|second| Ok(MalType::WithMeta(Box::new(first), Box::new(second))))
            }),
            _ => Err(format!("Unexpected token {:?}.", token)),
        }
    }

    fn read_symbol(&self, name: String) -> MalType {
        match name.as_str() {
            "true" => MalType::True,
            "false" => MalType::False,
            "nil" => MalType::Nil,
            _ => MalType::Symbol(name),
        }
    }

    fn read_hashmap(&mut self) -> Result<MalType, String> {
        let mut hashmap = HashMap::new();

        loop {
            let token = match self.tokens.front() {
                Some(token) => token.clone(),
                None => break,
            };

            if let Token::RightBrace = token {
                self.tokens.pop_front();
                return Ok(MalType::Hashmap(hashmap));
            }

            let key = match self.read_form() {
                Ok(MalType::String(value)) => Hashable::String(value),
                Ok(MalType::Keyword(name)) => Hashable::Keyword(name),
                Ok(_) => return Err(format!("Unexpected token {:?}", token)),
                Err(message) => return Err(message),
            };
            match self.read_form() {
                Ok(result) => hashmap.insert(key, result),
                Err(message) => return Err(message),
            };
        }

        Err("Unexpected EOF.".to_string())
    }
    fn read_vector(&mut self) -> Result<MalType, String> {
        let mut list = Vec::new();

        loop {
            let token = match self.tokens.front() {
                Some(token) => token,
                None => break,
            };

            if let Token::RightBracket = token {
                self.tokens.pop_front();
                return Ok(MalType::Vector(list));
            }

            match self.read_form() {
                Ok(result) => list.push(result),
                Err(message) => return Err(message),
            }
        }

        Err("Unexpected EOF.".to_string())
    }
}

pub fn read_str(string: &str) -> Result<MalType, String> {
    let tokens = tokenize(string);
    match tokens {
        Ok(tokens) => {
            let mut reader = Reader::new(tokens);
            reader.read_form()
        }
        Err(message) => Err(message),
    }
}

fn tokenize(s: &str) -> Result<VecDeque<Token>, String> {
    let mut chars: VecDeque<char> = s.chars().collect();
    let mut tokens = VecDeque::new();
    let mut is_comment = false;
    while let Some(c) = chars.pop_front() {
        if c == '\n' {
            is_comment = false;
            continue;
        }

        if c.is_whitespace() || c == ',' {
            continue;
        }

        if is_comment {
            continue;
        }

        if c == ';' {
            is_comment = true;
            continue;
        }

        let token = match c {
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '\'' => Token::Quote,
            '`' => Token::Backtick,
            '~' => {
                if let Some('@') = chars.front() {
                    chars.pop_front();
                    Token::TildeAt
                } else {
                    Token::Tilde
                }
            }
            '@' => Token::At,
            '^' => Token::Caret,
            '\"' => match string(&mut chars) {
                Ok(token) => token,
                Err(message) => return Err(message),
            },
            '-' => match chars.front() {
                Some(c) if c.is_numeric() => number(true, chars.pop_front().unwrap(), &mut chars),
                _ => symbol(c, &mut chars),
            },
            ':' => keyword(&mut chars),
            c if c.is_numeric() => number(false, c, &mut chars),
            c => symbol(c, &mut chars),
        };
        tokens.push_back(token);
    }
    Ok(tokens)
}

fn string(chars: &mut VecDeque<char>) -> Result<Token, String> {
    let mut string = Vec::new();
    while chars.front().is_some() && *chars.front().unwrap() != '\"' {
        let c = chars.pop_front().unwrap();
        string.push(c);
        if c == '\\' {
            let c = chars.pop_front().unwrap();
            string.push(c);
        }
    }

    if chars.front().is_none() || *chars.front().unwrap() != '\"' {
        return Err("Unexpected EOF.".to_string());
    }

    chars.pop_front();
    Ok(Token::String(string.iter().collect()))
}

fn number(negative: bool, c: char, chars: &mut VecDeque<char>) -> Token {
    let mut number = Vec::new();
    number.push(c);
    while let Some(c) = chars.front() {
        if !c.is_numeric() {
            break;
        }

        number.push(*c);
        chars.pop_front();
    }

    let number = number.iter().collect::<String>().parse().unwrap();
    Token::Number(if negative { -1 * number } else { number })
}

fn symbol(c: char, chars: &mut VecDeque<char>) -> Token {
    let mut name = Vec::new();
    name.push(c);
    while let Some(c) = chars.front() {
        if c.is_whitespace() || is_special_char(*c) {
            break;
        }

        name.push(*c);
        chars.pop_front();
    }

    Token::Symbol(name.iter().collect())
}

fn keyword(chars: &mut VecDeque<char>) -> Token {
    let mut name = Vec::new();
    while let Some(c) = chars.front() {
        if c.is_whitespace() || is_special_char(*c) {
            break;
        }

        name.push(*c);
        chars.pop_front();
    }

    Token::Keyword(name.iter().collect())
}

fn is_special_char(c: char) -> bool {
    "[]{}()'`~^@".contains(c)
}

fn unescape_string(value: &str) -> String {
    let mut result = Vec::new();
    let mut chars = value.chars();
    let mut is_escaped = false;
    while let Some(c) = chars.next() {
        if c == '\\' && !is_escaped {
            is_escaped = true;
        } else if is_escaped && c == 'n' {
            result.push('\n');
            is_escaped = false;
        } else {
            result.push(c);
            is_escaped = false;
        }
    }
    result.iter().collect()
}
