mod backend;
mod tui;

pub use backend::Backend;
use libpipe::Pipeline;
use std::fmt::Debug;
pub use tui::Tui;

use crate::config::SimulationConfiguration;

/// Represents a means of running an interface.
///
/// Perhaps additional configuration is necessary; thus, using a datastructure
/// to represent the interface will allow such configuration without requiring
/// a change in the interface.
pub trait Interface: Debug {
    type Ok: Debug;
    type Error: Debug;

    fn run(
        self,
        pipeline: Box<dyn Pipeline>,
        config: SimulationConfiguration,
    ) -> Result<Self::Ok, Self::Error>;
}
