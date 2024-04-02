use super::Interface;
use std::error::Error;

#[derive(Debug, Clone, Copy)]
pub struct Backend;

impl Interface for Backend {
    type Ok = ();

    type Error = Box<dyn Error>;

    fn run(self, pipeline: Box<dyn libpipe::Pipeline>) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}
