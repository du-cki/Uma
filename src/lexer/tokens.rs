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

    Let,
    Mut,
    If,
    Else,
    Return,
    Func,
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
