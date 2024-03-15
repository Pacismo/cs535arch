use serde_json::ser::{CompactFormatter, PrettyFormatter};
use std::io::Write;

pub type JsonSerializer<'a, F> = &'a mut serde_json::Serializer<&'a mut dyn Write, F>;

pub trait Serializable<S>
where
    S: serde::Serializer,
{
    fn serialize_to(&self, ser: S) -> Result<S::Ok, S::Error>;
}

pub type CompactJson<'a> = JsonSerializer<'a, CompactFormatter>;
pub type PrettyJson<'a> = JsonSerializer<'a, PrettyFormatter<'a>>;

impl<T: ?Sized + serde::Serialize, S: serde::Serializer> Serializable<S> for T {
    fn serialize_to(
        &self,
        ser: S,
    ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error> {
        self.serialize(ser)
    }
}
