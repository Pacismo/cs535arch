//! This is a library that enables object-safe serialization.
//!
//! [`Serializable`] is implemented for all types implementing [`Serialize`](serde::Serialize)
//! for all types implementing [`Serializer`](serde::Serializer).
//!
//! This is primarily used for argument-forwarding, as [`Serialize`](serde::Serialize) is
//! not an object-safe trait (since it uses generic parameters in its associated function)

use serde_json::ser::{CompactFormatter, PrettyFormatter};
use std::io::Write;

pub type JsonSerializer<'a, F> = &'a mut serde_json::Serializer<&'a mut dyn Write, F>;
pub type CompactJson<'a> = JsonSerializer<'a, CompactFormatter>;
pub type PrettyJson<'a> = JsonSerializer<'a, PrettyFormatter<'a>>;

/// Represents the requirement that an object be serializable to a particular serializer.
///
/// For instance, you may want a type implementing a trait to be serializable to [`PrettyJson`],
/// so you'd use:
///
/// ```ignore
/// use serde::Serialize;
/// use libser::{Serializable, PrettyJson};
///
/// #[derive(Serialize)]
/// trait Foo<'a>: Serializable<PrettyJson<'a>> {
///     /// --snip--
/// }
///
/// impl<'a> Serialize for dyn Foo<'a> {
///     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
///     where
///         S: serde::Serializer,
///         Self: Serializable<S>, // Only available for explicitly-enabled serializers
///     {
///         self.serialize_to(serializer)
///     }
/// }
/// ```
///
/// The implementation of [`Serialize`](serde::Serialize) for `dyn Foo` allows us to
/// serialize any object implementing `Foo` using Serde's `Serialize` trait rather than
/// having to explicitly implement `Serialize` for any object with a `dyn Foo` as a field.
///
/// Thus, this code becomes legal:
///
/// ```ignore
/// #[derive(Serialize)]
/// pub struct Bar(Box<dyn Foo>);
/// ```
///
/// And now `Bar` implements `Serialize` without needing to have it be implemented manually.
///
/// `Serializable` is blanket-implemented for all types implementing [`Serialize`](serde::Serialize)
/// for all types implementing [`Serializer`](serde::Serializer), so no explicit implementation is
/// required.
pub trait Serializable<S>
where
    S: serde::Serializer,
{
    fn serialize_to(&self, ser: S) -> Result<S::Ok, S::Error>;
}

impl<T, S> Serializable<S> for T
where
    T: ?Sized + serde::Serialize,
    S: serde::Serializer,
{
    fn serialize_to(
        &self,
        ser: S,
    ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error> {
        self.serialize(ser)
    }
}
