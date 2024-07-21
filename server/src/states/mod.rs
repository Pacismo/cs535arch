mod runtimes;

pub use runtimes::Runtime;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

pub type Runtimes = Arc<RwLock<HashMap<Uuid, Arc<Mutex<Runtime>>>>>;
