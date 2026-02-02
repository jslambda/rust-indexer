//! Sample crate for rust2json tests.

/// Utility helpers module.
pub mod utils;

/// Greeter struct.
pub struct Greeter {
    pub name: String,
}

/// Greeting trait.
pub trait Greet {
    /// Say hello.
    fn greet(&self) -> String;
}

/// Possible errors.
pub enum ErrorKind {
    Network,
    Unknown,
}

/// Builds default greeter.
pub fn make_greeter() -> Greeter {
    Greeter { name: "world".into() }
}

impl Greeter {
    /// Return greeting.
    pub fn greeting(&self) -> String {
        format!("Hello {}", self.name)
    }
}
