use std::collections::HashMap;

pub enum ArgValue {
    Bool(bool),
    String(String),
    List(Vec<String>),
}

pub struct ArgMatches {
    pub(crate) values: HashMap<String, ArgValue>,
}

impl ArgMatches {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn get_string(&self, name: &str) -> Option<&String> {
        match self.values.get(name) {
            Some(ArgValue::String(s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_bool(&self, name: &str) -> bool {
        match self.values.get(name) {
            Some(ArgValue::Bool(b)) => *b,
            _ => false,
        }
    }

    pub fn get_vec(&self, name: &str) -> Option<&Vec<String>> {
        match self.values.get(name) {
            Some(ArgValue::List(l)) => Some(l),
            _ => None,
        }
    }
}
