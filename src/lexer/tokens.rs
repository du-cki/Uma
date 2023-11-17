#[derive(Debug, PartialEq)]
pub enum TokenType {
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
    Func
}

impl TokenType {
    pub fn precedence(&self) -> u8 {
        match *self {
            TokenType::Expo                   => 3,
            TokenType::Div | TokenType::Multi => 2,
            TokenType::Add | TokenType::Sub   => 1,
            _                                 => 0,
        }
    }

    pub fn is_op(&self) -> bool {
        use TokenType as TT;

        match *self {
            TT::Equals | TT::Expo | TT::Add | TT::Sub | TT::Multi | TT::Div => true,
            _ => false,
        }
    }
}
