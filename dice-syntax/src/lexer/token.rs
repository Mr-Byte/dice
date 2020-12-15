use dice_core::{
    error::{
        codes::{INVALID_ESCAPE_SEQUENCE, UNTERMINATED_STRING},
        Error, ResultExt as _,
    },
    source::Source,
    span::Span,
};
use logos::{Lexer, Logos};
use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    iter::Iterator,
    rc::Rc,
};

#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub span: Span,
    pub slice: &'a str,
}

impl<'a> Token<'a> {
    pub fn tokenize(input: &'a Source) -> TokenIter<'a> {
        TokenIter::new(input)
    }

    pub const fn end_of_input() -> Token<'a> {
        Self {
            kind: TokenKind::EndOfInput,
            span: Span::new(0..0),
            slice: "",
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

pub struct TokenIter<'a> {
    source: &'a Source,
    lexer: logos::Lexer<'a, TokenKind>,
}

impl<'a> TokenIter<'a> {
    fn new(source: &'a Source) -> Self {
        let lexer = TokenKind::lexer(source.source());
        Self { lexer, source }
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.lexer.next()?;
        let span = Span::new(self.lexer.span());
        let slice = self.lexer.slice();
        let result = self
            .lexer
            .extras
            .error()
            .map_or_else(|| Ok(Token { kind, span, slice }), Err)
            .with_span(span)
            .with_source(|| self.source.clone());

        Some(result)
    }
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
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
    // TODO: Change the parser to look for a double Bang token instead of this token.
    #[token("!!")]
    ErrorPropagate,
    // TODO: Change the parser to look for a double Question token instead of this token.
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
    #[regex("(d[_a-zA-Z][_a-zA-Z0-9]*)|([_a-ce-zA-Z][_a-zA-Z0-9]*)")]
    Identifier,
    #[regex("[0-9]+")]
    Integer,
    #[regex(r"[0-9]+\.[0-9]+")]
    Float,
    #[regex(r#"""#, lex_string)]
    String,

    // TODO: Propagate error for unexpected tokens.
    #[error]
    #[regex(r"[ \t\r\n\f]+|//[^\r\n]+", logos::skip)]
    Error,
}

impl Display for TokenKind {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::EndOfInput => {}
            TokenKind::LeftParen => {}
            TokenKind::RightParen => {}
            TokenKind::LeftCurly => {}
            TokenKind::RightCurly => {}
            TokenKind::LeftSquare => {}
            TokenKind::RightSquare => {}
            TokenKind::Semicolon => {}
            TokenKind::Colon => {}
            TokenKind::Comma => {}
            TokenKind::Pipe => {}
            TokenKind::RangeExclusive => {}
            TokenKind::RangeInclusive => {}
            TokenKind::Arrow => {}
            TokenKind::WideArrow => {}
            TokenKind::Dot => {}
            TokenKind::QuestionMark => {}
            TokenKind::ErrorPropagate => {}
            TokenKind::Coalesce => {}
            TokenKind::Minus => {}
            TokenKind::Plus => {}
            TokenKind::Remainder => {}
            TokenKind::Star => {}
            TokenKind::Slash => {}
            TokenKind::Not => {}
            TokenKind::NotEqual => {}
            TokenKind::Equal => {}
            TokenKind::Greater => {}
            TokenKind::GreaterEqual => {}
            TokenKind::Less => {}
            TokenKind::LessEqual => {}
            TokenKind::Assign => {}
            TokenKind::MulAssign => {}
            TokenKind::DivAssign => {}
            TokenKind::AddAssign => {}
            TokenKind::SubAssign => {}
            TokenKind::DiceRoll => {}
            TokenKind::LazyAnd => {}
            TokenKind::Pipeline => {}
            TokenKind::Object => {}
            TokenKind::False => {}
            TokenKind::True => {}
            TokenKind::Null => {}
            TokenKind::If => {}
            TokenKind::Else => {}
            TokenKind::While => {}
            TokenKind::Loop => {}
            TokenKind::For => {}
            TokenKind::Break => {}
            TokenKind::Continue => {}
            TokenKind::Return => {}
            TokenKind::Function => {}
            TokenKind::Let => {}
            TokenKind::Mut => {}
            TokenKind::In => {}
            TokenKind::Operator => {}
            TokenKind::Class => {}
            TokenKind::Is => {}
            TokenKind::Import => {}
            TokenKind::As => {}
            TokenKind::From => {}
            TokenKind::Export => {}
            TokenKind::Super => {}
            TokenKind::Reserved => {}
            TokenKind::Identifier => {}
            TokenKind::Integer => {}
            TokenKind::Float => {}
            TokenKind::String => {}
            TokenKind::Error => {}
        }

        todo!()
    }
}

#[derive(Default, Clone)]
pub struct LexerResult(Rc<RefCell<Option<Error>>>);

impl LexerResult {
    fn error(&self) -> Option<Error> {
        self.0.borrow_mut().take()
    }
}

fn lex_string(lexer: &mut Lexer<TokenKind>) -> bool {
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
                        *lexer.extras.0.borrow_mut() =
                            Some(Error::new(INVALID_ESCAPE_SEQUENCE).with_tags(dice_core::error_tags! {
                                sequence => format!("\\{}", next)
                            }));
                        return false;
                    }
                    None => {
                        *lexer.extras.0.borrow_mut() = Some(Error::new(UNTERMINATED_STRING));
                        return false;
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
                *lexer.extras.0.borrow_mut() = Some(Error::new(UNTERMINATED_STRING));
                return false;
            }
        }
    }

    lexer.bump(bump_count);

    true
}
