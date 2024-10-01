use std::io::IsTerminal;

macro_rules! colour_my_pencils {
    ($colour_code:expr, $text:expr) => {{
        let is_terminal = std::io::stdout().is_terminal();

        if !is_terminal {
            return $text.to_string();
        }

        format!("\x1b[{}m{}\x1b[0m", $colour_code, $text)
    }};
}

pub trait Colour
where
    Self: std::fmt::Display + std::fmt::Debug,
{
    fn red(&self) -> String {
        colour_my_pencils!("0;1;31", self)
    }

    fn blue(&self) -> String {
        colour_my_pencils!("0;1;34", self)
    }

    fn green(&self) -> String {
        colour_my_pencils!("0;1;32", self)
    }
}

impl Colour for &str {}
impl Colour for String {}
