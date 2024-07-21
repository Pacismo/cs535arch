mod config;

use std::{collections::HashMap, str::FromStr, sync::Arc, time::Instant};

use crate::{
    config::{CacheConfiguration, SimulationConfiguration},
    states::{Runtime, Runtimes},
};
use config::InitFormData;
use libseis::types::Word;
use rocket::{
    form::Form, get, http, post, response::content::RawHtml, routes, serde::json::Json, Route,
    State,
};
use serde_json::Value;
use tokio::sync::Mutex;
use uuid::Uuid;

async fn get_uuid(runtimes: &Runtimes, uuid: Uuid) -> Option<Arc<Mutex<Runtime>>> {
    let lock = runtimes.read().await;
    lock.get(&uuid).map(|r| r.clone())
}

#[post("/", data = "<config>")]
pub async fn init(
    runtimes: &State<Runtimes>,
    config: Form<InitFormData>,
) -> Result<RawHtml<String>, (http::Status, String)> {
    let mut uuid = Uuid::new_v4();

    let mut lock = runtimes.write().await;

    while lock.contains_key(&uuid) {
        uuid = Uuid::new_v4();
    }

    let config = SimulationConfiguration::new(
        config.miss_penalty,
        config.volatile_penalty,
        config.writethrough,
        config.pipelining,
        [
            (
                "data",
                CacheConfiguration::new(
                    config.cache_data_set_bits,
                    config.cache_data_offset_bits,
                    config.cache_data_ways,
                ),
            ),
            (
                "instruction",
                CacheConfiguration::new(
                    config.cache_instruction_set_bits,
                    config.cache_instruction_offset_bits,
                    config.cache_instruction_ways,
                ),
            ),
        ],
    );

    lock.insert(uuid, Runtime::new(uuid, config));

    println!("Created a new configuration");

    Ok(RawHtml(format!(include_str!("response.html"), uuid = uuid)))
}

#[get("/<uuid>")]
pub async fn dashboard(
    runtimes: &State<Runtimes>,
    uuid: &str,
) -> Result<String, (http::Status, String)> {
    let uuid = Uuid::from_str(uuid).map_err(|e| (http::Status::BadRequest, e.to_string()))?;

    let runtime_arc = get_uuid(runtimes, uuid).await.ok_or((
        http::Status::BadRequest,
        format!("UUID {uuid} is not an active simulation"),
    ))?;
    let mut runtime = runtime_arc.lock().await;
    runtime.last_used = Instant::now();

    Ok(format!(
        "{uuid}: {:#}\n{:#?}",
        runtime.config.to_json(),
        runtime
    ))
}

#[get("/<uuid>/watchlist")]
pub async fn read_watchlist(
    runtimes: &State<Runtimes>,
    uuid: &str,
) -> Result<Json<HashMap<u32, String>>, (http::Status, String)> {
    let uuid = Uuid::from_str(uuid).map_err(|e| (http::Status::BadRequest, e.to_string()))?;

    let runtime_arc = get_uuid(runtimes, uuid)
        .await
        .ok_or((
            http::Status::NotFound,
            "The provided UUID does not correspond to an active simulation".into(),
        ))?
        .clone();
    let mut runtime = runtime_arc.lock().await;
    runtime.last_used = Instant::now();

    Ok(Json(runtime.read_watchlist()))
}

#[get("/<uuid>/registers")]
pub async fn read_registers(
    runtimes: &State<Runtimes>,
    uuid: &str,
) -> Result<Json<Value>, (http::Status, String)> {
    let uuid = Uuid::from_str(uuid).map_err(|e| (http::Status::BadRequest, e.to_string()))?;

    let runtime_arc = get_uuid(runtimes, uuid)
        .await
        .ok_or((
            http::Status::NotFound,
            "The provided UUID does not correspond to an active simulation".into(),
        ))?
        .clone();
    let mut runtime = runtime_arc.lock().await;
    runtime.last_used = Instant::now();

    Ok(Json(runtime.read_regs()))
}

#[get("/<uuid>/memory/<address>/<type>")]
pub async fn read_address(
    runtimes: &State<Runtimes>,
    uuid: &str,
    address: Word,
    r#type: &str,
) -> Result<String, (http::Status, String)> {
    let uuid = Uuid::from_str(uuid).map_err(|e| (http::Status::BadRequest, e.to_string()))?;

    let runtime_arc = get_uuid(runtimes, uuid)
        .await
        .ok_or((
            http::Status::NotFound,
            "The provided UUID does not correspond to an active simulation".into(),
        ))?
        .clone();
    let mut runtime = runtime_arc.lock().await;
    runtime.last_used = Instant::now();

    match r#type.to_lowercase().as_str() {
        "byte" => Ok(runtime
            .state
            .memory_module()
            .memory()
            .read_byte(address)
            .to_string()),
        "short" => Ok(runtime
            .state
            .memory_module()
            .memory()
            .read_short(address)
            .to_string()),
        "word" => Ok(runtime
            .state
            .memory_module()
            .memory()
            .read_word(address)
            .to_string()),
        "float" => Ok(
            f32::from_bits(runtime.state.memory_module().memory().read_word(address)).to_string(),
        ),
        t => Err((
            http::Status::BadRequest,
            format!("\"{t}\" is not a valid type"),
        )),
    }
}

pub fn exports() -> Vec<Route> {
    routes![
        init,
        read_address,
        dashboard,
        read_watchlist,
        read_registers
    ]
}
