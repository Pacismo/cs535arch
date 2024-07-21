mod api;
mod config;
mod states;

use std::{
    error::Error,
    time::{Duration, Instant},
};

use states::Runtimes;
use uuid::Uuid;

const PAGES: usize = 16;

/// How long each session lasts (5 minutes)
const LIFETIME: Duration = Duration::from_secs(5 * 60);
/// How often the garbage collector runs (1 minute between runs)
const GC_FREQUENCY: Duration = Duration::from_secs(60);

async fn gc(runtimes: Runtimes) {
    loop {
        tokio::time::sleep(GC_FREQUENCY).await;

        let mut lock = runtimes.write().await;
        let now = Instant::now();

        // Get a list of UUIDs for expired sessions (time since last usage MUST not exceed `LIFETIME`)
        let expired: Vec<Uuid> = lock
            .values()
            .flat_map(|v| v.try_lock().ok())
            .filter_map(|v| (v.last_used + LIFETIME < now).then_some(v.uuid))
            .collect();

        let count = expired.len();

        for x in expired {
            lock.remove(&x);
        }

        if count > 0 {
            println!("Cleared {count} expired instances");
        }
    }
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let runtimes = Runtimes::default();

    let gc = tokio::spawn(gc(runtimes.clone()));

    let rocket = rocket::build()
        .mount("/", api::web::exports())
        .mount("/simulation", api::app::exports())
        .manage(runtimes.clone())
        .launch();

    rocket.await?;

    // Lock to await final GC
    let _lock = runtimes.write().await;
    gc.abort();

    Ok(())
}
