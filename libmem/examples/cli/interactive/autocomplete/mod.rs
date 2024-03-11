use std::{
    fmt::{Debug, Display},
    ops::Deref,
    sync::Arc,
};

pub trait Node: Display + Debug + Send + Sync {
    fn matches(&self, input: &str) -> bool;

    fn exact(&self, input: &str) -> bool;

    fn complete(&self, input: &str) -> String;

    fn subtree(&self) -> &[Arc<dyn Node>];
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

    fn subtree(&self) -> &[Arc<dyn Node>] {
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

    fn subtree(&self) -> &[Arc<dyn Node>] {
        &[]
    }
}

#[derive(Debug, Clone)]
pub struct StringCompleter<T>
where
    T: Deref<Target = str> + Sized + Send + Sync + Debug,
{
    pub string: T,
    pub subtree: Arc<[Arc<dyn Node>]>,
}

impl<T> Node for StringCompleter<T>
where
    T: Deref<Target = str> + Sized + Send + Sync + Debug,
{
    fn matches(&self, input: &str) -> bool {
        self.string.contains(input)
    }

    fn exact(&self, input: &str) -> bool {
        self.string.deref() == input
    }

    fn complete(&self, input: &str) -> String {
        let mut parts = input.split_whitespace().collect::<Vec<_>>();
        if parts.len() == 0 {
            self.string.to_owned()
        } else {
            if !input.ends_with(' ') {
                let last = parts.len() - 1;
                parts[last] = self.string.deref();
            } else {
                parts.push(&self.string.deref());
            }

            let first = parts[0].to_owned();

            parts
                .into_iter()
                .skip(1)
                .fold(first, |l, r| format!("{l} {r}"))
        }
    }

    fn subtree(&self) -> &[Arc<dyn Node>] {
        &self.subtree
    }
}

impl<T> Display for StringCompleter<T>
where
    T: Deref<Target = str> + Sized + Send + Sync + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string.deref())
    }
}

#[derive(Debug, Clone)]
pub struct ArgumentField<T>
where
    T: Deref<Target = str> + Sized + Send + Sync + Debug,
{
    pub string: T,
    pub subtree: Arc<[Arc<dyn Node>]>,
}

impl<T> Node for ArgumentField<T>
where
    T: Deref<Target = str> + Sized + Send + Sync + Debug,
{
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

    fn subtree(&self) -> &[Arc<dyn Node>] {
        &self.subtree
    }
}

impl<T> Display for ArgumentField<T>
where
    T: Deref<Target = str> + Sized + Send + Sync + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.string.deref(), f)
    }
}
