use std::{
    fmt::{Debug, Display},
    ops::Deref,
    sync::Arc,
};

pub trait Node: Display + Debug + Send + Sync {
    fn matches(&self, input: &str) -> bool;

    #[allow(unused)]
    fn exact(&self, input: &str) -> bool;

    fn complete(&self, input: &[&str], display: bool) -> Vec<String>;

    #[allow(unused)]
    fn subtree(&self) -> &[Arc<dyn Node>];
}

impl Node for String {
    fn matches(&self, input: &str) -> bool {
        self.contains(input)
    }

    fn exact(&self, input: &str) -> bool {
        self == input
    }

    fn complete(&self, input: &[&str], _: bool) -> Vec<String> {
        if input.len() == 0 {
            vec![]
        } else if input[0].is_empty() {
            vec![self.clone()]
        } else if self.matches(input[0]) {
            vec![self.clone()]
        } else {
            vec![]
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

    fn complete(&self, input: &[&str], _: bool) -> Vec<String> {
        if input.len() == 0 {
            vec![]
        } else if input[0].len() == 0 {
            vec![self.to_string()]
        } else if self.matches(input[0]) {
            vec![self.to_string()]
        } else {
            vec![]
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

    fn complete(&self, input: &[&str], display: bool) -> Vec<String> {
        if input.len() == 0 {
            vec![]
        } else if input[0].len() == 0 {
            vec![self.string.to_string()]
        } else if self.matches(input[0]) {
            if input.len() > 1 {
                self.subtree
                    .iter()
                    .flat_map(|c| {
                        c.complete(&input[1..], display)
                            .into_iter()
                            .map(|s| format!("{} {}", self.string.deref(), s))
                    })
                    .collect()
            } else {
                vec![self.string.to_string()]
            }
        } else {
            vec![]
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

    fn complete(&self, input: &[&str], display: bool) -> Vec<String> {
        if input.len() == 0 {
            vec![]
        } else if input[0].len() == 0 {
            if display {
                vec![self.string.to_string()]
            } else {
                vec![]
            }
        } else {
            if input.len() > 1 {
                self.subtree
                    .iter()
                    .flat_map(|c| {
                        c.complete(&input[1..], display)
                            .into_iter()
                            .map(|s| format!("{} {}", input[0], s))
                    })
                    .collect()
            } else {
                vec![input[0].to_string()]
            }
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
