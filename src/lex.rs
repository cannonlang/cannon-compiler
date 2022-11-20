use core::fmt;
use core::{fmt::Debug, iter::Peekable, ops::Deref};

use unicode_xid::UnicodeXID;

use crate::Error;
use crate::{
    span::{Pos, Span},
};

#[derive(Debug, Eq, PartialEq)]
pub enum TokenType {
    Comment,
    Id,
    Num,
    Punct,
    String,
}

pub struct Token {
    pub ty: TokenType,
    pub body: String,
}

impl Token {
    const fn comment(comment: String) -> Self {
        Self {
            ty: TokenType::Comment,
            body: comment,
        }
    }

    const fn id(id: String) -> Self {
        Self {
            ty: TokenType::Id,
            body: id,
        }
    }

    const fn num(num: String) -> Self {
        Self {
            ty: TokenType::Num,
            body: num,
        }
    }

    const fn punct(punct: String) -> Self {
        Self {
            ty: TokenType::Punct,
            body: punct,
        }
    }

    const fn string(string: String) -> Self {
        Self {
            ty: TokenType::String,
            body: string,
        }
    }

    #[must_use]
    pub fn is_keyword(&self) -> bool {
        self.ty == TokenType::Id && matches!(&*self.body, "fn" | "pub" | "type")
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token({:?}, {:?})", self.ty, self.body)
    }
}

impl Deref for Token {
    type Target = String;

    fn deref(&self) -> &String {
        &self.body
    }
}

#[derive(Debug)]
pub enum GroupType {
    Paren,   // ()
    Bracket, // []
    Brace,   // {}
}

impl GroupType {
    const fn end_char(&self) -> char {
        match self {
            Self::Paren => ')',
            Self::Bracket => ']',
            Self::Brace => '}',
        }
    }

    const fn from_start_char(c: char) -> Option<Self> {
        match c {
            '(' => Some(Self::Paren),
            '[' => Some(Self::Bracket),
            '{' => Some(Self::Brace),
            _ => None,
        }
    }

    const fn build(self, body: Vec<Lexeme>) -> Group {
        Group { ty: self, body }
    }
}

pub struct Group {
    pub ty: GroupType,
    pub body: Vec<Lexeme>,
}

impl Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Group(")?;
        self.ty.fmt(f)?;
        f.write_str(", ")?;
        self.body.fmt(f)?;
        f.write_str(")")
    }
}

pub enum LexemeBody {
    Token(Token),
    Group(Group),
}

impl Debug for LexemeBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Token(t) => t.fmt(f),
            Self::Group(g) => g.fmt(f),
        }
    }
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

pub struct Lexeme {
    pub span: Span,
    pub body: LexemeBody,
}

impl Debug for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.body.fmt(f)
    }
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

#[allow(clippy::missing_errors_doc)]
#[allow(clippy::missing_panics_doc)]
#[allow(clippy::too_many_lines)]
pub fn do_group(
    file: &mut Peekable<impl Iterator<Item = char>>,
    pos: &mut Pos,
    end_char: Option<char>,
) -> Result<Vec<Lexeme>, Error> {
    let mut result = Vec::new();
    loop {
        match file.next() {
            x if x == end_char => {
                if x.is_some() {
                    pos.1 += 1;
                }
                break;
            },
            None => Err(Error::Eof(*pos))?,
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
            Some(c @ ('#' | ';' | '=')) => {
                let start = *pos;
                pos.1 += 1;
                result.push(Lexeme::new((start, *pos), Token::punct(c.into())));
            }
            Some(':') => {
                let start = *pos;
                pos.1 += 1;
                let punct = if file.peek() == Some(&':') {
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
                if file.peek() == Some(&'/') {
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
                } else if file.peek() == Some(&'*') {
                    todo!("Multiline?");
                } else {
                    result.push(Lexeme::new(start, Token::punct("/".into())));
                }
            }
            Some('"') => {
                let start = *pos;
                pos.1 += 1;
                let mut text = String::new();
                loop {
                    let Some(c) = file.next() else { Err(Error::Eof(*pos))? };
                    if c == '\n' {
                        Err(Error::UnexpectedChar(c, *pos))?;
                    } else if c == '\\' {
                        todo!();
                    } else if c == '"' {
                        break;
                    } else {
                        pos.1 += 1;
                        text.push(c);
                    }
                }
                pos.1 += 1;
                result.push(Lexeme::new((start, *pos), Token::string(text)));
            }
            Some(x) if let Some(ty) = GroupType::from_start_char(x) => {
                let start = *pos;
                pos.1 += 1;
                let inner = do_group(file, pos, Some(ty.end_char()))?;
                result.push(Lexeme::new((start, *pos), ty.build(inner)));
            }
            Some(x) => Err(Error::UnexpectedChar(x, *pos))?,
        }
    }
    Ok(result)
}

#[allow(clippy::missing_errors_doc)]
pub fn lex(file: impl Iterator<Item = char>) -> Result<Vec<Lexeme>, Error> {
    do_group(&mut file.peekable(), &mut Pos(1, 1), None)
}

#[must_use]
fn highlight_group(pos: &mut Pos, group: &[Lexeme]) -> String {
    let mut result = String::new();
    for lexeme in group {
        let start = lexeme.span.start;
        while *pos != start {
            if pos.0 < start.0 {
                pos.1 = 1;
                pos.0 += 1;
                result += "\n";
            } else if pos.1 < start.1 {
                pos.1 += 1;
                result += " ";
            } else {
                panic!("invalid span");
            }
        }
        match &lexeme.body {
            LexemeBody::Token(token) => {
                match token.ty {
                    TokenType::Comment => result += "\x1B[34m",
                    TokenType::Id => {
                        if token.is_keyword() {
                            result += "\x1B[31m";
                        } else {
                            result += "\x1B[37m";
                        }
                    }
                    TokenType::Num => result += "\x1B[36m",
                    TokenType::Punct => result += "\x1B[33m",
                    TokenType::String => result += "\x1B[32m",
                }
                if token.ty == TokenType::String {
                    result.push('"');
                    result += &token.body;
                    result.push('"');
                    pos.1 += token.body.len() + 2;
                } else {
                    result += &token.body;
                    pos.1 += token.body.len();
                }
            }
            LexemeBody::Group(group) => {
                result += "\x1B[35m";
                match group.ty {
                    GroupType::Brace => result += "{",
                    GroupType::Bracket => result += "[",
                    GroupType::Paren => result += "(",
                }
                pos.1 += 1;
                result += &highlight_group(pos, &group.body);
                let mut end = lexeme.span.end;
                end.1 -= 1; // One column backward, since the span is inclusive
                while *pos != end {
                    if pos.0 < end.0 {
                        pos.1 = 1;
                        pos.0 += 1;
                        result += "\n";
                    } else if pos.1 < end.1 {
                        pos.1 += 1;
                        result += " ";
                    } else {
                        panic!("invalid span");
                    }
                }
                result += "\x1B[35m";
                match group.ty {
                    GroupType::Brace => result += "}",
                    GroupType::Bracket => result += "]",
                    GroupType::Paren => result += ")",
                }
                pos.1 += 1;
            }
        }
    }
    result
}

#[must_use]
pub fn highlight(file: &[Lexeme]) -> String {
    highlight_group(&mut Pos(1, 1), file) + "\x1B[0m"
}
