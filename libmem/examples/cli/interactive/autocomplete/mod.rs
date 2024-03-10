use std::fmt::{Debug, Display};

pub trait Node: Display + Debug {
    fn matches(&self, input: &str) -> bool;

    fn exact(&self, input: &str) -> bool;

    fn complete(&self, input: &str) -> String;

    fn subtree(&self) -> &[Box<dyn Node>];

    fn box_clone(&self) -> Box<dyn Node>;
}

impl Clone for Box<dyn Node> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
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

    fn subtree(&self) -> &[Box<dyn Node>] {
        &[]
    }

    fn box_clone(&self) -> Box<dyn Node> {
        Box::new(self.to_owned())
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

    fn subtree(&self) -> &[Box<dyn Node>] {
        &[]
    }

    fn box_clone(&self) -> Box<dyn Node> {
        Box::new(*self)
    }
}

#[derive(Debug)]
pub struct StringCompleter {
    pub string: String,
    pub subtree: Vec<Box<dyn Node>>,
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

    fn subtree(&self) -> &[Box<dyn Node>] {
        &self.subtree
    }

    fn box_clone(&self) -> Box<dyn Node> {
        Box::new(Self {
            string: self.string.clone(),
            subtree: self.subtree.clone(),
        })
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
    pub subtree: Vec<Box<dyn Node>>,
}

impl Node for ArgumentField {
    fn matches(&self, _: &str) -> bool {
        true
    }

    fn exact(&self, _: &str) -> bool {
        true
    }

    fn complete(&self, input: &str) -> String {
        input.to_owned()
    }

    fn subtree(&self) -> &[Box<dyn Node>] {
        &self.subtree
    }

    fn box_clone(&self) -> Box<dyn Node> {
        Box::new(Self {
            string: self.string.clone(),
            subtree: self.subtree.clone(),
        })
    }
}

impl Display for ArgumentField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.string, f)
    }
}
