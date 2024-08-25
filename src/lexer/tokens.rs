#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Identifier,
    String,
    Number,
    Float,
    True,
    False,
    None,
    Mut,

    PareL,    // (
    PareR,    // )
    BraceL,   // {
    BraceR,   // }
    BracketL, // [
    BracketR, // ]
    Sentinal,
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

    pub fn is_op(&self) -> bool {
        use TokenKind as TT;

        match *self {
            TT::Equals | TT::Expo | TT::Add | TT::Sub | TT::Multi | TT::Div => true,
            _ => false,
        }
    }
}
