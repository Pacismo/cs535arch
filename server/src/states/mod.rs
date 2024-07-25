mod runtimes;

pub use runtimes::*;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::sync::RwLock;
use uuid::Uuid;

pub type Runtimes = Arc<RwLock<HashMap<Uuid, (Arc<RwLock<Runtime>>, Mutex<Instant>)>>>;
