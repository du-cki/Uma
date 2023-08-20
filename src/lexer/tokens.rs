
#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    String(std::string::String),
    Number(String),
    Float(String),
    Bool(String),
    None,

    PareL,    // (
    PareR,    // )
    BraceL,   // {
    BraceR,   // }
    BracketL, // [
    BracketR, // ]
    Semi,
    Dot,
    Comma,


    Equals,
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
