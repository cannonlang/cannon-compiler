use std::{iter::Peekable, ops::Deref};

use unicode_xid::UnicodeXID;

use crate::{
    error::CannonError,
    span::{Pos, Span},
};

#[derive(Debug)]
pub enum TokenType {
    Comment,
    Eof,
    Id,
    Num,
    Punct,
}

#[derive(Debug)]
pub struct Token {
    pub ty: TokenType,
    pub body: String,
}

impl Token {
    fn eof(end: Option<char>) -> Self {
        Token {
            ty: TokenType::Eof,
            body: end.map_or_else(|| "".into(), |x| x.into()),
        }
    }

    fn comment(comment: String) -> Self {
        Token {
            ty: TokenType::Comment,
            body: comment,
        }
    }

    fn id(id: String) -> Self {
        Token {
            ty: TokenType::Id,
            body: id,
        }
    }

    fn num(num: String) -> Self {
        Token {
            ty: TokenType::Num,
            body: num,
        }
    }

    fn punct(punct: String) -> Self {
        Token {
            ty: TokenType::Punct,
            body: punct,
        }
    }
}

#[derive(Debug)]
pub enum GroupType {
    Paren,   // ()
    Bracket, // []
    Brace,   // {}
}

impl GroupType {
    fn end_char(&self) -> char {
        match self {
            Self::Paren => ')',
            Self::Bracket => ']',
            Self::Brace => '}',
        }
    }

    fn from_start_char(c: char) -> Option<Self> {
        match c {
            '(' => Some(Self::Paren),
            '[' => Some(Self::Bracket),
            '{' => Some(Self::Brace),
            _ => None,
        }
    }

    fn build(self, body: Vec<Lexeme>) -> Group {
        Group { ty: self, body }
    }
}

#[derive(Debug)]
pub struct Group {
    pub ty: GroupType,
    pub body: Vec<Lexeme>,
}

impl Deref for Token {
    type Target = String;

    fn deref(&self) -> &String {
        &self.body
    }
}

#[derive(Debug)]
pub enum LexemeBody {
    Token(Token),
    Group(Group),
}

impl From<Token> for LexemeBody {
    fn from(token: Token) -> Self {
        Self::Token(token)
    }
}

impl From<Group> for LexemeBody {
    fn from(group: Group) -> Self {
        Self::Group(group)
    }
}

#[derive(Debug)]
pub struct Lexeme {
    pub span: Span,
    pub body: LexemeBody,
}

impl Lexeme {
    fn new(span: impl Into<Span>, body: impl Into<LexemeBody>) -> Self {
        Self {
            span: span.into(),
            body: body.into(),
        }
    }
}

impl Deref for Lexeme {
    type Target = LexemeBody;

    fn deref(&self) -> &LexemeBody {
        &self.body
    }
}

pub fn lex_group(
    file: &mut Peekable<impl Iterator<Item = char>>,
    pos: &mut Pos,
    end_char: Option<char>,
) -> Result<Vec<Lexeme>, CannonError> {
    let mut result = Vec::new();
    loop {
        match file.next() {
            x if x == end_char => {
                result.push(Lexeme::new(*pos, Token::eof(x)));
                if x.is_some() {
                    pos.1 += 1;
                }
                break;
            },
            None => Err(CannonError::Eof(*pos))?,
            Some(' ') => pos.1 += 1,
            Some('\n') => {
                pos.0 += 1;
                pos.1 = 1;
            }
            Some(x) if x.is_xid_start() || x == '_' => {
                let mut id = x.to_string();
                let start = *pos;
                pos.1 += 1;
                while let Some(x) = file.peek() {
                    if !x.is_xid_continue() { break; }
                    id.push(*x);
                    file.next();
                    pos.1 += 1;
                }
                result.push(Lexeme::new((start, *pos), Token::id(id)));
            }
            Some(x) if x.is_numeric() => {
                let mut num = x.to_string();
                let start = *pos;
                pos.1 += 1;
                while let Some(x) = file.peek() {
                    if !x.is_xid_continue() { break; }
                    num.push(*x);
                    file.next();
                    pos.1 += 1;
                }
                result.push(Lexeme::new((start, *pos), Token::num(num)));
            }
            Some(c @ ('#' | ';')) => {
                let start = *pos;
                pos.1 += 1;
                result.push(Lexeme::new((start, *pos), Token::punct(c.into())));
            }
            Some(':') => {
                let start = *pos;
                pos.1 += 1;
                let punct = if let Some(':') = file.peek() {
                    file.next();
                    pos.1 += 1;
                    "::".into()
                } else {
                    ":".into()
                };
                result.push(Lexeme::new((start, *pos), Token::punct(punct)));
            }
            Some('*') => {
                let mut punct = String::from("-");
                let start = *pos;
                pos.1 += 1;
                if let Some(c @ '=') = file.peek() {
                    punct.push(*c);
                    file.next();
                    pos.1 += 1;
                }
                result.push(Lexeme::new((start, *pos), Token::punct(punct)));
            }
            Some('-') => {
                let mut punct = String::from("-");
                let start = *pos;
                pos.1 += 1;
                if let Some(c @ ('>' | '-' | '=')) = file.peek() {
                    punct.push(*c);
                    file.next();
                    pos.1 += 1;
                }
                result.push(Lexeme::new((start, *pos), Token::punct(punct)));
            }
            Some('/') => {
                let start = *pos;
                pos.1 += 1;
                if let Some('/') = file.peek() {
                    let mut text = String::from("//");
                    file.next();
                    pos.1 += 1;
                    while let Some(&c) = file.peek() {
                        if c == '\n' {
                            break;
                        }
                        text.push(c);
                        file.next();
                        pos.1 += 1;
                    }
                    result.push(Lexeme::new((start, *pos), Token::comment(text)));
                } else if let Some('*') = file.peek() {
                    todo!("Multiline?")
                } else {
                    result.push(Lexeme::new(start, Token::punct(":".into())));
                }
            }
            Some(x) if let Some(ty) = GroupType::from_start_char(x) => {
                let start = *pos;
                pos.1 += 1;
                let inner = lex_group(file, pos, Some(ty.end_char()))?;
                result.push(Lexeme::new(Span { start, end: *pos }, ty.build(inner)));
            }
            Some(x) => Err(CannonError::UnexpectedChar(x, *pos))?,
        }
    }
    Ok(result)
}

pub fn lex(file: impl Iterator<Item = char>) -> Result<Vec<Lexeme>, CannonError> {
    lex_group(&mut file.peekable(), &mut Pos(1, 1), None)
}
