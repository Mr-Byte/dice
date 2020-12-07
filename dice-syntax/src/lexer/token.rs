use crate::SyntaxError;
use dice_core::span::Span;
use logos::{Lexer, Logos};
use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    iter::Iterator,
    rc::Rc,
};

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    span: Span,
}

impl Token {
    pub fn tokenize(input: &str) -> impl Iterator<Item = Result<Token, SyntaxError>> + '_ {
        let lexer = TokenKind::lexer(input);
        let error = lexer.extras.clone();

        lexer.spanned().map(move |(kind, range)| {
            let span = Span::new(range);
            error.error().map_or_else(|| Ok(Token { kind, span }), |err| todo!())
        })
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub const fn end_of_input() -> Token {
        Self {
            kind: TokenKind::EndOfInput,
            span: Span::new(0..0),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(extras = LexerResult)]
pub enum TokenKind {
    // End of input.
    EndOfInput,
    // Delimiters
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftCurly,
    #[token("}")]
    RightCurly,
    #[token("[")]
    LeftSquare,
    #[token("]")]
    RightSquare,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("|")]
    Pipe,
    // Operators
    #[token("..")]
    RangeExclusive,
    #[token("..=")]
    RangeInclusive,
    #[token("->")]
    Arrow,
    #[token("=>")]
    WideArrow,
    #[token(".")]
    Dot,
    #[token("?")]
    QuestionMark,
    #[token("!!")]
    ErrorPropagate,
    #[token("??")]
    Coalesce,
    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[token("%")]
    Remainder,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("!")]
    Not,
    #[token("!=")]
    NotEqual,
    #[token("==")]
    Equal,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token("=")]
    Assign,
    #[token("*=")]
    MulAssign,
    #[token("/=")]
    DivAssign,
    #[token("+=")]
    AddAssign,
    #[token("-=")]
    SubAssign,
    #[token("d")]
    DiceRoll,
    #[token("&&")]
    LazyAnd,
    #[token("|>")]
    Pipeline,
    // Keywords
    #[token(r"#")]
    Object,
    #[token("false")]
    False,
    #[token("true")]
    True,
    #[token("null")]
    Null,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("loop")]
    Loop,
    #[token("for")]
    For,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("return")]
    Return,
    #[token("fn")]
    Function,
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,
    #[token("in")]
    In,
    #[token("op")]
    Operator,
    #[token("class")]
    Class,
    #[token("is")]
    Is,
    #[token("import")]
    Import,
    #[token("as")]
    As,
    #[token("from")]
    From,
    #[token("export")]
    Export,
    #[token("super")]
    Super,
    #[regex("await|async|yield|do|const|match|enum|trait|type|try|when")]
    Reserved,

    // Literals,
    #[regex("(d[_a-zA-Z][_a-zA-Z0-9]*)|([_a-ce-zA-Z][_a-zA-Z0-9]*)", |lex| lex.slice().to_owned())]
    Identifier(String),
    #[regex("[0-9]+", parse_int)]
    Integer(i64),
    #[regex(r"[0-9]+\.[0-9]+", parse_float)]
    Float(f64),
    #[regex(r#"""#, lex_string)]
    String(String),

    // TODO: Propagate error for unexpected tokens.
    #[error]
    #[regex(r"[ \t\r\n\f]+|//[^\r\n]+", logos::skip)]
    Error,
}

#[derive(Debug, Default, Clone)]
pub struct LexerResult(Rc<RefCell<Option<()>>>);

impl LexerResult {
    fn error(&self) -> Option<()> {
        self.0.borrow().clone()
    }
}

fn parse_int(lexer: &mut Lexer<TokenKind>) -> Option<i64> {
    match lexer.slice().parse() {
        Ok(int) => Some(int),
        Err(err) => {
            *lexer.extras.0.borrow_mut() = Some(());
            None
        }
    }
}

fn parse_float(lexer: &mut Lexer<TokenKind>) -> Option<f64> {
    match lexer.slice().parse() {
        Ok(float) => Some(float),
        Err(err) => {
            *lexer.extras.0.borrow_mut() = Some(());
            None
        }
    }
}

fn lex_string(lexer: &mut Lexer<TokenKind>) -> Option<String> {
    let remainder = lexer.remainder();
    let mut result = String::new();
    let mut chars = remainder.chars();
    let mut bump_count = 0;

    loop {
        match chars.next() {
            Some('\\') => {
                let next = chars.next();

                match next {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some(next) => {
                        let sequence = format!("\\{}", next);
                        // Unknown escape sequence
                        *lexer.extras.0.borrow_mut() = Some(());
                        return None;
                    }
                    None => {
                        // Unterminated string
                        *lexer.extras.0.borrow_mut() = Some(());
                        return None;
                    }
                }

                bump_count += '\\'.len_utf8();
                bump_count += next.unwrap().len_utf8();
            }
            Some('"') => {
                bump_count += '"'.len_utf8();
                break;
            }
            Some(current) => {
                bump_count += current.len_utf8();
                result.push(current);
            }
            None => {
                // Unterminated string
                *lexer.extras.0.borrow_mut() = Some(());
                return None;
            }
        }
    }

    lexer.bump(bump_count);

    Some(result)
}
