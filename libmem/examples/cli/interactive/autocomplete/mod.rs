use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

pub trait Node: Display + Debug {
    fn matches(&self, input: &str) -> bool;

    fn exact(&self, input: &str) -> bool;

    fn complete(&self, input: &str) -> String;

    fn subtree(&self) -> &[Rc<dyn Node>];
}

impl Node for String {
    fn matches(&self, input: &str) -> bool {
        self.contains(input)
    }

    fn exact(&self, input: &str) -> bool {
        self == input
    }

    fn complete(&self, input: &str) -> String {
        let mut parts = input.split_whitespace().collect::<Vec<_>>();
        if parts.len() == 0 {
            self.clone()
        } else {
            let last = parts.len() - 1;
            parts[last] = self.as_str();

            let first = parts[0].to_owned();

            parts
                .into_iter()
                .skip(1)
                .fold(first, |l, r| format!("{l} {r}"))
        }
    }

    fn subtree(&self) -> &[Rc<dyn Node>] {
        &[]
    }
}

impl Node for &'static str {
    fn matches(&self, input: &str) -> bool {
        self.contains(input)
    }

    fn exact(&self, input: &str) -> bool {
        *self == input
    }

    fn complete(&self, input: &str) -> String {
        let mut parts = input.split_whitespace().collect::<Vec<_>>();
        if parts.len() == 0 {
            self.to_string()
        } else {
            if !input.ends_with(' ') {
                let last = parts.len() - 1;
                parts[last] = self;
            } else {
                parts.push(&self);
            }

            let first = parts[0].to_owned();

            parts
                .into_iter()
                .skip(1)
                .fold(first, |l, r| format!("{l} {r}"))
        }
    }

    fn subtree(&self) -> &[Rc<dyn Node>] {
        &[]
    }
}

#[derive(Debug, Clone)]
pub struct StringCompleter {
    pub string: String,
    pub subtree: Vec<Rc<dyn Node>>,
}

impl Node for StringCompleter {
    fn matches(&self, input: &str) -> bool {
        self.string.contains(input)
    }

    fn exact(&self, input: &str) -> bool {
        self.string == input
    }

    fn complete(&self, input: &str) -> String {
        let mut parts = input.split_whitespace().collect::<Vec<_>>();
        if parts.len() == 0 {
            self.string.clone()
        } else {
            if !input.ends_with(' ') {
                let last = parts.len() - 1;
                parts[last] = self.string.as_str();
            } else {
                parts.push(&self.string.as_str());
            }

            let first = parts[0].to_owned();

            parts
                .into_iter()
                .skip(1)
                .fold(first, |l, r| format!("{l} {r}"))
        }
    }

    fn subtree(&self) -> &[Rc<dyn Node>] {
        &self.subtree
    }
}

impl Display for StringCompleter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}

#[derive(Debug, Clone)]
pub struct ArgumentField {
    pub string: String,
    pub subtree: Vec<Rc<dyn Node>>,
}

impl Node for ArgumentField {
    fn matches(&self, _: &str) -> bool {
        true
    }

    fn exact(&self, _: &str) -> bool {
        true
    }

    fn complete(&self, input: &str) -> String {
        if input.ends_with(' ') {
            let mut parts: Vec<&str> = input.split_whitespace().collect();
            parts.push(&self.string);
            let first = parts[0].to_string();

            parts
                .into_iter()
                .skip(1)
                .fold(first, |a, e| format!("{a} {e}"))
        } else {
            input.to_owned()
        }
    }

    fn subtree(&self) -> &[Rc<dyn Node>] {
        &self.subtree
    }
}

impl Display for ArgumentField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.string, f)
    }
}
