#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Identifier,
    String,
    Number,
    Float,
    None,
    True,
    False,

    PareL,    // (
    PareR,    // )
    BraceL,   // {
    BraceR,   // }
    BracketL, // [
    BracketR, // ]
    Colon,
    Semi,
    Dot,
    Comma,

    Equals,
    Expo,
    Add,
    Sub,
    Multi,
    Div,
    At,
    Ellipsis,

    Let,
    Mut,
    If,
    Else,
    Return,
    Func,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub value: Option<String>,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, value: Option<String>, line: usize, column: usize) -> Token {
        Token {
            kind,
            value,
            line,
            column,
        }
    }

    pub fn repr(&self) -> String {
        match self.kind {
            TokenKind::Add => String::from("+"),
            TokenKind::Sub => String::from("-"),
            TokenKind::Multi => String::from("*"),
            TokenKind::Div => String::from("/"),
            _ => panic!("Unknown token kind."),
        }
    }
}

impl TokenKind {
    pub fn precedence(&self) -> i8 {
        use TokenKind as TT;

        match *self {
            TT::Expo => 3,
            TT::Div | TT::Multi => 2,
            TT::Add | TT::Sub => 1,
            _ => -1,
        }
    }
}
